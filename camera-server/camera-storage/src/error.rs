//! 错误处理模块

use thiserror::Error;

/// 存储管理模块错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("文件系统错误: {0}")]
    FileSystem(String),
    
    #[error("文件打包错误: {0}")]
    Packaging(String),
    
    #[error("存储空间错误: {0}")]
    DiskSpace(String),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("ZIP错误: {0}")]
    Zip(String),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("其他错误: {0}")]
    Other(String),
}
