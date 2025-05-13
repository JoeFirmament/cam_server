//! 配置模块

use serde::{Deserialize, Serialize};

/// 摄像头配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// 摄像头设备路径，如 "/dev/video0"
    pub device_path: String,
    
    /// 视频宽度
    pub width: u32,
    
    /// 视频高度
    pub height: u32,
    
    /// 帧率
    pub fps: u32,
    
    /// 像素格式，如 "YUYV", "MJPG"
    pub pixel_format: String,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            device_path: "/dev/video0".to_string(),
            width: 1920,
            height: 1080,
            fps: 30,
            pixel_format: "YUYV".to_string(),
        }
    }
}

/// 视频录制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// 输出目录
    pub output_dir: String,
    
    /// 视频编码器，如 "h264", "h265"
    pub encoder: String,
    
    /// 视频容器格式，如 "mp4", "mkv"
    pub container: String,
    
    /// 视频比特率，单位为 bps
    pub bitrate: u32,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            output_dir: "./recordings".to_string(),
            encoder: "h264".to_string(),
            container: "mp4".to_string(),
            bitrate: 4_000_000, // 4 Mbps
        }
    }
}

/// 视频拆分配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitConfig {
    /// 输出目录
    pub output_dir: String,
    
    /// 图像格式，如 "jpg", "png"
    pub image_format: String,
    
    /// 提取帧的频率，如每秒提取几帧
    pub frame_rate: f32,
    
    /// 图像质量 (1-100)
    pub quality: u8,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            output_dir: "./frames".to_string(),
            image_format: "jpg".to_string(),
            frame_rate: 1.0, // 每秒1帧
            quality: 90,
        }
    }
}