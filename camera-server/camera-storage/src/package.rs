//! 文件打包功能模块

use crate::{Error, Result};
use log::{info, error, debug};
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use walkdir::WalkDir;
use zip::{ZipWriter, write::FileOptions};

/// 打包任务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageTaskStatus {
    /// 准备中
    Preparing,
    /// 打包中
    Packaging,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Canceled,
}

/// 打包任务信息
#[derive(Debug, Clone)]
pub struct PackageTaskInfo {
    /// 任务ID
    pub id: String,
    /// 源目录
    pub source_dir: PathBuf,
    /// 目标文件
    pub target_file: PathBuf,
    /// 任务状态
    pub status: PackageTaskStatus,
    /// 进度 (0.0 - 1.0)
    pub progress: f32,
    /// 总文件数
    pub total_files: usize,
    /// 已处理文件数
    pub processed_files: usize,
    /// 总大小（字节）
    pub total_size: u64,
    /// 已处理大小（字节）
    pub processed_size: u64,
    /// 创建时间（Unix时间戳）
    pub created_at: u64,
    /// 完成时间（Unix时间戳）
    pub completed_at: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
}

/// 打包管理器
pub struct PackageManager {
    /// 打包输出目录
    output_dir: PathBuf,
    /// 当前任务
    current_task: Option<PackageTaskInfo>,
}

impl PackageManager {
    /// 创建新的打包管理器
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Result<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        
        // 确保目录存在
        std::fs::create_dir_all(&output_dir)?;
        
        Ok(Self {
            output_dir,
            current_task: None,
        })
    }
    
    /// 开始打包任务
    pub fn start_package_task<P: AsRef<Path>>(&mut self, source_dir: P, name: &str) -> Result<String> {
        let source_dir = source_dir.as_ref().to_path_buf();
        
        if !source_dir.exists() || !source_dir.is_dir() {
            return Err(Error::FileSystem(format!(
                "源目录不存在: {}", source_dir.display()
            )));
        }
        
        if self.current_task.is_some() {
            return Err(Error::Packaging("已有打包任务正在进行".to_string()));
        }
        
        // 生成任务ID
        let task_id = format!("package_{}", uuid::Uuid::new_v4());
        
        // 生成目标文件路径
        let target_file = self.output_dir.join(format!("{}.zip", name));
        
        // 计算总文件数和总大小
        let mut total_files = 0;
        let mut total_size = 0;
        
        for entry in WalkDir::new(&source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            total_files += 1;
            total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
        
        // 创建任务信息
        let task_info = PackageTaskInfo {
            id: task_id.clone(),
            source_dir: source_dir.clone(),
            target_file: target_file.clone(),
            status: PackageTaskStatus::Preparing,
            progress: 0.0,
            total_files,
            processed_files: 0,
            total_size,
            processed_size: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completed_at: None,
            error: None,
        };
        
        self.current_task = Some(task_info);
        
        // 在实际应用中，这里应该启动一个异步任务来执行打包
        // 为了简化示例，这里直接执行打包
        
        match self.do_package() {
            Ok(_) => {
                if let Some(task) = &mut self.current_task {
                    task.status = PackageTaskStatus::Completed;
                    task.progress = 1.0;
                    task.completed_at = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    );
                }
            }
            Err(e) => {
                if let Some(task) = &mut self.current_task {
                    task.status = PackageTaskStatus::Failed;
                    task.error = Some(e.to_string());
                }
                return Err(e);
            }
        }
        
        Ok(task_id)
    }
    
    /// 执行打包
    fn do_package(&mut self) -> Result<()> {
        let task = match &mut self.current_task {
            Some(task) => task,
            None => return Err(Error::Packaging("没有正在进行的打包任务".to_string())),
        };
        
        task.status = PackageTaskStatus::Packaging;
        
        // 创建ZIP文件
        let file = File::create(&task.target_file)?;
        let mut zip = ZipWriter::new(file);
        
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
            
        let source_dir = &task.source_dir;
        let source_path = source_dir.as_path();
        
        // 遍历源目录中的所有文件
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let name = path.strip_prefix(source_path)
                .map_err(|e| Error::Packaging(format!("路径处理错误: {}", e)))?;
                
            if path.is_file() {
                // 更新进度
                task.processed_files += 1;
                task.progress = task.processed_files as f32 / task.total_files as f32;
                
                // 添加文件到ZIP
                zip.start_file(
                    name.to_string_lossy().into_owned(),
                    options,
                )?;
                
                let mut file = File::open(path)?;
                let file_size = file.metadata()?.len();
                
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                
                zip.write_all(&buffer)?;
                
                // 更新已处理大小
                task.processed_size += file_size;
            } else if name.as_os_str().len() != 0 {
                // 添加目录到ZIP
                zip.add_directory(
                    name.to_string_lossy().into_owned(),
                    options,
                )?;
            }
        }
        
        // 完成ZIP文件
        zip.finish()?;
        
        Ok(())
    }
    
    /// 获取当前任务状态
    pub fn get_current_task(&self) -> Option<&PackageTaskInfo> {
        self.current_task.as_ref()
    }
    
    /// 取消当前任务
    pub fn cancel_current_task(&mut self) -> Result<bool> {
        if let Some(task) = &mut self.current_task {
            if task.status == PackageTaskStatus::Packaging || task.status == PackageTaskStatus::Preparing {
                task.status = PackageTaskStatus::Canceled;
                
                // 删除未完成的ZIP文件
                if task.target_file.exists() {
                    std::fs::remove_file(&task.target_file)?;
                }
                
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// 清除当前任务
    pub fn clear_current_task(&mut self) {
        self.current_task = None;
    }
}
