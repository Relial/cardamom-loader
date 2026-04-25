use std::{
    env::current_exe,
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow, bail};
use mimalloc::MiMalloc;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use windows::{
    Win32::{
        Foundation::HINSTANCE,
        System::{
            Console::{AllocConsole, SetConsoleTitleW},
            LibraryLoader::LoadLibraryW,
            SystemServices::DLL_PROCESS_ATTACH,
        },
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
            match entry_path.canonicalize() {
                Ok(absolute_path) => {
                    info!(
                        "Loading plugin: {}",
                        entry_path.file_name().unwrap_or(OsStr::new("?")).display()
                    );
                    unsafe {
                        match LoadLibraryW(&HSTRING::from(absolute_path.as_path())) {
                            Ok(_) => info!("Loaded successfully"),
                            Err(e) => error!("Error loading plugin, skipping: {e}"),
                        }
                    };
                }
                Err(e) => error!(
                    "Failed to convert path at {} to absolute: {e}",
                    entry_path.display()
                ),
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
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
                        info!("Plugins directory doesn't exist. Creating.");
                        match std::fs::create_dir(&plugins_path) {
                            Ok(_) => {
                                info!("Created successfully");
                                return Ok(());
                            }
                            Err(e) => bail!("Failed to create plugins directory: {e}"),
                        }
                    }
                    load_plugins(&plugins_path)
                        .map_err(|e| anyhow!("Error loading plugins: {e}"))?;
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
