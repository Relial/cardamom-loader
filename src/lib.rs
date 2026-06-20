use std::{
    env::current_exe,
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use mimalloc::MiMalloc;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HMODULE},
        System::{
            Console::{AllocConsole, SetConsoleTitleW},
            LibraryLoader::LoadLibraryW,
            SystemServices::DLL_PROCESS_ATTACH,
        },
        UI::{
            Shell::{FOLDERID_SystemX86, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath},
            WindowsAndMessaging::{MB_OK, MessageBoxW},
        },
    },
    core::{HSTRING, w},
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[allow(non_snake_case)]
mod exports;

#[derive(Serialize, Deserialize)]
struct Config {
    console: bool,
    plugins_folder_name: String,
}

impl Config {
    fn load_from_file(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config = toml::from_slice(&bytes).context("Failed to deserialize config file")?;
        Ok(config)
    }

    fn save_to_file(&self, path: &Path) -> Result<()> {
        let mut file = File::create_new(path)
            .with_context(|| format!("Failed to create file at {}", path.display()))?;
        let serialized = toml::to_string_pretty(self).context("Failed to serialize to toml")?;
        file.write_all(serialized.as_bytes())
            .with_context(|| format!("Failed to write to file at {}", path.display()))?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            console: false,
            plugins_folder_name: "plugins".to_string(),
        }
    }
}

fn enable_console() {
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .init();

    unsafe {
        if let Err(e) = AllocConsole() {
            debug!("Error allocating console: {e}");
        }
        if let Err(e) = SetConsoleTitleW(w!("Cardamom Loader")) {
            debug!("Failed to set console title: {e}");
        }
    }
}

fn get_exe_dir() -> Result<PathBuf> {
    let mut exe = current_exe().context("Failed to get current executable")?;
    if exe.pop() {
        Ok(exe)
    } else {
        Err(anyhow!("Failed to get executable's parent directory"))
    }
}

fn load_plugin(path: &Path) -> Result<()> {
    let absolute_path = path
        .canonicalize()
        .with_context(|| format!("Failed to convert path at {} to absolute", path.display()))?;
    unsafe { LoadLibraryW(&HSTRING::from(absolute_path.as_path()))? };
    Ok(())
}

fn load_plugins(path: &Path) -> Result<()> {
    for entry in path
        .read_dir()
        .with_context(|| format!("Failed to read plugin dir at {}", path.display()))?
        .flatten()
    {
        let entry_path = entry.path();
        if let Some(ext) = entry_path.extension()
            && ext == "dll"
        {
            if let Err(e) = load_plugin(&entry_path) {
                error!("Failed to load plugin at {}: {e:#}", entry_path.display());
            } else {
                let plugin_name = entry_path.file_name().unwrap_or(OsStr::new("?"));
                info!("Loaded {}", plugin_name.display());
            }
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
        path.push_str("\\dinput8.dll");
        let path_hstring = HSTRING::from(path);
        let module = LoadLibraryW(&path_hstring).context("Failed to load original dinput8.dll")?;
        Ok(module)
    }
}

fn main() -> Result<()> {
    let exe_dir = get_exe_dir()?;
    let config_path = exe_dir.join("cardamom-loader.toml");
    let config = Config::load_from_file(&config_path).unwrap_or_else(|e| {
        enable_console();
        warn!("{e:#}");
        let new = Config::default();
        if let Err(e) = new.save_to_file(&config_path) {
            error!("{e:#}");
        } else {
            info!("Created new config");
            info!("This console window will be hidden on future game launches. If you'd like it to open again, edit cardamom-loader.toml");
        }
        new
    });
    if config.console {
        enable_console();
    }

    let plugins_path = exe_dir.join(config.plugins_folder_name);
    if !plugins_path.is_dir() {
        info!("No plugins directory found at {}", plugins_path.display());
    } else {
        info!("Loading plugins from {}", plugins_path.display());
        load_plugins(&plugins_path).context("Error loading plugins")?;
        info!("Finished loading");
    }
    Ok(())
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(_hinst: HINSTANCE, fdw_reason: u32, _lpv_reserved: *mut ()) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH {
        match load_original_dll() {
            Ok(dll) => {
                exports::init(dll);
                if let Err(e) = main() {
                    error!("{e:#}");
                }
            }
            Err(e) => {
                let body = HSTRING::from(format!(
                    "Failed to load real dinput8.dll from system dir: {e:#}"
                ));
                let caption = HSTRING::from("Cardamom Loader error");
                unsafe { MessageBoxW(None, &body, &caption, MB_OK) };
            }
        }
    }
    true
}
