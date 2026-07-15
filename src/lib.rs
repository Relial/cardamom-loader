use std::{
    env::{current_exe, set_var, var_os},
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::{debug, error, info, level_filters::LevelFilter, warn};
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

const OVERRIDE_DLL_NAME: &str = "dinput8_c.dll";

#[allow(non_snake_case)]
mod exports;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Default, Clone, Copy)]
#[repr(u8)]
enum LogLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    #[default]
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl LogLevel {
    fn as_str(&self) -> &str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    console: bool,
    plugins_folder_name: String,
    log_level: LogLevel,
}

impl Config {
    fn load_or_create_new(exe_dir: impl AsRef<Path>) -> Self {
        let config_path = exe_dir.as_ref().join("cardamom-loader.toml");
        Config::load_from_file(&config_path).unwrap_or_else(|e| {
            let new = Self::default();
            enable_console(new.log_level);
            warn!("{e:#}");
            if let Err(e) = create_default_config(config_path) {
                error!("{e:#}");
            } else {
                info!("Created new config");
                info!("This console window will be hidden on future game launches. If you'd like it to open again, edit cardamom-loader.toml");
            }
            new
        })
    }

    fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config = toml::from_slice(&bytes).context("Failed to deserialize config file")?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            console: false,
            plugins_folder_name: "cardamom".to_string(),
            log_level: Default::default(),
        }
    }
}

const DEFAULT_CONFIG: &str = r#"console = false
plugins_folder_name = "cardamom"

# 0 Off | 1 Error | 2 Warn | 3 Info | 4 Debug | 5 Trace
log_level = 3
"#;

fn create_default_config(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut file = File::create(path)
        .with_context(|| format!("Failed to create file at {}", path.display()))?;
    file.write_all(DEFAULT_CONFIG.as_bytes())
        .with_context(|| format!("Failed to write to file at {}", path.display()))?;
    Ok(())
}

fn enable_console(log_level: LogLevel) {
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(log_level)
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

fn load_dll(path: impl AsRef<Path>) -> Result<HMODULE> {
    let path = path.as_ref();
    let absolute_path = path
        .canonicalize()
        .with_context(|| format!("Failed to convert path {} to absolute", path.display()))?;
    let module = unsafe {
        LoadLibraryW(&HSTRING::from(absolute_path.as_path()))
            .with_context(|| format!("LoadLibraryW for path {} failed", absolute_path.display()))?
    };
    Ok(module)
}

fn load_plugins(path: &Path) -> Result<()> {
    for entry in path
        .read_dir()
        .with_context(|| format!("Failed to read plugin dir at {}", path.display()))?
        .flatten()
    {
        let entry_path = entry.path();
        if let Some(ext) = entry_path.extension()
            && ext.eq_ignore_ascii_case("dll")
        {
            if let Err(e) = load_dll(&entry_path) {
                error!("Failed to load plugin at {}: {e:#}", entry_path.display());
            } else {
                let plugin_name = entry_path.file_name().unwrap_or(OsStr::new("?"));
                info!("Loaded {}", plugin_name.display());
            }
        }
    }
    Ok(())
}

fn load_real_dinput() -> Result<HMODULE> {
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
        load_dll(&path).context("Failed to load original dinput8.dll")
    }
}

fn load_dinput(exe_dir: impl AsRef<Path>) -> Result<HMODULE> {
    let override_dll_path = exe_dir.as_ref().join(OVERRIDE_DLL_NAME);
    match load_dll(&override_dll_path) {
        Ok(module) => {
            info!("Loaded {OVERRIDE_DLL_NAME}");
            Ok(module)
        }
        Err(e) => {
            debug!("Failed to load {OVERRIDE_DLL_NAME}: {e:#}");
            load_real_dinput()
        }
    }
}

fn plugin_loader(plugins_dir: impl AsRef<Path>) -> Result<()> {
    let plugins_dir = plugins_dir.as_ref();
    if !plugins_dir.is_dir() {
        info!("No plugins directory found at {}", plugins_dir.display());
    } else {
        info!("Loading plugins from {}", plugins_dir.display());
        load_plugins(plugins_dir).context("Error loading plugins")?;
        info!("Finished loading");
    }
    Ok(())
}

fn message_box_error(error: anyhow::Error) {
    let body = HSTRING::from(format!("{error:#}"));
    let caption = HSTRING::from("Cardamom Loader error");
    unsafe { MessageBoxW(None, &body, &caption, MB_OK) };
}

fn dinput_load_error(error: anyhow::Error) {
    message_box_error(error.context("Failed to load real dinput8.dll from system dir"));
}

fn set_env_log_level(log_level: LogLevel) {
    unsafe {
        set_var("CARDAMOM_LOG_LEVEL", log_level.as_str());
    }
}

fn set_env_loaded() {
    unsafe {
        set_var("CARDAMOM_LOADED", "");
    }
}

fn get_env_loaded() -> bool {
    var_os("CARDAMOM_LOADED").is_some()
}

fn main() {
    let already_loaded = get_env_loaded();
    set_env_loaded();

    match get_exe_dir() {
        Ok(exe_dir) => {
            let config = Config::load_or_create_new(&exe_dir);
            if config.console {
                enable_console(config.log_level);
            }
            match load_dinput(&exe_dir) {
                Ok(dinput) => {
                    exports::init(dinput);
                    if already_loaded {
                        debug!("Already loaded, skipping plugin loading");
                    } else {
                        set_env_log_level(config.log_level);
                        let plugins_dir = exe_dir.join(config.plugins_folder_name);
                        if let Err(e) = plugin_loader(plugins_dir) {
                            error!("{e:#}");
                        }
                    }
                }
                Err(e) => {
                    dinput_load_error(e);
                }
            }
        }
        Err(e) => {
            enable_console(LogLevel::default());
            error!(
                "Failed to get exe directory, not loading {OVERRIDE_DLL_NAME} or plugins: {e:#}"
            );
            if let Err(e) = load_real_dinput() {
                dinput_load_error(e);
            }
        }
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(_hinst: HINSTANCE, fdw_reason: u32, _lpv_reserved: *mut ()) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH {
        main();
    }
    true
}
