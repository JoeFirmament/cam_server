#!/bin/bash

# 设置RK3588开发环境的脚本

echo "===== 开始设置RK3588开发环境 ====="

# 更新系统包
echo "正在更新系统包..."
sudo apt-get update
sudo apt-get upgrade -y

# 安装Rust
if ! command -v rustc &> /dev/null; then
    echo "正在安装Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust已安装，正在更新..."
    rustup update
fi

# 安装必要的系统依赖
echo "正在安装系统依赖..."
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libv4l-dev \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev \
    v4l-utils \
    ffmpeg

# 安装RK3588特定的开发库
echo "正在安装RK3588特定的开发库..."
sudo apt-get install -y \
    librga-dev \
    rockchip-mpp-dev

# 克隆代码仓库（如果不存在）
if [ ! -d "cam_server" ]; then
    echo "正在克隆代码仓库..."
    git clone git@github.com:JoeFirmament/cam_server.git
    cd cam_server
else
    echo "代码仓库已存在，正在更新..."
    cd cam_server
    git pull
fi

# 构建项目
echo "正在构建项目..."
cargo build --workspace

# 检查安装结果
echo "===== 环境设置完成 ====="
echo "Rust版本:"
rustc --version
echo "Cargo版本:"
cargo --version
echo "V4L2工具版本:"
v4l2-ctl --version

echo "===== 设置完成 ====="
echo "请运行 'source $HOME/.cargo/env' 或重新打开终端以使Rust环境生效"
