//! 存储空间管理模块

use crate::{Error, Result};
use log::{info, error, debug};
use std::path::Path;

/// 磁盘空间信息
#[derive(Debug, Clone)]
pub struct DiskSpaceInfo {
    /// 总空间（字节）
    pub total: u64,
    /// 已用空间（字节）
    pub used: u64,
    /// 可用空间（字节）
    pub available: u64,
    /// 使用率（0.0 - 1.0）
    pub usage_ratio: f32,
}

/// 存储空间管理器
pub struct DiskManager;

impl DiskManager {
    /// 创建新的存储空间管理器
    pub fn new() -> Self {
        Self
    }
    
    /// 获取指定路径所在磁盘的空间信息
    pub fn get_disk_space<P: AsRef<Path>>(&self, path: P) -> Result<DiskSpaceInfo> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(Error::DiskSpace(format!(
                "路径不存在: {}", path.display()
            )));
        }
        
        // 在Linux/Unix系统上使用statvfs获取磁盘信息
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            
            let statvfs = nix::sys::statvfs::statvfs(path)
                .map_err(|e| Error::DiskSpace(format!("获取磁盘信息失败: {}", e)))?;
                
            let block_size = statvfs.block_size() as u64;
            let total = statvfs.blocks() * block_size;
            let available = statvfs.blocks_available() * block_size;
            let used = total - available;
            let usage_ratio = if total > 0 { used as f32 / total as f32 } else { 0.0 };
            
            Ok(DiskSpaceInfo {
                total,
                used,
                available,
                usage_ratio,
            })
        }
        
        // 在Windows系统上使用GetDiskFreeSpaceEx获取磁盘信息
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use std::ffi::OsStr;
            use std::iter::once;
            use winapi::um::fileapi::GetDiskFreeSpaceExW;
            use winapi::shared::minwindef::PULARGE_INTEGER;
            
            let path_str: Vec<u16> = OsStr::new(path)
                .encode_wide()
                .chain(once(0))
                .collect();
                
            let mut free_bytes_available = 0u64;
            let mut total_bytes = 0u64;
            let mut total_free_bytes = 0u64;
            
            let result = unsafe {
                GetDiskFreeSpaceExW(
                    path_str.as_ptr(),
                    &mut free_bytes_available as *mut u64 as PULARGE_INTEGER,
                    &mut total_bytes as *mut u64 as PULARGE_INTEGER,
                    &mut total_free_bytes as *mut u64 as PULARGE_INTEGER,
                )
            };
            
            if result == 0 {
                return Err(Error::DiskSpace("获取磁盘信息失败".to_string()));
            }
            
            let used = total_bytes - total_free_bytes;
            let usage_ratio = if total_bytes > 0 { used as f32 / total_bytes as f32 } else { 0.0 };
            
            Ok(DiskSpaceInfo {
                total: total_bytes,
                used,
                available: total_free_bytes,
                usage_ratio,
            })
        }
        
        // 对于其他平台，返回一个模拟的磁盘信息
        #[cfg(not(any(unix, windows)))]
        {
            // 模拟数据，实际应用中应该使用平台特定的API
            let total = 1024 * 1024 * 1024 * 100; // 100 GB
            let available = 1024 * 1024 * 1024 * 50; // 50 GB
            let used = total - available;
            let usage_ratio = used as f32 / total as f32;
            
            Ok(DiskSpaceInfo {
                total,
                used,
                available,
                usage_ratio,
            })
        }
    }
    
    /// 检查指定路径是否有足够的可用空间
    pub fn has_enough_space<P: AsRef<Path>>(&self, path: P, required_space: u64) -> Result<bool> {
        let disk_info = self.get_disk_space(path)?;
        Ok(disk_info.available >= required_space)
    }
    
    /// 获取指定目录的大小
    pub fn get_directory_size<P: AsRef<Path>>(&self, dir: P) -> Result<u64> {
        let dir = dir.as_ref();
        
        if !dir.exists() || !dir.is_dir() {
            return Err(Error::DiskSpace(format!(
                "目录不存在: {}", dir.display()
            )));
        }
        
        let mut total_size = 0;
        
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            total_size += entry.metadata()?.len();
        }
        
        Ok(total_size)
    }
}
