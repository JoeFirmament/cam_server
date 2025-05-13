//! 摄像头采集和视频处理核心功能
//! 
//! 该模块提供摄像头设备检测、初始化、视频流采集、视频编码和录制、
//! 视频拆分为图像帧等功能。

pub mod camera;
pub mod video;
pub mod error;
pub mod config;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;