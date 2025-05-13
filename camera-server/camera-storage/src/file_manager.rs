//! 文件系统操作和管理模块

use crate::{Error, Result};
use log::{info, error, debug};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 文件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    /// 视频文件
    Video,
    /// 图像文件
    Image,
    /// 其他文件
    Other,
}

/// 文件信息
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 文件名
    pub name: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 文件类型
    pub file_type: FileType,
    /// 创建时间（Unix时间戳）
    pub created_at: u64,
    /// 修改时间（Unix时间戳）
    pub modified_at: u64,
}

/// 文件管理器
pub struct FileManager {
    /// 视频文件目录
    video_dir: PathBuf,
    /// 图像帧目录
    frames_dir: PathBuf,
}

impl FileManager {
    /// 创建新的文件管理器
    pub fn new<P: AsRef<Path>>(video_dir: P, frames_dir: P) -> Result<Self> {
        let video_dir = video_dir.as_ref().to_path_buf();
        let frames_dir = frames_dir.as_ref().to_path_buf();
        
        // 确保目录存在
        std::fs::create_dir_all(&video_dir)?;
        std::fs::create_dir_all(&frames_dir)?;
        
        Ok(Self {
            video_dir,
            frames_dir,
        })
    }
    
    /// 获取视频文件列表
    pub fn list_videos(&self) -> Result<Vec<FileInfo>> {
        self.list_files(&self.video_dir, FileType::Video)
    }
    
    /// 获取图像帧目录列表
    pub fn list_frame_dirs(&self) -> Result<Vec<FileInfo>> {
        // 只列出目录
        let entries = WalkDir::new(&self.frames_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != self.frames_dir && e.file_type().is_dir());
            
        let mut result = Vec::new();
        
        for entry in entries {
            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy().to_string();
            
            let metadata = entry.metadata()?;
            let size = metadata.len();
            
            let created_at = metadata.created()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);
                
            let modified_at = metadata.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);
                
            result.push(FileInfo {
                path,
                name,
                size,
                file_type: FileType::Other, // 目录类型
                created_at,
                modified_at,
            });
        }
        
        Ok(result)
    }
    
    /// 列出指定目录中的文件
    fn list_files(&self, dir: &Path, file_type: FileType) -> Result<Vec<FileInfo>> {
        let entries = WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != dir && e.file_type().is_file());
            
        let mut result = Vec::new();
        
        for entry in entries {
            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy().to_string();
            
            let metadata = entry.metadata()?;
            let size = metadata.len();
            
            let created_at = metadata.created()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);
                
            let modified_at = metadata.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);
                
            result.push(FileInfo {
                path,
                name,
                size,
                file_type: file_type.clone(),
                created_at,
                modified_at,
            });
        }
        
        Ok(result)
    }
    
    /// 删除文件
    pub fn delete_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(Error::FileSystem(format!(
                "文件不存在: {}", path.display()
            )));
        }
        
        // 检查文件是否在允许的目录中
        if !path.starts_with(&self.video_dir) && !path.starts_with(&self.frames_dir) {
            return Err(Error::FileSystem(format!(
                "不允许删除该目录中的文件: {}", path.display()
            )));
        }
        
        if path.is_file() {
            std::fs::remove_file(path)?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        }
        
        Ok(())
    }
    
    /// 重命名文件
    pub fn rename_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        
        if !from.exists() {
            return Err(Error::FileSystem(format!(
                "源文件不存在: {}", from.display()
            )));
        }
        
        // 检查文件是否在允许的目录中
        if !from.starts_with(&self.video_dir) && !from.starts_with(&self.frames_dir) {
            return Err(Error::FileSystem(format!(
                "不允许重命名该目录中的文件: {}", from.display()
            )));
        }
        
        std::fs::rename(from, to)?;
        
        Ok(())
    }
}
