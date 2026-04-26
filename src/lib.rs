use std::{
    env::current_exe,
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use mimalloc::MiMalloc;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HMODULE},
        System::{
            Console::{AllocConsole, SetConsoleTitleW},
            LibraryLoader::LoadLibraryW,
            SystemServices::DLL_PROCESS_ATTACH,
        },
        UI::Shell::{FOLDERID_SystemX86, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath},
    },
    core::{HSTRING, w},
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[allow(non_snake_case)]
mod exports;

#[derive(Serialize, Deserialize, Default)]
struct Config {
    console: bool,
}

fn enable_console() -> Result<()> {
    unsafe { AllocConsole()? };
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .init();
    unsafe {
        SetConsoleTitleW(w!("Cardamom Loader"))
            .map_err(|e| anyhow!("Failed to set console title: {e}"))?
    };
    Ok(())
}

fn get_exe_dir() -> Result<PathBuf> {
    let mut exe = current_exe().map_err(|e| anyhow!("Failed to get current executable: {e}"))?;
    if !exe.pop() {
        bail!("Failed to get executable's parent directory")
    }
    Ok(exe)
}

fn create_new_config(path: &Path) -> Result<()> {
    let mut file = File::create(path)
        .map_err(|e| anyhow!("Failed to create config file at {}: {e}", path.display()))?;
    let new = Config::default();
    let serialized = toml::to_string_pretty(&new)
        .map_err(|e| anyhow!("Failed to serialize default config to toml: {e}"))?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| anyhow!("Failed to write to new config file: {e}"))?;
    Ok(())
}

fn read_config(path: &Path) -> Result<Config> {
    let config: Config = {
        match std::fs::read(path) {
            Ok(bytes) => match toml::from_slice(&bytes) {
                Ok(config) => config,
                Err(e) => {
                    let mut error_string = format!("Failed to parse config file: {e}. ");
                    match create_new_config(path) {
                        Ok(_) => error_string.push_str("Successfully created new config file."),
                        Err(e) => error_string.push_str(&format!(
                            "Failed to create new config file at {}: {e}",
                            path.display()
                        )),
                    }
                    bail!(anyhow!(error_string));
                }
            },
            Err(e) => {
                let mut error_string =
                    format!("Failed to read config file at {}: {e}. ", path.display());
                match create_new_config(path) {
                    Ok(_) => error_string.push_str("Successfully created new config file."),
                    Err(e) => error_string.push_str(&format!(
                        "Failed to create new config file at {}: {e}",
                        path.display()
                    )),
                }
                bail!(anyhow!(error_string))
            }
        }
    };
    Ok(config)
}

fn load_plugins(path: &Path) -> Result<()> {
    for entry in path
        .read_dir()
        .map_err(|e| anyhow!("Failed to read plugin dir at {}: {e}", path.display()))?
        .flatten()
    {
        let entry_path = entry.path();
        if let Some(ext) = entry_path.extension()
            && ext == "dll"
        {
            let absolute_path = entry_path.canonicalize().with_context(|| {
                format!(
                    "Failed to convert path at {} to absolute.",
                    entry_path.display()
                )
            })?;
            let plugin_name = entry_path.file_name().unwrap_or(OsStr::new("?")).display();
            unsafe {
                match LoadLibraryW(&HSTRING::from(absolute_path.as_path())) {
                    Ok(_) => info!("Loaded {}", plugin_name),
                    Err(e) => error!("Error loading {}, skipping: {e}", plugin_name),
                }
            };
        }
    }
    Ok(())
}

fn load_original_dll() -> Result<HMODULE> {
    unsafe {
        let id = FOLDERID_SystemX86;
        let system_path = SHGetKnownFolderPath(&id, KNOWN_FOLDER_FLAG::default(), None)
            .context("Error finding Windows system directory")?;
        let mut path = system_path.to_string().with_context(|| {
            format!(
                "Failed to convert system path {} to Rust string",
                system_path.display()
            )
        })?;
        path.push_str("\\dsound.dll");
        let path_hstring = HSTRING::from(path);
        let module = LoadLibraryW(&path_hstring).context("Failed to load original dsound.dll")?;
        Ok(module)
    }
}

fn main() -> Result<()> {
    match load_original_dll() {
        Ok(dll) => {
            exports::init(dll);
        }
        Err(e) => {
            enable_console()?;
            bail!(e);
        }
    };

    match get_exe_dir() {
        Ok(exe_dir) => {
            let config_path = exe_dir.join("cardamom-loader.toml");
            match read_config(&config_path) {
                Ok(config) => {
                    if config.console {
                        enable_console()?;
                        info!("Initialized successfully");
                    }
                    let plugins_path = exe_dir.join("plugins");
                    if !plugins_path.is_dir() {
                        info!("No plugins directory found at {}", plugins_path.display());
                    } else {
                        info!("Loading plugins from {}", plugins_path.display());
                        load_plugins(&plugins_path)
                            .map_err(|e| anyhow!("Error loading plugins: {e}"))?;
                    }
                }
                Err(e) => {
                    enable_console()?;
                    bail!(e)
                }
            }
        }
        Err(e) => {
            enable_console()?;
            bail!(e)
        }
    }
    info!("Finished loading");
    Ok(())
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(_hinst: HINSTANCE, fdw_reason: u32, _lpv_reserved: *mut ()) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH
        && let Err(e) = main()
    {
        error!("{e}");
    }
    true
}
