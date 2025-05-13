use anyhow::Result;
use camera_core::camera::Camera;
use camera_core::config::{CameraConfig, RecordingConfig};
use camera_core::video::VideoRecorder;
use log::{info, error};
use std::path::Path;
use std::time::Duration;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    info!("视频录制测试工具");

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
            let output_dir = "video_test_output";
            std::fs::create_dir_all(output_dir)?;

            // 创建摄像头配置
            let config = CameraConfig {
                device_path: device.path.clone(),
                width: 640,
                height: 480,
                fps: 30,
                pixel_format: "NV12".to_string(), // 使用NV12格式，这是Mac摄像头的原生格式
            };

            // 创建录制配置
            let recording_config = RecordingConfig {
                output_dir: output_dir.to_string(),
                encoder: "h264".to_string(),
                container: "mp4".to_string(),
                bitrate: 2_000_000, // 2 Mbps
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

            // 创建视频录制器
            let mut recorder = VideoRecorder::new(recording_config);

            // 开始录制
            println!("开始录制视频...");
            let video_path = match recorder.start_recording() {
                Ok(path) => {
                    println!("录制视频到: {}", path.display());
                    path
                },
                Err(e) => {
                    error!("开始录制失败: {}", e);
                    println!("开始录制失败: {}", e);
                    return Ok(());
                }
            };

            // 录制5秒视频
            println!("录制5秒视频...");
            for i in 1..=5 {
                println!("录制中... {} 秒", i);
                
                // 捕获帧并添加到视频
                match camera.capture_frame() {
                    Ok(frame) => {
                        // 这里应该将帧添加到视频中
                        // 但由于我们还没有实现这个功能，所以只是模拟
                        println!("捕获帧 {} ({}x{})", i, frame.width(), frame.height());
                    },
                    Err(e) => {
                        error!("捕获帧失败: {}", e);
                        println!("捕获帧失败: {}", e);
                    }
                }

                // 等待一秒
                std::thread::sleep(Duration::from_secs(1));
            }

            // 停止录制
            println!("停止录制视频...");
            match recorder.stop_recording() {
                Ok(Some(path)) => println!("视频已保存到: {}", path.display()),
                Ok(None) => println!("没有录制的视频"),
                Err(e) => {
                    error!("停止录制失败: {}", e);
                    println!("停止录制失败: {}", e);
                }
            }

            // 停止视频采集
            println!("停止视频采集...");
            if let Err(e) = camera.stop_capture() {
                error!("停止视频采集失败: {}", e);
                println!("停止视频采集失败: {}", e);
            }

            println!("测试完成，视频已保存到 {} 目录", output_dir);
        },
        Err(e) => {
            error!("列出摄像头设备失败: {}", e);
            println!("列出摄像头设备失败: {}", e);
        }
    }

    Ok(())
}
