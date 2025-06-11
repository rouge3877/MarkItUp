//! Global configuration management
//! This module handles the loading and parsing of configuration files
//! and environment variables for the application.
//! Usage:
//! ```rust
//! use markitup::config::SETTINGS;
//! // fn main() {
//! //     let cfg = &*SETTINGS.read().unwrap();
//! //     println!("{:?}", cfg.model_path);
//! // }

use config::{Config, ConfigError, Environment, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{env, fs, path::PathBuf, sync::RwLock};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub model_path: PathBuf,
    pub image_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub is_ai_entitle: bool,
    pub doubao_api_key: Option<String>,
    pub is_ai_sweep: bool,
    pub deepseek_api_key: Option<String>,
}

// debug print settings
fn debug_print_settings(settings: &Settings) {
    if cfg!(debug_assertions) {
        eprintln!("=== Configuration Settings ===");
        eprintln!("model_path: {:?}", settings.model_path);
        eprintln!("image_path: {:?}", settings.image_path);
        eprintln!("output_path: {:?}", settings.output_path);
        eprintln!("is_ai_entitle: {}", settings.is_ai_entitle);
        eprintln!("doubao_api_key: {:?}", settings.doubao_api_key.as_ref());
        eprintln!("is_ai_sweep: {}", settings.is_ai_sweep);
        eprintln!("deepseek_api_key: {:?}", settings.deepseek_api_key.as_ref());
        eprintln!("==============================");
    }
}

pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let settings = Settings::new().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });
    
    // Debug output for all configuration settings
    debug_print_settings(&settings);
    
    RwLock::new(settings)
});

// 提供一个便捷的访问函数，保持原有的使用方式
pub fn get_settings() -> Settings {
    SETTINGS.read().unwrap().clone()
}

// 添加更新配置的函数
pub fn update_settings_with_cli_args(
    image_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    ai_enable: Option<bool>,
) {
    let mut settings = SETTINGS.write().unwrap();

    if let Some(path) = image_path {
        settings.image_path = path;
    }

    if let Some(path) = output_path {
        settings.output_path = Some(path);
    }

    if let Some(enable) = ai_enable {
        settings.is_ai_entitle = enable;
        settings.is_ai_sweep = enable; // Assuming is_ai_sweep should also be updated
    } else {
        // 如果没有提供 ai_enable 参数，则一律disable AI功能
        settings.is_ai_entitle = false;
        settings.is_ai_sweep = false;
    }
    
    // Debug output after CLI updates
    debug_print_settings(&settings);
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // 1. built-in default config
        let mut builder = Config::builder()
            .add_source(File::from_str(
                include_str!("../Config.toml"),
                FileFormat::Toml,
            ));

        // 2. try to load external config file
        if let Ok(exe_path) = env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                let external = dir.join("Config.toml");
                if fs::metadata(&external).is_ok() {
                    builder = builder.add_source(
                        File::with_name(external.to_str().unwrap()).required(false),
                    );
                }
            }
        }

        // 3. load environment variables
        builder = builder.add_source(Environment::with_prefix("APP").separator("__"));

        // 构建并 Deserialize 到 Settings
        builder.build()?.try_deserialize()
    }
}
