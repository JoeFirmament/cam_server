use anyhow::Result;
use log::info;
use std::fs;
use std::path::Path;
use camera_core::camera::Camera;
use camera_core::config::CameraConfig;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("JPEG捕获测试工具");

    // 列出所有摄像头设备
    let devices = Camera::list_devices()?;
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
    let output_dir = "jpeg_test_output";
    fs::create_dir_all(output_dir)?;

    // 创建摄像头配置
    let config = CameraConfig {
        device_path: device.path.clone(),
        width: 1920,
        height: 1080,
        fps: 30,
        pixel_format: "YUYV".to_string(), // 默认像素格式
    };

    // 创建摄像头实例
    let mut camera = Camera::new(config);

    // 初始化摄像头
    println!("初始化摄像头...");
    camera.initialize()?;

    // 开始视频采集
    println!("开始视频采集...");
    camera.start_capture()?;

    // 捕获10帧JPEG图像，使用不同的质量设置
    println!("捕获10帧JPEG图像...");
    for i in 1..=10 {
        let quality = (i * 10).min(100) as u8; // 质量从10到100
        println!("捕获第 {} 帧，JPEG质量: {}...", i, quality);

        // 捕获JPEG图像
        let jpeg_data = camera.capture_jpeg(quality)?;

        // 保存JPEG图像
        let output_path = Path::new(output_dir).join(format!("frame_{}_q{}.jpg", i, quality));
        fs::write(&output_path, &jpeg_data)?;
        println!("保存图像到: {}", output_path.display());

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // 停止视频采集
    println!("停止视频采集...");
    camera.stop_capture()?;

    println!("测试完成，图像已保存到 {} 目录", output_dir);
    Ok(())
}
