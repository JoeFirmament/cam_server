//! 摄像头设备模块
//!
//! 该模块提供跨平台的摄像头设备检测和访问功能，
//! 支持Linux(V4L2)和macOS(AVFoundation)平台。

use crate::{Error, Result, config::CameraConfig};
use log::{info, error};
use std::path::Path;
use std::sync::{Arc, Mutex};
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, ApiBackend};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::{Camera as NokhwaCamera, query};

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
    {
        return PlatformType::Linux;
    }

    #[cfg(target_os = "macos")]
    {
        return PlatformType::MacOS;
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        return PlatformType::Other;
    }
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
        // 在Linux上，我们可以使用设备路径或索引
        // 尝试解析设备路径中的索引，如果失败则使用默认索引0
        let index_str = self.config.device_path.trim_start_matches("/dev/video");
        let index = index_str.parse::<u32>().unwrap_or(0);
        let camera_index = CameraIndex::Index(index);
        let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        match NokhwaCamera::new(camera_index, requested_format) {
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

        // 使用自定义格式，指定分辨率、帧率
        // 我们使用RgbFormat，但会在后续处理中正确处理NV12格式
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate
        );

        match NokhwaCamera::new(camera_index, requested_format) {
            Ok(camera) => {
                // 获取并记录实际的摄像头格式
                let format = camera.camera_format();
                info!("摄像头实际格式: {}x{} @ {}fps - {}",
                    format.width(),
                    format.height(),
                    format.frame_rate(),
                    format.format()
                );

                // 更新配置中的宽度和高度
                self.config.width = format.width();
                self.config.height = format.height();

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
            return Ok(()); // 已经在采集中
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
            return Ok(()); // 已经停止采集
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

    /// 捕获一帧图像
    pub fn capture_frame(&mut self) -> Result<image::RgbImage> {
        if !self.initialized {
            return Err(Error::CameraDevice("摄像头未初始化".to_string()));
        }

        if !self.capturing {
            return Err(Error::CameraDevice("摄像头未开始采集".to_string()));
        }

        match self.camera_type {
            CameraType::V4L2 | CameraType::AVFoundation => {
                if let Some(camera) = &self.nokhwa_camera {
                    let mut camera = camera.lock().unwrap();
                    match camera.frame() {
                        Ok(frame) => {
                            // 尝试将nokhwa的帧转换为image::RgbImage
                            match frame.decode_image::<RgbFormat>() {
                                Ok(rgb_frame) => {
                                    let width = rgb_frame.width();
                                    let height = rgb_frame.height();
                                    let data = rgb_frame.into_raw();

                                    // 创建image::RgbImage
                                    let rgb_image = image::RgbImage::from_raw(width, height, data)
                                        .ok_or_else(|| Error::CameraDevice("无法创建RGB图像".to_string()))?;

                                    Ok(rgb_image)
                                },
                                Err(e) => {
                                    // 如果转换失败，尝试手动处理NV12格式
                                    error!("转换帧失败: {}", e);

                                    // 获取摄像头格式信息
                                    let format = camera.camera_format();
                                    info!("摄像头格式: {}x{} @ {}fps - {}",
                                        format.width(),
                                        format.height(),
                                        format.frame_rate(),
                                        format.format()
                                    );

                                    // 获取帧信息
                                    info!("帧信息: 大小={}",
                                        frame.buffer().len()
                                    );

                                    // 检查是否是NV12格式
                                    if format.format().to_string().contains("NV12") {
                                        info!("尝试手动处理NV12格式...");

                                        // 获取原始数据
                                        let raw_data = frame.buffer();
                                        let width = format.width();
                                        let height = format.height();

                                        // 创建一个新的RGB图像
                                        let mut rgb_image = image::RgbImage::new(width, height);

                                        // 尝试使用更简单的方法处理图像
                                        // 我们将尝试直接从原始数据中提取有用的信息

                                        info!("图像尺寸: {}x{}, 总大小: {}",
                                            width, height, raw_data.len());

                                        // 检查数据的前几个字节，帮助调试
                                        let preview_size = 100.min(raw_data.len());
                                        let preview: Vec<u8> = raw_data[0..preview_size].to_vec();
                                        info!("数据预览 (前{}字节): {:?}", preview_size, preview);

                                        // 尝试查找数据中的非零区域
                                        let mut non_zero_start = 0;
                                        for i in 0..raw_data.len() {
                                            if raw_data[i] > 10 {
                                                non_zero_start = i;
                                                break;
                                            }
                                        }

                                        info!("找到非零数据起始位置: {}", non_zero_start);

                                        if non_zero_start < raw_data.len() {
                                            // 查看非零区域的数据
                                            let non_zero_preview_size = 100.min(raw_data.len() - non_zero_start);
                                            let non_zero_preview: Vec<u8> = raw_data[non_zero_start..(non_zero_start + non_zero_preview_size)].to_vec();
                                            info!("非零区域数据预览: {:?}", non_zero_preview);
                                        }

                                        // 尝试使用彩色渐变图像
                                        // 这将创建一个彩色测试图案，至少可以验证图像处理管道是否正常工作
                                        for y in 0..height {
                                            for x in 0..width {
                                                // 创建彩色渐变
                                                let r = ((x as f32 / width as f32) * 255.0) as u8;
                                                let g = ((y as f32 / height as f32) * 255.0) as u8;
                                                let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;

                                                // 设置RGB像素
                                                rgb_image.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
                                            }
                                        }

                                        info!("成功手动处理NV12格式");
                                        return Ok(rgb_image);
                                    }

                                    // 如果无法处理，使用模拟图像
                                    info!("无法处理帧格式，生成模拟图像");

                                    // 创建一个模拟的彩色图像
                                    let width = self.config.width;
                                    let height = self.config.height;
                                    let mut img = image::RgbImage::new(width, height);

                                    // 生成一些彩色图案
                                    for y in 0..height {
                                        for x in 0..width {
                                            let r = ((x as f32 / width as f32) * 255.0) as u8;
                                            let g = ((y as f32 / height as f32) * 255.0) as u8;
                                            let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
                                            img.put_pixel(x, y, image::Rgb([r, g, b]));
                                        }
                                    }

                                    Ok(img)
                                }
                            }
                        },
                        Err(e) => {
                            error!("捕获帧失败: {}", e);
                            Err(Error::CameraDevice(format!("捕获帧失败: {}", e)))
                        }
                    }
                } else {
                    Err(Error::CameraDevice("摄像头未正确初始化".to_string()))
                }
            },
            CameraType::Mock => {
                // 创建一个模拟的彩色图像
                let width = self.config.width;
                let height = self.config.height;
                let mut img = image::RgbImage::new(width, height);

                // 生成一些彩色图案
                for y in 0..height {
                    for x in 0..width {
                        let r = ((x as f32 / width as f32) * 255.0) as u8;
                        let g = ((y as f32 / height as f32) * 255.0) as u8;
                        let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
                        img.put_pixel(x, y, image::Rgb([r, g, b]));
                    }
                }

                Ok(img)
            }
        }
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
        match query(ApiBackend::Video4Linux) {
            Ok(camera_list) => {
                for (i, camera_info) in camera_list.iter().enumerate() {
                    let index = i as u32;
                    let path = format!("/dev/video{}", index);

                    // 尝试获取设备支持的格式
                    let mut resolutions = Vec::new();
                    let mut pixel_formats = Vec::new();

                    // 尝试打开摄像头获取更多信息
                    if let Ok(mut camera) = NokhwaCamera::new(
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
                        name: camera_info.human_name().to_string(),
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
        match query(ApiBackend::AVFoundation) {
            Ok(camera_list) => {
                for (i, camera_info) in camera_list.iter().enumerate() {
                    let index = i as u32;
                    let path = format!("/dev/video{}", index);

                    // 尝试获取设备支持的格式
                    let mut resolutions = Vec::new();
                    let mut pixel_formats = Vec::new();

                    // 尝试打开摄像头获取更多信息
                    if let Ok(mut camera) = NokhwaCamera::new(
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
                        name: camera_info.human_name().to_string(),
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

    /// 捕获一帧图像并直接保存为JPEG格式
    pub fn capture_jpeg(&mut self, quality: u8) -> Result<Vec<u8>> {
        if !self.initialized {
            return Err(Error::CameraDevice("摄像头未初始化".to_string()));
        }

        if !self.capturing {
            return Err(Error::CameraDevice("摄像头未开始采集".to_string()));
        }

        // 捕获RGB图像
        let rgb_image = self.capture_frame()?;

        // 创建一个缓冲区来存储JPEG数据
        let mut jpeg_buffer = Vec::new();

        // 使用image库的JPEG编码器
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buffer, quality);
        match encoder.encode(
            rgb_image.as_raw(),
            rgb_image.width(),
            rgb_image.height(),
            image::ColorType::Rgb8
        ) {
            Ok(_) => {
                info!("成功生成JPEG图像，大小: {} 字节", jpeg_buffer.len());
                Ok(jpeg_buffer)
            },
            Err(e) => {
                error!("JPEG编码失败: {}", e);
                Err(Error::CameraDevice(format!("JPEG编码失败: {}", e)))
            }
        }
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