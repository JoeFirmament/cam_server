//! 摄像头设备模块

use crate::{Error, Result, config::CameraConfig};
use log::{info, error, debug};
use std::path::Path;

/// 摄像头设备信息
#[derive(Debug, Clone)]
pub struct CameraInfo {
    /// 设备路径
    pub path: String,
    
    /// 设备名称
    pub name: String,
    
    /// 设备驱动
    pub driver: String,
    
    /// 设备支持的分辨率列表
    pub resolutions: Vec<(u32, u32)>,
    
    /// 设备支持的像素格式列表
    pub pixel_formats: Vec<String>,
}

/// 摄像头设备
pub struct Camera {
    /// 摄像头配置
    config: CameraConfig,
    
    /// 是否已初始化
    initialized: bool,
    
    /// 是否正在采集
    capturing: bool,
    
    // 这里将来会添加 V4L2 设备句柄
}

impl Camera {
    /// 创建新的摄像头实例
    pub fn new(config: CameraConfig) -> Self {
        Self {
            config,
            initialized: false,
            capturing: false,
        }
    }
    
    /// 初始化摄像头
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // 检查设备是否存在
        if !Path::new(&self.config.device_path).exists() {
            return Err(Error::CameraDevice(format!(
                "摄像头设备不存在: {}", self.config.device_path
            )));
        }
        
        // 这里将来会添加 V4L2 设备初始化代码
        
        info!("摄像头已初始化: {}", self.config.device_path);
        self.initialized = true;
        Ok(())
    }
    
    /// 开始视频采集
    pub fn start_capture(&mut self) -> Result<()> {
        if !self.initialized {
            return Err(Error::CameraDevice("摄像头未初始化".to_string()));
        }
        
        if self.capturing {
            return Ok(); // 已经在采集中
        }
        
        // 这里将来会添加开始采集的代码
        
        info!("开始视频采集: {}", self.config.device_path);
        self.capturing = true;
        Ok(())
    }
    
    /// 停止视频采集
    pub fn stop_capture(&mut self) -> Result<()> {
        if !self.capturing {
            return Ok(); // 已经停止采集
        }
        
        // 这里将来会添加停止采集的代码
        
        info!("停止视频采集: {}", self.config.device_path);
        self.capturing = false;
        Ok(())
    }
    
    /// 获取摄像头是否正在采集
    pub fn is_capturing(&self) -> bool {
        self.capturing
    }
    
    /// 获取摄像头配置
    pub fn config(&self) -> &CameraConfig {
        &self.config
    }
    
    /// 设置摄像头配置
    pub fn set_config(&mut self, config: CameraConfig) -> Result<()> {
        if self.capturing {
            return Err(Error::CameraDevice(
                "无法在采集过程中更改配置，请先停止采集".to_string()
            ));
        }
        
        self.config = config;
        
        // 如果已初始化，需要重新初始化以应用新配置
        if self.initialized {
            self.initialized = false;
            self.initialize()?;
        }
        
        Ok(())
    }
    
    /// 列出系统中的所有摄像头设备
    pub fn list_devices() -> Result<Vec<CameraInfo>> {
        // 这里将来会添加列出设备的代码
        // 暂时返回一个模拟的设备列表
        
        Ok(vec![
            CameraInfo {
                path: "/dev/video0".to_string(),
                name: "USB Camera".to_string(),
                driver: "uvcvideo".to_string(),
                resolutions: vec![(1920, 1080), (1280, 720), (640, 480)],
                pixel_formats: vec!["YUYV".to_string(), "MJPG".to_string()],
            }
        ])
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        if self.capturing {
            if let Err(e) = self.stop_capture() {
                error!("关闭摄像头时出错: {}", e);
            }
        }
    }
}