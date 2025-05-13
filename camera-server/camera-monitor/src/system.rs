//! 系统资源监控模块

use crate::{Error, Result};
use log::{info, error, debug};
use sysinfo::{System, SystemExt, ProcessorExt, DiskExt, NetworkExt};
use std::time::{Duration, Instant};

/// CPU信息
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// CPU使用率（0.0 - 100.0）
    pub usage: f32,
    /// CPU温度（摄氏度）
    pub temperature: Option<f32>,
    /// CPU频率（MHz）
    pub frequency: u64,
    /// CPU核心数
    pub cores: usize,
}

/// 内存信息
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// 总内存（字节）
    pub total: u64,
    /// 已用内存（字节）
    pub used: u64,
    /// 可用内存（字节）
    pub available: u64,
    /// 使用率（0.0 - 1.0）
    pub usage_ratio: f32,
}

/// 磁盘信息
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// 名称
    pub name: String,
    /// 挂载点
    pub mount_point: String,
    /// 总空间（字节）
    pub total: u64,
    /// 已用空间（字节）
    pub used: u64,
    /// 可用空间（字节）
    pub available: u64,
    /// 使用率（0.0 - 1.0）
    pub usage_ratio: f32,
}

/// 网络信息
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// 接口名称
    pub name: String,
    /// 接收字节数
    pub received_bytes: u64,
    /// 发送字节数
    pub transmitted_bytes: u64,
    /// 接收速率（字节/秒）
    pub receive_rate: u64,
    /// 发送速率（字节/秒）
    pub transmit_rate: u64,
}

/// 系统信息
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// 主机名
    pub hostname: String,
    /// 操作系统名称
    pub os_name: String,
    /// 操作系统版本
    pub os_version: String,
    /// 内核版本
    pub kernel_version: String,
    /// 系统启动时间（秒）
    pub uptime: u64,
    /// CPU信息
    pub cpu: CpuInfo,
    /// 内存信息
    pub memory: MemoryInfo,
    /// 磁盘信息列表
    pub disks: Vec<DiskInfo>,
    /// 网络信息列表
    pub networks: Vec<NetworkInfo>,
}

/// 系统监控器
pub struct SystemMonitor {
    /// sysinfo系统对象
    system: System,
    /// 上次更新时间
    last_update: Instant,
    /// 上次网络接收字节数
    last_received_bytes: Vec<(String, u64)>,
    /// 上次网络发送字节数
    last_transmitted_bytes: Vec<(String, u64)>,
}

impl SystemMonitor {
    /// 创建新的系统监控器
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let last_update = Instant::now();
        
        // 初始化网络流量计数
        let mut last_received_bytes = Vec::new();
        let mut last_transmitted_bytes = Vec::new();
        
        for (interface_name, network) in system.networks() {
            last_received_bytes.push((interface_name.clone(), network.received()));
            last_transmitted_bytes.push((interface_name.clone(), network.transmitted()));
        }
        
        Self {
            system,
            last_update,
            last_received_bytes,
            last_transmitted_bytes,
        }
    }
    
    /// 获取系统信息
    pub fn get_system_info(&mut self) -> Result<SystemInfo> {
        // 刷新系统信息
        self.system.refresh_all();
        
        // 计算时间差
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;
        
        // 获取CPU信息
        let cpu_usage = self.system.global_processor_info().cpu_usage();
        let cpu_info = CpuInfo {
            usage: cpu_usage,
            temperature: None, // sysinfo不直接提供CPU温度
            frequency: self.system.global_processor_info().frequency(),
            cores: self.system.processors().len(),
        };
        
        // 获取内存信息
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let available_memory = total_memory - used_memory;
        let memory_usage_ratio = if total_memory > 0 {
            used_memory as f32 / total_memory as f32
        } else {
            0.0
        };
        
        let memory_info = MemoryInfo {
            total: total_memory * 1024, // sysinfo返回的是KB
            used: used_memory * 1024,
            available: available_memory * 1024,
            usage_ratio: memory_usage_ratio,
        };
        
        // 获取磁盘信息
        let mut disks = Vec::new();
        for disk in self.system.disks() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let usage_ratio = if total > 0 { used as f32 / total as f32 } else { 0.0 };
            
            disks.push(DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total,
                used,
                available,
                usage_ratio,
            });
        }
        
        // 获取网络信息
        let mut networks = Vec::new();
        for (interface_name, network) in self.system.networks() {
            let received = network.received();
            let transmitted = network.transmitted();
            
            // 查找上次的计数
            let last_received = self.last_received_bytes.iter()
                .find(|(name, _)| name == interface_name)
                .map(|(_, bytes)| *bytes)
                .unwrap_or(0);
                
            let last_transmitted = self.last_transmitted_bytes.iter()
                .find(|(name, _)| name == interface_name)
                .map(|(_, bytes)| *bytes)
                .unwrap_or(0);
                
            // 计算速率
            let receive_rate = if elapsed > 0.0 {
                ((received - last_received) as f64 / elapsed) as u64
            } else {
                0
            };
            
            let transmit_rate = if elapsed > 0.0 {
                ((transmitted - last_transmitted) as f64 / elapsed) as u64
            } else {
                0
            };
            
            // 更新计数
            if let Some(item) = self.last_received_bytes.iter_mut()
                .find(|(name, _)| name == interface_name)
            {
                item.1 = received;
            } else {
                self.last_received_bytes.push((interface_name.clone(), received));
            }
            
            if let Some(item) = self.last_transmitted_bytes.iter_mut()
                .find(|(name, _)| name == interface_name)
            {
                item.1 = transmitted;
            } else {
                self.last_transmitted_bytes.push((interface_name.clone(), transmitted));
            }
            
            networks.push(NetworkInfo {
                name: interface_name.clone(),
                received_bytes: received,
                transmitted_bytes: transmitted,
                receive_rate,
                transmit_rate,
            });
        }
        
        // 构建系统信息
        let system_info = SystemInfo {
            hostname: self.system.host_name().unwrap_or_else(|| "Unknown".to_string()),
            os_name: self.system.name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: self.system.os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: self.system.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            uptime: self.system.uptime(),
            cpu: cpu_info,
            memory: memory_info,
            disks,
            networks,
        };
        
        Ok(system_info)
    }
}
