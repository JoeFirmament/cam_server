//! 系统监控功能
//! 
//! 该模块提供系统资源监控、服务状态监控和日志记录管理功能。

pub mod error;
pub mod system;
pub mod service;
pub mod logger;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
