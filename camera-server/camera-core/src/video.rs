//! 视频处理模块

use crate::{Error, Result, config::{RecordingConfig, SplitConfig}};
use log::{info, error};
use std::path::{Path, PathBuf};

/// 视频录制器
pub struct VideoRecorder {
    /// 录制配置
    config: RecordingConfig,

    /// 是否正在录制
    recording: bool,

    /// 当前录制文件路径
    current_file: Option<PathBuf>,

    // 这里将来会添加 FFmpeg 相关的字段
}

impl VideoRecorder {
    /// 创建新的视频录制器
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            config,
            recording: false,
            current_file: None,
        }
    }

    /// 开始录制
    pub fn start_recording(&mut self) -> Result<PathBuf> {
        if self.recording {
            if let Some(path) = &self.current_file {
                return Ok(path.clone());
            }
        }

        // 创建输出目录
        std::fs::create_dir_all(&self.config.output_dir)?;

        // 生成输出文件名
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("video_{}.{}", timestamp, self.config.container);
        let output_path = Path::new(&self.config.output_dir).join(filename);

        // 这里将来会添加 FFmpeg 初始化和开始录制的代码

        info!("开始录制视频: {}", output_path.display());
        self.recording = true;
        self.current_file = Some(output_path.clone());

        Ok(output_path)
    }

    /// 停止录制
    pub fn stop_recording(&mut self) -> Result<Option<PathBuf>> {
        if !self.recording {
            return Ok(None);
        }

        // 这里将来会添加停止录制的代码

        info!("停止录制视频");
        let result = self.current_file.clone();
        self.recording = false;

        Ok(result)
    }

    /// 获取是否正在录制
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// 获取当前录制文件路径
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    /// 获取录制配置
    pub fn config(&self) -> &RecordingConfig {
        &self.config
    }

    /// 设置录制配置
    pub fn set_config(&mut self, config: RecordingConfig) -> Result<()> {
        if self.recording {
            return Err(Error::VideoProcessing(
                "无法在录制过程中更改配置，请先停止录制".to_string()
            ));
        }

        self.config = config;
        Ok(())
    }
}

impl Drop for VideoRecorder {
    fn drop(&mut self) {
        if self.recording {
            if let Err(e) = self.stop_recording() {
                error!("关闭录制器时出错: {}", e);
            }
        }
    }
}

/// 视频拆分器
pub struct VideoSplitter {
    /// 拆分配置
    config: SplitConfig,

    /// 是否正在拆分
    splitting: bool,

    /// 当前拆分任务ID
    current_task_id: Option<String>,

    /// 当前拆分进度 (0.0 - 1.0)
    progress: f32,
}

impl VideoSplitter {
    /// 创建新的视频拆分器
    pub fn new(config: SplitConfig) -> Self {
        Self {
            config,
            splitting: false,
            current_task_id: None,
            progress: 0.0,
        }
    }

    /// 开始拆分视频
    pub fn start_splitting(&mut self, video_path: &Path) -> Result<String> {
        if self.splitting {
            return Err(Error::VideoProcessing(
                "已有拆分任务正在进行".to_string()
            ));
        }

        // 检查视频文件是否存在
        if !video_path.exists() {
            return Err(Error::VideoProcessing(format!(
                "视频文件不存在: {}", video_path.display()
            )));
        }

        // 创建输出目录
        let video_name = video_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let output_dir = Path::new(&self.config.output_dir)
            .join(format!("frames_{}", video_name));

        std::fs::create_dir_all(&output_dir)?;

        // 生成任务ID
        let task_id = format!("split_{}", uuid::Uuid::new_v4());

        // 这里将来会添加异步拆分任务的代码

        info!("开始拆分视频: {} -> {}", video_path.display(), output_dir.display());
        self.splitting = true;
        self.current_task_id = Some(task_id.clone());
        self.progress = 0.0;

        Ok(task_id)
    }

    /// 获取拆分任务状态
    pub fn get_task_status(&self, task_id: &str) -> Option<(bool, f32)> {
        if let Some(current_id) = &self.current_task_id {
            if current_id == task_id {
                return Some((self.splitting, self.progress));
            }
        }

        None
    }

    /// 取消拆分任务
    pub fn cancel_task(&mut self, task_id: &str) -> Result<bool> {
        if let Some(current_id) = &self.current_task_id {
            if current_id == task_id && self.splitting {
                // 这里将来会添加取消任务的代码

                info!("取消拆分任务: {}", task_id);
                self.splitting = false;
                self.progress = 0.0;
                self.current_task_id = None;

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 获取拆分配置
    pub fn config(&self) -> &SplitConfig {
        &self.config
    }

    /// 设置拆分配置
    pub fn set_config(&mut self, config: SplitConfig) -> Result<()> {
        if self.splitting {
            return Err(Error::VideoProcessing(
                "无法在拆分过程中更改配置，请先取消拆分任务".to_string()
            ));
        }

        self.config = config;
        Ok(())
    }
}