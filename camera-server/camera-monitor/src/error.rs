//! 错误处理模块

use thiserror::Error;

/// 系统监控模块错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("系统监控错误: {0}")]
    System(String),
    
    #[error("服务监控错误: {0}")]
    Service(String),
    
    #[error("日志错误: {0}")]
    Logger(String),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("其他错误: {0}")]
    Other(String),
}
