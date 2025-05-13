//! 服务状态监控模块

use crate::{Error, Result};
use log::{info, error, debug};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 服务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 启动中
    Starting,
    /// 停止中
    Stopping,
    /// 错误
    Error,
    /// 未知
    Unknown,
}

/// 服务健康状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 降级
    Degraded,
    /// 未知
    Unknown,
}

/// 服务信息
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// 服务名称
    pub name: String,
    /// 服务状态
    pub status: ServiceStatus,
    /// 健康状态
    pub health: HealthStatus,
    /// 启动时间（Unix时间戳）
    pub start_time: Option<u64>,
    /// 运行时间（秒）
    pub uptime: Option<u64>,
    /// 最后检查时间（Unix时间戳）
    pub last_check: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 额外信息
    pub extra: HashMap<String, String>,
}

/// 服务监控器
pub struct ServiceMonitor {
    /// 服务列表
    services: HashMap<String, ServiceInfo>,
    /// 上次检查时间
    last_check: Instant,
}

impl ServiceMonitor {
    /// 创建新的服务监控器
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            last_check: Instant::now(),
        }
    }
    
    /// 注册服务
    pub fn register_service(&mut self, name: &str) -> Result<()> {
        if self.services.contains_key(name) {
            return Err(Error::Service(format!("服务已存在: {}", name)));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let service_info = ServiceInfo {
            name: name.to_string(),
            status: ServiceStatus::Unknown,
            health: HealthStatus::Unknown,
            start_time: None,
            uptime: None,
            last_check: now,
            error: None,
            extra: HashMap::new(),
        };
        
        self.services.insert(name.to_string(), service_info);
        
        Ok(())
    }
    
    /// 更新服务状态
    pub fn update_service_status(
        &mut self,
        name: &str,
        status: ServiceStatus,
        health: HealthStatus,
    ) -> Result<()> {
        let service = self.services.get_mut(name)
            .ok_or_else(|| Error::Service(format!("服务不存在: {}", name)))?;
            
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // 如果服务从非运行状态变为运行状态，更新启动时间
        if service.status != ServiceStatus::Running && status == ServiceStatus::Running {
            service.start_time = Some(now);
            service.uptime = Some(0);
        }
        
        // 如果服务从运行状态变为非运行状态，清除启动时间和运行时间
        if service.status == ServiceStatus::Running && status != ServiceStatus::Running {
            service.start_time = None;
            service.uptime = None;
        }
        
        // 如果服务保持运行状态，更新运行时间
        if service.status == ServiceStatus::Running && status == ServiceStatus::Running {
            if let Some(start_time) = service.start_time {
                service.uptime = Some(now - start_time);
            }
        }
        
        service.status = status;
        service.health = health;
        service.last_check = now;
        service.error = None; // 清除错误信息
        
        Ok(())
    }
    
    /// 设置服务错误
    pub fn set_service_error(&mut self, name: &str, error: &str) -> Result<()> {
        let service = self.services.get_mut(name)
            .ok_or_else(|| Error::Service(format!("服务不存在: {}", name)))?;
            
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        service.status = ServiceStatus::Error;
        service.health = HealthStatus::Unhealthy;
        service.last_check = now;
        service.error = Some(error.to_string());
        
        Ok(())
    }
    
    /// 设置服务额外信息
    pub fn set_service_extra(&mut self, name: &str, key: &str, value: &str) -> Result<()> {
        let service = self.services.get_mut(name)
            .ok_or_else(|| Error::Service(format!("服务不存在: {}", name)))?;
            
        service.extra.insert(key.to_string(), value.to_string());
        
        Ok(())
    }
    
    /// 获取服务信息
    pub fn get_service(&self, name: &str) -> Option<&ServiceInfo> {
        self.services.get(name)
    }
    
    /// 获取所有服务信息
    pub fn get_all_services(&self) -> Vec<&ServiceInfo> {
        self.services.values().collect()
    }
    
    /// 检查服务健康状态
    pub fn check_services_health(&self) -> (usize, usize, usize, usize) {
        let mut healthy = 0;
        let mut unhealthy = 0;
        let mut degraded = 0;
        let mut unknown = 0;
        
        for service in self.services.values() {
            match service.health {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Unhealthy => unhealthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::Unknown => unknown += 1,
            }
        }
        
        (healthy, unhealthy, degraded, unknown)
    }
    
    /// 移除服务
    pub fn remove_service(&mut self, name: &str) -> Result<()> {
        if !self.services.contains_key(name) {
            return Err(Error::Service(format!("服务不存在: {}", name)));
        }
        
        self.services.remove(name);
        
        Ok(())
    }
}
