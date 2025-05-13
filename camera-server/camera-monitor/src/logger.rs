//! 日志记录和管理模块

use crate::{Error, Result};
use log::{info, error, debug, LevelFilter};
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 信息
    Info,
    /// 调试
    Debug,
    /// 跟踪
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warning => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 时间戳（Unix时间戳，毫秒）
    pub timestamp: u64,
    /// 日志级别
    pub level: LogLevel,
    /// 模块名称
    pub module: String,
    /// 日志消息
    pub message: String,
}

/// 日志管理器配置
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// 日志文件路径
    pub log_file: PathBuf,
    /// 日志级别
    pub level: LogLevel,
    /// 是否输出到控制台
    pub console_output: bool,
    /// 最大日志文件大小（字节）
    pub max_file_size: u64,
    /// 最大日志文件数量
    pub max_files: usize,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_file: PathBuf::from("camera-server.log"),
            level: LogLevel::Info,
            console_output: true,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_files: 5,
        }
    }
}

/// 日志管理器
pub struct Logger {
    /// 配置
    config: LoggerConfig,
}

impl Logger {
    /// 创建新的日志管理器
    pub fn new(config: LoggerConfig) -> Result<Self> {
        // 确保日志目录存在
        if let Some(parent) = config.log_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 初始化日志系统
        let level_filter: LevelFilter = config.level.into();
        
        let mut builder = env_logger::Builder::new();
        builder.filter_level(level_filter);
        
        if config.console_output {
            builder.target(env_logger::Target::Stdout);
        }
        
        // 添加文件输出
        // 注意：env_logger不直接支持同时输出到文件和控制台
        // 在实际应用中，可能需要使用更复杂的日志库如log4rs
        
        builder.init();
        
        info!("日志系统初始化完成，级别: {:?}", config.level);
        
        Ok(Self {
            config,
        })
    }
    
    /// 获取日志文件路径
    pub fn log_file_path(&self) -> &Path {
        &self.config.log_file
    }
    
    /// 设置日志级别
    pub fn set_level(&mut self, level: LogLevel) -> Result<()> {
        self.config.level = level;
        
        // 在实际应用中，这里应该重新配置日志系统的级别
        // 但env_logger不支持运行时更改级别
        
        info!("日志级别已更改为: {:?}", level);
        
        Ok(())
    }
    
    /// 读取日志文件
    pub fn read_log(&self, max_lines: usize) -> Result<Vec<String>> {
        let log_file = &self.config.log_file;
        
        if !log_file.exists() {
            return Ok(Vec::new());
        }
        
        let mut file = File::open(log_file)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let lines: Vec<String> = content.lines()
            .map(|line| line.to_string())
            .collect();
            
        if lines.len() <= max_lines {
            Ok(lines)
        } else {
            Ok(lines[lines.len() - max_lines..].to_vec())
        }
    }
    
    /// 清除日志文件
    pub fn clear_log(&self) -> Result<()> {
        let log_file = &self.config.log_file;
        
        if log_file.exists() {
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(log_file)?;
                
            drop(file);
            
            info!("日志文件已清除");
        }
        
        Ok(())
    }
    
    /// 轮转日志文件
    pub fn rotate_log(&self) -> Result<()> {
        let log_file = &self.config.log_file;
        
        if !log_file.exists() {
            return Ok(());
        }
        
        let metadata = std::fs::metadata(log_file)?;
        
        if metadata.len() < self.config.max_file_size {
            return Ok(());
        }
        
        // 轮转日志文件
        for i in (1..self.config.max_files).rev() {
            let src = log_file.with_extension(format!("log.{}", i));
            let dst = log_file.with_extension(format!("log.{}", i + 1));
            
            if src.exists() {
                std::fs::rename(&src, &dst)?;
            }
        }
        
        let backup = log_file.with_extension("log.1");
        std::fs::rename(log_file, &backup)?;
        
        // 创建新的日志文件
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(log_file)?;
            
        drop(file);
        
        info!("日志文件已轮转");
        
        Ok(())
    }
}
