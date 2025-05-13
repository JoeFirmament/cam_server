//! 存储管理功能
//! 
//! 该模块提供文件系统操作、文件命名和组织、存储空间管理、
//! 静态帧文件夹管理和文件打包功能。

pub mod error;
pub mod file_manager;
pub mod frame_manager;
pub mod package;
pub mod disk;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
