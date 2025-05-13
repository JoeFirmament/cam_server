//! 静态帧文件夹管理模块

use crate::{Error, Result};
use log::{info, error, debug};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 帧文件夹信息
#[derive(Debug, Clone)]
pub struct FrameDirInfo {
    /// 文件夹路径
    pub path: PathBuf,
    /// 文件夹名称
    pub name: String,
    /// 帧数量
    pub frame_count: usize,
    /// 总大小（字节）
    pub total_size: u64,
    /// 创建时间（Unix时间戳）
    pub created_at: u64,
}

/// 帧文件信息
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 文件名
    pub name: String,
    /// 帧序号
    pub frame_number: usize,
    /// 文件大小（字节）
    pub size: u64,
}

/// 帧管理器
pub struct FrameManager {
    /// 帧文件夹根目录
    frames_root_dir: PathBuf,
}

impl FrameManager {
    /// 创建新的帧管理器
    pub fn new<P: AsRef<Path>>(frames_root_dir: P) -> Result<Self> {
        let frames_root_dir = frames_root_dir.as_ref().to_path_buf();
        
        // 确保目录存在
        std::fs::create_dir_all(&frames_root_dir)?;
        
        Ok(Self {
            frames_root_dir,
        })
    }
    
    /// 获取所有帧文件夹信息
    pub fn list_frame_dirs(&self) -> Result<Vec<FrameDirInfo>> {
        let entries = WalkDir::new(&self.frames_root_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != self.frames_root_dir && e.file_type().is_dir());
            
        let mut result = Vec::new();
        
        for entry in entries {
            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy().to_string();
            
            let metadata = entry.metadata()?;
            
            let created_at = metadata.created()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);
                
            // 计算帧数量和总大小
            let mut frame_count = 0;
            let mut total_size = 0;
            
            for file_entry in WalkDir::new(&path)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path() != path && e.file_type().is_file())
            {
                frame_count += 1;
                total_size += file_entry.metadata()?.len();
            }
            
            result.push(FrameDirInfo {
                path,
                name,
                frame_count,
                total_size,
                created_at,
            });
        }
        
        Ok(result)
    }
    
    /// 获取指定帧文件夹中的所有帧
    pub fn list_frames<P: AsRef<Path>>(&self, frame_dir: P) -> Result<Vec<FrameInfo>> {
        let frame_dir = frame_dir.as_ref();
        
        if !frame_dir.exists() || !frame_dir.is_dir() {
            return Err(Error::FileSystem(format!(
                "帧文件夹不存在: {}", frame_dir.display()
            )));
        }
        
        let entries = WalkDir::new(frame_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != frame_dir && e.file_type().is_file());
            
        let mut result = Vec::new();
        
        for entry in entries {
            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // 尝试从文件名中提取帧序号
            let frame_number = name
                .split('_')
                .last()
                .and_then(|s| s.split('.').next())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
                
            let size = entry.metadata()?.len();
            
            result.push(FrameInfo {
                path,
                name,
                frame_number,
                size,
            });
        }
        
        // 按帧序号排序
        result.sort_by_key(|f| f.frame_number);
        
        Ok(result)
    }
    
    /// 创建新的帧文件夹
    pub fn create_frame_dir(&self, name: &str) -> Result<PathBuf> {
        let dir_path = self.frames_root_dir.join(name);
        
        if dir_path.exists() {
            return Err(Error::FileSystem(format!(
                "帧文件夹已存在: {}", dir_path.display()
            )));
        }
        
        std::fs::create_dir_all(&dir_path)?;
        
        Ok(dir_path)
    }
    
    /// 删除帧文件夹
    pub fn delete_frame_dir<P: AsRef<Path>>(&self, frame_dir: P) -> Result<()> {
        let frame_dir = frame_dir.as_ref();
        
        if !frame_dir.exists() || !frame_dir.is_dir() {
            return Err(Error::FileSystem(format!(
                "帧文件夹不存在: {}", frame_dir.display()
            )));
        }
        
        // 检查是否在允许的目录中
        if !frame_dir.starts_with(&self.frames_root_dir) {
            return Err(Error::FileSystem(format!(
                "不允许删除该目录: {}", frame_dir.display()
            )));
        }
        
        std::fs::remove_dir_all(frame_dir)?;
        
        Ok(())
    }
}
