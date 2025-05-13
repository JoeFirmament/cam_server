//! 摄像头服务器主应用

mod config;
mod app;
mod cli;

use anyhow::{Result, Context};
use clap::Parser;
use log::{info, error, debug};
use cli::Cli;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();
    
    // 初始化日志系统
    init_logger(cli.verbose)?;
    
    // 加载配置
    let config_path = cli.config.unwrap_or_else(|| "config.toml".to_string());
    let config = config::load_config(&config_path)
        .context(format!("加载配置文件失败: {}", config_path))?;
        
    info!("配置加载成功: {}", config_path);
    
    // 创建应用实例
    let mut app = App::new(config)
        .context("创建应用实例失败")?;
        
    // 运行应用
    info!("摄像头服务器启动中...");
    app.run().await?;
    
    Ok(())
}

/// 初始化日志系统
fn init_logger(verbose: bool) -> Result<()> {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
        
    Ok(())
}
