# 开发环境信息

## 当前开发环境

**日期**: 2025年5月13日

**操作系统**: macOS (Darwin Kernel Version 24.4.0)
**CPU架构**: ARM64 (Apple Silicon)
**Rust版本**: 1.86.0 (adf9b6ad1 2025-02-28)

## 目标部署环境

**设备**: RK3588开发板（如香橙派5 Plus）
**操作系统**: Radxa OS 或 Armbian（基于Debian/Ubuntu的RK3588优化版本）
**CPU架构**: ARM64

## 开发工具

- **编辑器/IDE**: Visual Studio Code
- **版本控制**: Git
- **构建工具**: Cargo (Rust包管理器)
- **依赖管理**: Cargo.toml (Workspace)
- **代码仓库**: GitHub (git@github.com:JoeFirmament/cam_server.git)

## 主要依赖库

- **Web框架**: Actix-web 4.3
- **摄像头接口**: 
  - Linux: nokhwa 0.10 (input-v4l feature)
  - macOS: nokhwa 0.10 (input-avfoundation feature)
- **视频处理**: ffmpeg-next 6.0 (暂时注释掉，与系统FFmpeg版本不兼容)
- **图像处理**: image 0.24
- **序列化/反序列化**: serde 1.0
- **异步运行时**: tokio 1.28
- **日志**: log 0.4, env_logger 0.10

## 开发模式

项目采用混合开发模式：
1. 在Mac上开发基础功能和跨平台代码
2. 通过GitHub同步到RK3588开发板
3. 在RK3588上开发和测试硬件相关功能
4. 最终在RK3588上部署和运行

## 交叉编译设置

```bash
# 安装交叉编译工具链
rustup target add aarch64-unknown-linux-gnu

# 配置Cargo
cat >> ~/.cargo/config << EOF
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF

# 安装必要的系统依赖
brew install aarch64-linux-gnu-binutils
```

## 测试环境

- **单元测试**: Rust内置测试框架
- **集成测试**: 自定义测试工具 (tools/camera_capture_test, tools/camera_detector)
- **性能测试**: 待定

## 注意事项

1. **平台差异**:
   - Mac使用AVFoundation访问摄像头
   - Linux(RK3588)使用V4L2访问摄像头
   - 代码需要处理平台差异，提供统一的API

2. **硬件加速**:
   - Mac上暂时使用软件编码
   - RK3588上将使用硬件编码加速
   - 需要针对不同平台优化编码参数

3. **依赖兼容性**:
   - 某些依赖可能在不同平台上有兼容性问题
   - 使用条件编译和特性标志处理平台特定代码

4. **开发流程**:
   - 在Mac上开发和测试基础功能
   - 定期同步到RK3588测试硬件相关功能
   - 使用Git分支管理不同平台的特定代码

## 环境设置脚本

### Mac开发环境设置

```bash
#!/bin/bash
# setup_mac_dev.sh

# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装必要的系统依赖
brew install ffmpeg pkg-config

# 安装交叉编译工具链
rustup target add aarch64-unknown-linux-gnu
brew install aarch64-linux-gnu-binutils

# 克隆代码仓库
git clone git@github.com:JoeFirmament/cam_server.git
cd cam_server

# 构建项目
cargo build --workspace
```

### RK3588开发环境设置

```bash
#!/bin/bash
# setup_rk3588_dev.sh

# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装必要的系统依赖
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libv4l-dev \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev

# 克隆代码仓库
git clone git@github.com:JoeFirmament/cam_server.git
cd cam_server

# 构建项目
cargo build --workspace
```

---

*注：此文档将随着项目进展不断更新。*
