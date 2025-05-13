//! 配置管理模块

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use camera_core::config::{CameraConfig, RecordingConfig, SplitConfig};

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务器地址
    pub address: String,
    /// 服务器端口
    pub port: u16,
    /// 静态文件目录
    pub static_dir: String,
    /// 是否启用CORS
    pub enable_cors: bool,
    /// 是否启用HTTPS
    pub enable_https: bool,
    /// SSL证书路径
    pub ssl_cert: Option<String>,
    /// SSL密钥路径
    pub ssl_key: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 8080,
            static_dir: "./web-client".to_string(),
            enable_cors: true,
            enable_https: false,
            ssl_cert: None,
            ssl_key: None,
        }
    }
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 视频文件目录
    pub video_dir: String,
    /// 帧文件目录
    pub frames_dir: String,
    /// 打包文件目录
    pub package_dir: String,
    /// 最大磁盘使用率（0.0 - 1.0）
    pub max_disk_usage: f32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            video_dir: "./data/videos".to_string(),
            frames_dir: "./data/frames".to_string(),
            package_dir: "./data/packages".to_string(),
            max_disk_usage: 0.9,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志文件路径
    pub log_file: String,
    /// 日志级别
    pub level: String,
    /// 是否输出到控制台
    pub console_output: bool,
    /// 最大日志文件大小（MB）
    pub max_file_size: u64,
    /// 最大日志文件数量
    pub max_files: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_file: "./logs/camera-server.log".to_string(),
            level: "info".to_string(),
            console_output: true,
            max_file_size: 10,
            max_files: 5,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 摄像头配置
    pub camera: CameraConfig,
    /// 录制配置
    pub recording: RecordingConfig,
    /// 拆分配置
    pub split: SplitConfig,
    /// 服务器配置
    pub server: ServerConfig,
    /// 存储配置
    pub storage: StorageConfig,
    /// 日志配置
    pub log: LogConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            camera: CameraConfig::default(),
            recording: RecordingConfig::default(),
            split: SplitConfig::default(),
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            log: LogConfig::default(),
        }
    }
}

/// 加载配置文件
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
    let path = path.as_ref();
    
    // 如果配置文件不存在，创建默认配置
    if !path.exists() {
        let default_config = AppConfig::default();
        save_config(path, &default_config)?;
        return Ok(default_config);
    }
    
    // 读取配置文件
    let content = fs::read_to_string(path)
        .context(format!("读取配置文件失败: {}", path.display()))?;
        
    // 解析配置
    let config: AppConfig = match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => toml::from_str(&content)
            .context("解析TOML配置失败")?,
        Some("json") => serde_json::from_str(&content)
            .context("解析JSON配置失败")?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
            .context("解析YAML配置失败")?,
        _ => return Err(anyhow::anyhow!("不支持的配置文件格式")),
    };
    
    Ok(config)
}

/// 保存配置文件
pub fn save_config<P: AsRef<Path>>(path: P, config: &AppConfig) -> Result<()> {
    let path = path.as_ref();
    
    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context(format!("创建目录失败: {}", parent.display()))?;
    }
    
    // 序列化配置
    let content = match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => toml::to_string_pretty(config)
            .context("序列化TOML配置失败")?,
        Some("json") => serde_json::to_string_pretty(config)
            .context("序列化JSON配置失败")?,
        Some("yaml") | Some("yml") => serde_yaml::to_string(config)
            .context("序列化YAML配置失败")?,
        _ => return Err(anyhow::anyhow!("不支持的配置文件格式")),
    };
    
    // 写入配置文件
    fs::write(path, content)
        .context(format!("写入配置文件失败: {}", path.display()))?;
        
    Ok(())
}
