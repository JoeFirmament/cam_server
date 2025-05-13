//! API服务和Web服务器
//! 
//! 该模块提供RESTful API接口、Web服务器和文件下载服务。

pub mod server;
pub mod routes;
pub mod error;
pub mod config;
pub mod state;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;