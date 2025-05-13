use anyhow::Result;
use camera_core::camera::Camera;
use log::{info, error};

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    info!("开始检测摄像头设备...");
    
    // 列出所有摄像头设备
    match Camera::list_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("未找到摄像头设备");
            } else {
                println!("找到 {} 个摄像头设备:", devices.len());
                
                for (i, device) in devices.iter().enumerate() {
                    println!("设备 #{}:", i + 1);
                    println!("  路径: {}", device.path);
                    println!("  名称: {}", device.name);
                    println!("  驱动: {}", device.driver);
                    
                    println!("  支持的分辨率:");
                    for res in &device.resolutions {
                        println!("    {}x{}", res.0, res.1);
                    }
                    
                    println!("  支持的像素格式:");
                    for format in &device.pixel_formats {
                        println!("    {}", format);
                    }
                    
                    println!();
                }
            }
        },
        Err(e) => {
            error!("检测摄像头设备失败: {}", e);
            println!("检测摄像头设备失败: {}", e);
        }
    }
    
    Ok(())
}
