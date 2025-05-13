//! 摄像头设备模块
//!
//! 该模块提供跨平台的摄像头设备检测和访问功能，
//! 支持Linux(V4L2)和macOS(AVFoundation)平台。

use crate::{Error, Result, config::CameraConfig};
use log::{info, error, debug};
use std::path::Path;
use std::sync::{Arc, Mutex};
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::{Camera as NokhwaCamera, CameraFormat};

// 定义平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    /// Linux平台(RK3588)
    Linux,
    /// macOS平台
    MacOS,
    /// 其他平台
    Other,
}

// 获取当前运行平台
fn get_platform() -> PlatformType {
    #[cfg(target_os = "linux")]
    return PlatformType::Linux;

    #[cfg(target_os = "macos")]
    return PlatformType::MacOS;

    PlatformType::Other
}

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

/// 摄像头设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
    /// V4L2摄像头(Linux)
    V4L2,
    /// AVFoundation摄像头(macOS)
    AVFoundation,
    /// 模拟摄像头(用于测试)
    Mock,
}

/// 摄像头设备
pub struct Camera {
    /// 摄像头配置
    config: CameraConfig,

    /// 是否已初始化
    initialized: bool,

    /// 是否正在采集
    capturing: bool,

    /// 平台类型
    platform: PlatformType,

    /// 摄像头类型
    camera_type: CameraType,

    /// Nokhwa摄像头实例
    nokhwa_camera: Option<Arc<Mutex<NokhwaCamera>>>,
}

impl Camera {
    /// 创建新的摄像头实例
    pub fn new(config: CameraConfig) -> Self {
        let platform = get_platform();

        // 根据平台选择摄像头类型
        let camera_type = match platform {
            PlatformType::Linux => CameraType::V4L2,
            PlatformType::MacOS => CameraType::AVFoundation,
            PlatformType::Other => CameraType::Mock,
        };

        info!("创建摄像头实例，平台: {:?}, 类型: {:?}", platform, camera_type);

        Self {
            config,
            initialized: false,
            capturing: false,
            platform,
            camera_type,
            nokhwa_camera: None,
        }
    }

    /// 初始化摄像头
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        match self.camera_type {
            CameraType::V4L2 => self.initialize_v4l2(),
            CameraType::AVFoundation => self.initialize_avfoundation(),
            CameraType::Mock => self.initialize_mock(),
        }
    }

    /// 初始化V4L2摄像头(Linux)
    fn initialize_v4l2(&mut self) -> Result<()> {
        // 检查设备是否存在
        if !Path::new(&self.config.device_path).exists() {
            return Err(Error::CameraDevice(format!(
                "摄像头设备不存在: {}", self.config.device_path
            )));
        }

        // 使用nokhwa库初始化V4L2摄像头
        let index = CameraIndex::Path(self.config.device_path.clone());
        let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        match NokhwaCamera::new(index, requested_format) {
            Ok(camera) => {
                self.nokhwa_camera = Some(Arc::new(Mutex::new(camera)));
                info!("V4L2摄像头已初始化: {}", self.config.device_path);
                self.initialized = true;
                Ok(())
            },
            Err(e) => {
                error!("初始化V4L2摄像头失败: {}", e);
                Err(Error::CameraDevice(format!("初始化V4L2摄像头失败: {}", e)))
            }
        }
    }

    /// 初始化AVFoundation摄像头(macOS)
    fn initialize_avfoundation(&mut self) -> Result<()> {
        // 在macOS上，我们使用摄像头索引而不是设备路径
        // 尝试解析设备路径中的索引，如果失败则使用默认索引0
        let index_str = self.config.device_path.trim_start_matches("/dev/video");
        let index = index_str.parse::<u32>().unwrap_or(0);

        let camera_index = CameraIndex::Index(index);
        let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        match NokhwaCamera::new(camera_index, requested_format) {
            Ok(camera) => {
                self.nokhwa_camera = Some(Arc::new(Mutex::new(camera)));
                info!("AVFoundation摄像头已初始化，索引: {}", index);
                self.initialized = true;
                Ok(())
            },
            Err(e) => {
                error!("初始化AVFoundation摄像头失败: {}", e);
                Err(Error::CameraDevice(format!("初始化AVFoundation摄像头失败: {}", e)))
            }
        }
    }

    /// 初始化模拟摄像头(用于测试)
    fn initialize_mock(&mut self) -> Result<()> {
        info!("初始化模拟摄像头: {}", self.config.device_path);
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

        match self.camera_type {
            CameraType::V4L2 | CameraType::AVFoundation => {
                if let Some(camera) = &self.nokhwa_camera {
                    let mut camera = camera.lock().unwrap();
                    match camera.open_stream() {
                        Ok(_) => {
                            info!("开始视频采集: {}", self.config.device_path);
                            self.capturing = true;
                            Ok(())
                        },
                        Err(e) => {
                            error!("开始视频采集失败: {}", e);
                            Err(Error::CameraDevice(format!("开始视频采集失败: {}", e)))
                        }
                    }
                } else {
                    Err(Error::CameraDevice("摄像头未正确初始化".to_string()))
                }
            },
            CameraType::Mock => {
                info!("开始模拟视频采集: {}", self.config.device_path);
                self.capturing = true;
                Ok(())
            }
        }
    }

    /// 停止视频采集
    pub fn stop_capture(&mut self) -> Result<()> {
        if !self.capturing {
            return Ok(); // 已经停止采集
        }

        match self.camera_type {
            CameraType::V4L2 | CameraType::AVFoundation => {
                if let Some(camera) = &self.nokhwa_camera {
                    let mut camera = camera.lock().unwrap();
                    match camera.stop_stream() {
                        Ok(_) => {
                            info!("停止视频采集: {}", self.config.device_path);
                            self.capturing = false;
                            Ok(())
                        },
                        Err(e) => {
                            error!("停止视频采集失败: {}", e);
                            Err(Error::CameraDevice(format!("停止视频采集失败: {}", e)))
                        }
                    }
                } else {
                    Err(Error::CameraDevice("摄像头未正确初始化".to_string()))
                }
            },
            CameraType::Mock => {
                info!("停止模拟视频采集: {}", self.config.device_path);
                self.capturing = false;
                Ok(())
            }
        }
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
        let platform = get_platform();

        match platform {
            PlatformType::Linux => Self::list_v4l2_devices(),
            PlatformType::MacOS => Self::list_avfoundation_devices(),
            PlatformType::Other => Self::list_mock_devices(),
        }
    }

    /// 列出Linux系统中的V4L2摄像头设备
    fn list_v4l2_devices() -> Result<Vec<CameraInfo>> {
        let mut devices = Vec::new();

        // 使用nokhwa库列出设备
        match nokhwa::query_devices(nokhwa::utils::ApiBackend::Video4Linux) {
            Ok(camera_list) => {
                for (index, info) in camera_list {
                    let path = format!("/dev/video{}", index);

                    // 尝试获取设备支持的格式
                    let mut resolutions = Vec::new();
                    let mut pixel_formats = Vec::new();

                    // 尝试打开摄像头获取更多信息
                    if let Ok(camera) = NokhwaCamera::new(
                        CameraIndex::Index(index),
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate)
                    ) {
                        if let Ok(formats) = camera.compatible_camera_formats() {
                            for format in formats {
                                resolutions.push((format.width(), format.height()));
                                pixel_formats.push(format.format().to_string());
                            }
                        }
                    }

                    // 如果没有获取到格式信息，添加一些常见的格式
                    if resolutions.is_empty() {
                        resolutions = vec![(1920, 1080), (1280, 720), (640, 480)];
                    }

                    if pixel_formats.is_empty() {
                        pixel_formats = vec!["YUYV".to_string(), "MJPG".to_string()];
                    }

                    devices.push(CameraInfo {
                        path,
                        name: info.human_name().to_string(),
                        driver: "v4l2".to_string(),
                        resolutions,
                        pixel_formats,
                    });
                }
            },
            Err(e) => {
                error!("列出V4L2设备失败: {}", e);
                // 返回一个模拟的设备列表
                return Self::list_mock_devices();
            }
        }

        if devices.is_empty() {
            info!("未找到V4L2摄像头设备，返回模拟设备");
            return Self::list_mock_devices();
        }

        Ok(devices)
    }

    /// 列出macOS系统中的AVFoundation摄像头设备
    fn list_avfoundation_devices() -> Result<Vec<CameraInfo>> {
        let mut devices = Vec::new();

        // 使用nokhwa库列出设备
        match nokhwa::query_devices(nokhwa::utils::ApiBackend::AVFoundation) {
            Ok(camera_list) => {
                for (index, info) in camera_list {
                    let path = format!("/dev/video{}", index);

                    // 尝试获取设备支持的格式
                    let mut resolutions = Vec::new();
                    let mut pixel_formats = Vec::new();

                    // 尝试打开摄像头获取更多信息
                    if let Ok(camera) = NokhwaCamera::new(
                        CameraIndex::Index(index),
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate)
                    ) {
                        if let Ok(formats) = camera.compatible_camera_formats() {
                            for format in formats {
                                resolutions.push((format.width(), format.height()));
                                pixel_formats.push(format.format().to_string());
                            }
                        }
                    }

                    // 如果没有获取到格式信息，添加一些常见的格式
                    if resolutions.is_empty() {
                        resolutions = vec![(1920, 1080), (1280, 720), (640, 480)];
                    }

                    if pixel_formats.is_empty() {
                        pixel_formats = vec!["RGB".to_string(), "YUY2".to_string()];
                    }

                    devices.push(CameraInfo {
                        path,
                        name: info.human_name().to_string(),
                        driver: "avfoundation".to_string(),
                        resolutions,
                        pixel_formats,
                    });
                }
            },
            Err(e) => {
                error!("列出AVFoundation设备失败: {}", e);
                // 返回一个模拟的设备列表
                return Self::list_mock_devices();
            }
        }

        if devices.is_empty() {
            info!("未找到AVFoundation摄像头设备，返回模拟设备");
            return Self::list_mock_devices();
        }

        Ok(devices)
    }

    /// 列出模拟摄像头设备(用于测试)
    fn list_mock_devices() -> Result<Vec<CameraInfo>> {
        Ok(vec![
            CameraInfo {
                path: "/dev/video0".to_string(),
                name: "模拟摄像头".to_string(),
                driver: "mock".to_string(),
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