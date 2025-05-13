/**
 * 应用主逻辑
 */

/**
 * 应用管理器
 */
class AppManager {
    constructor() {
        // 应用状态
        this.state = {
            camera: {
                connected: false,
                device: '',
                resolution: '',
                fps: 0
            },
            recording: {
                active: false,
                format: '',
                bitrate: 0,
                currentFile: null
            },
            splitting: {
                active: false,
                taskId: null,
                progress: 0
            }
        };
        
        // 初始化应用
        this.init();
    }
    
    /**
     * 初始化应用
     */
    async init() {
        // 设置UI事件处理器
        this.setupUiHandlers();
        
        // 加载初始数据
        await this.loadInitialData();
        
        // 启动定时更新
        this.startPeriodicUpdates();
    }
    
    /**
     * 设置UI事件处理器
     */
    setupUiHandlers() {
        // 摄像头控制
        ui.onCameraConnect = () => this.connectCamera();
        ui.onCameraDisconnect = () => this.disconnectCamera();
        ui.onCameraApplySettings = () => this.applyCameraSettings();
        
        // 录制控制
        ui.onRecordingStart = () => this.startRecording();
        ui.onRecordingStop = () => this.stopRecording();
        ui.onRecordingApplySettings = () => this.applyRecordingSettings();
        
        // 拆分控制
        ui.onSplitStart = () => this.startSplit();
        ui.onSplitCancel = () => this.cancelSplit();
        ui.onSplitApplySettings = () => this.applySplitSettings();
        ui.onSplitFile = (path) => this.splitFile(path);
        ui.onDeleteRecordingFile = (path) => this.deleteRecordingFile(path);
        ui.onPackageSplitFolder = (path) => this.packageSplitFolder(path);
        ui.onDeleteSplitFolder = (path) => this.deleteSplitFolder(path);
    }
    
    /**
     * 加载初始数据
     */
    async loadInitialData() {
        try {
            // 获取摄像头状态
            await this.updateCameraStatus();
            
            // 获取录制状态
            await this.updateRecordingStatus();
            
            // 获取系统信息
            await this.updateSystemInfo();
            
            // 获取录制文件列表
            await this.updateRecordingFiles();
            
            // 获取拆分文件夹列表
            await this.updateSplitFolders();
        } catch (error) {
            console.error('加载初始数据失败:', error);
            this.showError('无法连接到服务器，请检查网络连接或服务器状态。');
        }
    }
    
    /**
     * 启动定时更新
     */
    startPeriodicUpdates() {
        // 每5秒更新一次状态
        setInterval(() => this.updateCameraStatus(), 5000);
        setInterval(() => this.updateRecordingStatus(), 5000);
        
        // 每10秒更新一次系统信息
        setInterval(() => this.updateSystemInfo(), 10000);
        
        // 每30秒更新一次文件列表
        setInterval(() => this.updateRecordingFiles(), 30000);
        setInterval(() => this.updateSplitFolders(), 30000);
        
        // 如果有拆分任务，每秒更新一次进度
        setInterval(() => {
            if (this.state.splitting.active && this.state.splitting.taskId) {
                this.updateSplitProgress(this.state.splitting.taskId);
            }
        }, 1000);
    }
    
    /**
     * 更新摄像头状态
     */
    async updateCameraStatus() {
        try {
            const status = await api.getCameraStatus();
            
            this.state.camera.connected = status.connected;
            
            if (status.connected) {
                this.state.camera.device = status.device;
                this.state.camera.resolution = `${status.width}x${status.height}`;
                this.state.camera.fps = status.fps;
                
                ui.updateCameraStatus('已连接', true);
            } else {
                ui.updateCameraStatus('未连接', false);
            }
        } catch (error) {
            console.error('获取摄像头状态失败:', error);
            ui.updateCameraStatus('状态未知', false);
        }
    }
    
    /**
     * 更新录制状态
     */
    async updateRecordingStatus() {
        try {
            const status = await api.getRecordingStatus();
            
            this.state.recording.active = status.recording;
            
            if (status.recording) {
                this.state.recording.currentFile = status.current_file;
                ui.updateRecordingStatus('录制中', true);
            } else {
                ui.updateRecordingStatus('未录制', false);
            }
        } catch (error) {
            console.error('获取录制状态失败:', error);
            ui.updateRecordingStatus('状态未知', false);
        }
    }
    
    /**
     * 更新系统信息
     */
    async updateSystemInfo() {
        try {
            const info = await api.getSystemInfo();
            ui.updateSystemStatus(info);
        } catch (error) {
            console.error('获取系统信息失败:', error);
        }
    }
    
    /**
     * 更新录制文件列表
     */
    async updateRecordingFiles() {
        try {
            const files = await api.getRecordingFiles();
            ui.updateRecordingFiles(files);
        } catch (error) {
            console.error('获取录制文件列表失败:', error);
        }
    }
    
    /**
     * 更新拆分文件夹列表
     */
    async updateSplitFolders() {
        try {
            const folders = await api.getSplitFolders();
            ui.updateSplitFolders(folders);
        } catch (error) {
            console.error('获取拆分文件夹列表失败:', error);
        }
    }
    
    /**
     * 更新拆分进度
     * @param {string} taskId - 任务ID
     */
    async updateSplitProgress(taskId) {
        try {
            const status = await api.getSplitStatus(taskId);
            
            if (status.completed) {
                this.state.splitting.active = false;
                this.state.splitting.taskId = null;
                this.state.splitting.progress = 0;
                
                ui.splitStartButton.disabled = false;
                ui.splitCancelButton.disabled = true;
                
                this.showMessage('视频拆分完成');
                this.updateSplitFolders();
            } else {
                this.state.splitting.progress = status.progress;
                // 这里可以添加进度条更新
            }
        } catch (error) {
            console.error('获取拆分进度失败:', error);
        }
    }
    
    /**
     * 连接摄像头
     */
    async connectCamera() {
        try {
            const devicePath = ui.cameraDeviceSelect.value;
            
            await api.connectCamera(devicePath);
            
            this.showMessage('摄像头已连接');
            await this.updateCameraStatus();
        } catch (error) {
            console.error('连接摄像头失败:', error);
            this.showError('连接摄像头失败: ' + error.message);
        }
    }
    
    /**
     * 断开摄像头
     */
    async disconnectCamera() {
        try {
            await api.disconnectCamera();
            
            this.showMessage('摄像头已断开');
            await this.updateCameraStatus();
        } catch (error) {
            console.error('断开摄像头失败:', error);
            this.showError('断开摄像头失败: ' + error.message);
        }
    }
    
    /**
     * 应用摄像头设置
     */
    async applyCameraSettings() {
        try {
            const resolution = ui.cameraResolutionSelect.value;
            const [width, height] = resolution.split('x').map(Number);
            const fps = Number(ui.cameraFpsSelect.value);
            
            const config = {
                width,
                height,
                fps
            };
            
            await api.updateCameraConfig(config);
            
            this.showMessage('摄像头设置已应用');
            await this.updateCameraStatus();
        } catch (error) {
            console.error('应用摄像头设置失败:', error);
            this.showError('应用摄像头设置失败: ' + error.message);
        }
    }
    
    /**
     * 开始录制
     */
    async startRecording() {
        try {
            await api.startRecording();
            
            this.showMessage('开始录制');
            await this.updateRecordingStatus();
        } catch (error) {
            console.error('开始录制失败:', error);
            this.showError('开始录制失败: ' + error.message);
        }
    }
    
    /**
     * 停止录制
     */
    async stopRecording() {
        try {
            await api.stopRecording();
            
            this.showMessage('停止录制');
            await this.updateRecordingStatus();
            await this.updateRecordingFiles();
        } catch (error) {
            console.error('停止录制失败:', error);
            this.showError('停止录制失败: ' + error.message);
        }
    }
    
    /**
     * 应用录制设置
     */
    async applyRecordingSettings() {
        try {
            const format = ui.recordingFormatSelect.value;
            const bitrate = Number(ui.recordingBitrateSelect.value);
            
            const config = {
                container: format.split(' ')[0].toLowerCase(),
                encoder: format.includes('H.264') ? 'h264' : 'h265',
                bitrate
            };
            
            await api.updateRecordingConfig(config);
            
            this.showMessage('录制设置已应用');
        } catch (error) {
            console.error('应用录制设置失败:', error);
            this.showError('应用录制设置失败: ' + error.message);
        }
    }
    
    /**
     * 显示消息
     * @param {string} message - 消息内容
     */
    showMessage(message) {
        // 这里可以实现消息提示，如使用toast或alert
        console.log('消息:', message);
        alert(message);
    }
    
    /**
     * 显示错误
     * @param {string} error - 错误内容
     */
    showError(error) {
        // 这里可以实现错误提示，如使用toast或alert
        console.error('错误:', error);
        alert('错误: ' + error);
    }
}

// 创建应用实例
const app = new AppManager();
