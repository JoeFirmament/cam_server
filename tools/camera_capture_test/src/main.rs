use anyhow::Result;
use camera_core::camera::Camera;
use camera_core::config::CameraConfig;
use log::{info, error};
use std::path::Path;

fn main() -> Result<()> {
    // 初始化日志，设置为最详细的级别
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .init();

    info!("摄像头捕获测试工具");

    // 列出所有摄像头设备
    match Camera::list_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("未找到摄像头设备");
                return Ok(());
            }

            println!("找到 {} 个摄像头设备:", devices.len());

            for (i, device) in devices.iter().enumerate() {
                println!("设备 #{}:", i + 1);
                println!("  路径: {}", device.path);
                println!("  名称: {}", device.name);
                println!("  驱动: {}", device.driver);
                println!();
            }

            // 使用第一个设备进行测试
            let device = &devices[0];
            println!("使用设备 '{}' 进行测试", device.name);

            // 创建输出目录
            let output_dir = "camera_test_output";
            std::fs::create_dir_all(output_dir)?;

            // 创建摄像头配置
            let config = CameraConfig {
                device_path: device.path.clone(),
                width: 640,
                height: 480,
                fps: 30,
                pixel_format: "NV12".to_string(), // 使用NV12格式，这是Mac摄像头的原生格式
            };

            // 创建摄像头实例
            let mut camera = Camera::new(config);

            // 初始化摄像头
            println!("初始化摄像头...");
            if let Err(e) = camera.initialize() {
                error!("初始化摄像头失败: {}", e);
                println!("初始化摄像头失败: {}", e);
                return Ok(());
            }

            // 开始视频采集
            println!("开始视频采集...");
            if let Err(e) = camera.start_capture() {
                error!("开始视频采集失败: {}", e);
                println!("开始视频采集失败: {}", e);
                return Ok(());
            }

            // 捕获10帧图像
            println!("捕获10帧图像...");
            for i in 1..=10 {
                println!("捕获第 {} 帧...", i);

                match camera.capture_frame() {
                    Ok(frame) => {
                        // 保存图像
                        let output_path = Path::new(output_dir).join(format!("frame_{}.png", i));
                        println!("保存图像到: {}", output_path.display());
                        frame.save(&output_path)?;
                    },
                    Err(e) => {
                        error!("捕获帧失败: {}", e);
                        println!("捕获帧失败: {}", e);
                    }
                }

                // 等待一小段时间
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // 停止视频采集
            println!("停止视频采集...");
            if let Err(e) = camera.stop_capture() {
                error!("停止视频采集失败: {}", e);
                println!("停止视频采集失败: {}", e);
            }

            println!("测试完成，图像已保存到 {} 目录", output_dir);
        },
        Err(e) => {
            error!("列出摄像头设备失败: {}", e);
            println!("列出摄像头设备失败: {}", e);
        }
    }

    Ok(())
}
