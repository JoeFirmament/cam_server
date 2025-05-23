//! 错误处理模块

use thiserror::Error;

/// 摄像头核心模块错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("摄像头设备错误: {0}")]
    CameraDevice(String),

    #[error("视频处理错误: {0}")]
    VideoProcessing(String),

    #[error("FFmpeg错误: {0}")]
    FFmpeg(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("Nokhwa错误: {0}")]
    Nokhwa(String),

    #[error("图像处理错误: {0}")]
    Image(String),

    #[error("其他错误: {0}")]
    Other(String),
}

// 从nokhwa错误转换
impl From<nokhwa::NokhwaError> for Error {
    fn from(err: nokhwa::NokhwaError) -> Self {
        Error::Nokhwa(err.to_string())
    }
}