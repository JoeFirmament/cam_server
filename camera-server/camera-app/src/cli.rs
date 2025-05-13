//! 命令行界面处理模块

use clap::Parser;

/// RK3588摄像头服务器
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// 配置文件路径
    #[clap(short, long)]
    pub config: Option<String>,
    
    /// 详细输出
    #[clap(short, long)]
    pub verbose: bool,
    
    /// 摄像头设备路径
    #[clap(long)]
    pub camera_device: Option<String>,
    
    /// 服务器地址
    #[clap(long)]
    pub server_address: Option<String>,
    
    /// 服务器端口
    #[clap(long)]
    pub server_port: Option<u16>,
    
    /// 静态文件目录
    #[clap(long)]
    pub static_dir: Option<String>,
    
    /// 视频文件目录
    #[clap(long)]
    pub video_dir: Option<String>,
    
    /// 帧文件目录
    #[clap(long)]
    pub frames_dir: Option<String>,
    
    /// 日志文件路径
    #[clap(long)]
    pub log_file: Option<String>,
    
    /// 日志级别
    #[clap(long)]
    pub log_level: Option<String>,
}
