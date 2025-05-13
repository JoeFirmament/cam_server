//! 应用核心逻辑模块

use anyhow::{Result, Context};
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::signal;
use crate::config::AppConfig;

use camera_core::camera::Camera;
use camera_core::video::{VideoRecorder, VideoSplitter};
use camera_storage::file_manager::FileManager;
use camera_storage::frame_manager::FrameManager;
use camera_storage::package::PackageManager;
use camera_storage::disk::DiskManager;
use camera_monitor::system::SystemMonitor;
use camera_monitor::service::ServiceMonitor;
use camera_monitor::logger::Logger;
use camera_api::server::Server;

/// 应用状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误
    Error,
}

/// 应用
pub struct App {
    /// 配置
    config: AppConfig,
    /// 应用状态
    state: AppState,
    /// 摄像头
    camera: Option<Arc<Mutex<Camera>>>,
    /// 视频录制器
    recorder: Option<Arc<Mutex<VideoRecorder>>>,
    /// 视频拆分器
    splitter: Option<Arc<Mutex<VideoSplitter>>>,
    /// 文件管理器
    file_manager: Option<Arc<Mutex<FileManager>>>,
    /// 帧管理器
    frame_manager: Option<Arc<Mutex<FrameManager>>>,
    /// 打包管理器
    package_manager: Option<Arc<Mutex<PackageManager>>>,
    /// 磁盘管理器
    disk_manager: Option<Arc<DiskManager>>,
    /// 系统监控器
    system_monitor: Option<Arc<Mutex<SystemMonitor>>>,
    /// 服务监控器
    service_monitor: Option<Arc<Mutex<ServiceMonitor>>>,
    /// 日志管理器
    logger: Option<Arc<Logger>>,
    /// API服务器
    server: Option<Server>,
}

impl App {
    /// 创建新的应用实例
    pub fn new(config: AppConfig) -> Result<Self> {
        Ok(Self {
            config,
            state: AppState::Initializing,
            camera: None,
            recorder: None,
            splitter: None,
            file_manager: None,
            frame_manager: None,
            package_manager: None,
            disk_manager: None,
            system_monitor: None,
            service_monitor: None,
            logger: None,
            server: None,
        })
    }
    
    /// 初始化应用
    async fn initialize(&mut self) -> Result<()> {
        info!("初始化应用...");
        
        // 初始化日志管理器
        let log_config = camera_monitor::logger::LoggerConfig {
            log_file: std::path::PathBuf::from(&self.config.log.log_file),
            level: match self.config.log.level.as_str() {
                "error" => camera_monitor::logger::LogLevel::Error,
                "warning" => camera_monitor::logger::LogLevel::Warning,
                "info" => camera_monitor::logger::LogLevel::Info,
                "debug" => camera_monitor::logger::LogLevel::Debug,
                "trace" => camera_monitor::logger::LogLevel::Trace,
                _ => camera_monitor::logger::LogLevel::Info,
            },
            console_output: self.config.log.console_output,
            max_file_size: self.config.log.max_file_size * 1024 * 1024,
            max_files: self.config.log.max_files,
        };
        
        let logger = Arc::new(
            camera_monitor::logger::Logger::new(log_config)
                .context("初始化日志管理器失败")?
        );
        
        self.logger = Some(logger);
        
        // 初始化磁盘管理器
        let disk_manager = Arc::new(camera_storage::disk::DiskManager::new());
        self.disk_manager = Some(disk_manager);
        
        // 初始化文件管理器
        let file_manager = Arc::new(Mutex::new(
            camera_storage::file_manager::FileManager::new(
                &self.config.storage.video_dir,
                &self.config.storage.frames_dir,
            ).context("初始化文件管理器失败")?
        ));
        
        self.file_manager = Some(file_manager);
        
        // 初始化帧管理器
        let frame_manager = Arc::new(Mutex::new(
            camera_storage::frame_manager::FrameManager::new(
                &self.config.storage.frames_dir,
            ).context("初始化帧管理器失败")?
        ));
        
        self.frame_manager = Some(frame_manager);
        
        // 初始化打包管理器
        let package_manager = Arc::new(Mutex::new(
            camera_storage::package::PackageManager::new(
                &self.config.storage.package_dir,
            ).context("初始化打包管理器失败")?
        ));
        
        self.package_manager = Some(package_manager);
        
        // 初始化系统监控器
        let system_monitor = Arc::new(Mutex::new(
            camera_monitor::system::SystemMonitor::new()
        ));
        
        self.system_monitor = Some(system_monitor);
        
        // 初始化服务监控器
        let service_monitor = Arc::new(Mutex::new(
            camera_monitor::service::ServiceMonitor::new()
        ));
        
        self.service_monitor = Some(service_monitor.clone());
        
        // 注册服务
        {
            let mut monitor = service_monitor.lock().await;
            monitor.register_service("camera").context("注册摄像头服务失败")?;
            monitor.register_service("recorder").context("注册录制器服务失败")?;
            monitor.register_service("splitter").context("注册拆分器服务失败")?;
            monitor.register_service("api").context("注册API服务失败")?;
        }
        
        // 初始化摄像头
        let camera = Arc::new(Mutex::new(
            camera_core::camera::Camera::new(self.config.camera.clone())
        ));
        
        self.camera = Some(camera.clone());
        
        // 初始化视频录制器
        let recorder = Arc::new(Mutex::new(
            camera_core::video::VideoRecorder::new(self.config.recording.clone())
        ));
        
        self.recorder = Some(recorder.clone());
        
        // 初始化视频拆分器
        let splitter = Arc::new(Mutex::new(
            camera_core::video::VideoSplitter::new(self.config.split.clone())
        ));
        
        self.splitter = Some(splitter.clone());
        
        // 初始化API服务器
        // 注意：这里只是示例，实际实现需要根据camera-api模块的具体接口
        /*
        let server = camera_api::server::Server::new(
            &self.config.server.address,
            self.config.server.port,
            camera.clone(),
            recorder.clone(),
            splitter.clone(),
            file_manager.clone(),
            frame_manager.clone(),
            package_manager.clone(),
            system_monitor.clone(),
            service_monitor.clone(),
        ).await.context("初始化API服务器失败")?;
        
        self.server = Some(server);
        */
        
        info!("应用初始化完成");
        self.state = AppState::Running;
        
        Ok(())
    }
    
    /// 运行应用
    pub async fn run(&mut self) -> Result<()> {
        // 初始化应用
        self.initialize().await?;
        
        info!("应用开始运行");
        
        // 启动API服务器
        /*
        if let Some(server) = &self.server {
            server.start().await.context("启动API服务器失败")?;
        }
        */
        
        // 等待终止信号
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("接收到终止信号，开始关闭应用...");
                self.shutdown().await?;
            }
            Err(e) => {
                error!("无法监听终止信号: {}", e);
                self.shutdown().await?;
            }
        }
        
        Ok(())
    }
    
    /// 关闭应用
    async fn shutdown(&mut self) -> Result<()> {
        info!("关闭应用...");
        self.state = AppState::Stopping;
        
        // 停止API服务器
        /*
        if let Some(server) = &self.server {
            server.stop().await.context("停止API服务器失败")?;
        }
        */
        
        // 停止摄像头
        if let Some(camera) = &self.camera {
            let mut camera = camera.lock().await;
            if camera.is_capturing() {
                camera.stop_capture().context("停止摄像头失败")?;
            }
        }
        
        // 停止录制
        if let Some(recorder) = &self.recorder {
            let mut recorder = recorder.lock().await;
            if recorder.is_recording() {
                recorder.stop_recording().context("停止录制失败")?;
            }
        }
        
        info!("应用已关闭");
        self.state = AppState::Stopped;
        
        Ok(())
    }
}
