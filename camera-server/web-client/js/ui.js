/**
 * UI交互逻辑
 */

/**
 * UI管理器
 */
class UiManager {
    constructor() {
        // 初始化UI元素引用
        this.initElements();
        
        // 初始化事件监听器
        this.initEventListeners();
    }
    
    /**
     * 初始化UI元素引用
     */
    initElements() {
        // 状态元素
        this.cameraStatusElement = document.getElementById('camera-status');
        this.recordingStatusElement = document.getElementById('recording-status');
        this.systemStatusElement = document.getElementById('system-status');
        
        // 摄像头控制元素
        this.cameraPreviewElement = document.getElementById('camera-preview');
        this.cameraConnectButton = document.getElementById('camera-connect');
        this.cameraDisconnectButton = document.getElementById('camera-disconnect');
        this.cameraDeviceSelect = document.getElementById('camera-device');
        this.cameraResolutionSelect = document.getElementById('camera-resolution');
        this.cameraFpsSelect = document.getElementById('camera-fps');
        this.cameraApplyButton = document.getElementById('camera-apply');
        
        // 录制控制元素
        this.recordingStartButton = document.getElementById('recording-start');
        this.recordingStopButton = document.getElementById('recording-stop');
        this.recordingFormatSelect = document.getElementById('recording-format');
        this.recordingBitrateSelect = document.getElementById('recording-bitrate');
        this.recordingApplyButton = document.getElementById('recording-apply');
        this.recordingFilesContainer = document.getElementById('recording-files');
        
        // 拆分控制元素
        this.splitStartButton = document.getElementById('split-start');
        this.splitCancelButton = document.getElementById('split-cancel');
        this.splitFormatSelect = document.getElementById('split-format');
        this.splitFramerateSelect = document.getElementById('split-framerate');
        this.splitQualityInput = document.getElementById('split-quality');
        this.splitQualityValueElement = document.getElementById('split-quality-value');
        this.splitApplyButton = document.getElementById('split-apply');
        this.splitFoldersContainer = document.getElementById('split-folders');
        
        // 系统状态元素
        this.cpuUsageElement = document.getElementById('cpu-usage');
        this.cpuCoresElement = document.getElementById('cpu-cores');
        this.cpuTempElement = document.getElementById('cpu-temp');
        this.memoryUsageElement = document.getElementById('memory-usage');
        this.memoryTotalElement = document.getElementById('memory-total');
        this.memoryUsedElement = document.getElementById('memory-used');
        this.storageUsageElement = document.getElementById('storage-usage');
        this.storageTotalElement = document.getElementById('storage-total');
        this.storageUsedElement = document.getElementById('storage-used');
    }
    
    /**
     * 初始化事件监听器
     */
    initEventListeners() {
        // 摄像头控制事件
        this.cameraConnectButton.addEventListener('click', () => this.onCameraConnect());
        this.cameraDisconnectButton.addEventListener('click', () => this.onCameraDisconnect());
        this.cameraApplyButton.addEventListener('click', () => this.onCameraApplySettings());
        
        // 录制控制事件
        this.recordingStartButton.addEventListener('click', () => this.onRecordingStart());
        this.recordingStopButton.addEventListener('click', () => this.onRecordingStop());
        this.recordingApplyButton.addEventListener('click', () => this.onRecordingApplySettings());
        
        // 拆分控制事件
        this.splitStartButton.addEventListener('click', () => this.onSplitStart());
        this.splitCancelButton.addEventListener('click', () => this.onSplitCancel());
        this.splitApplyButton.addEventListener('click', () => this.onSplitApplySettings());
        this.splitQualityInput.addEventListener('input', () => this.onSplitQualityChange());
    }
    
    /**
     * 更新摄像头状态
     * @param {string} status - 状态文本
     * @param {boolean} isConnected - 是否已连接
     */
    updateCameraStatus(status, isConnected) {
        this.cameraStatusElement.textContent = status;
        
        if (isConnected) {
            this.cameraStatusElement.classList.add('connected');
            this.cameraConnectButton.disabled = true;
            this.cameraDisconnectButton.disabled = false;
            this.recordingStartButton.disabled = false;
            
            // 显示"无预览"提示
            this.cameraPreviewElement.innerHTML = '<div class="no-preview">预览功能暂不可用</div>';
        } else {
            this.cameraStatusElement.classList.remove('connected');
            this.cameraConnectButton.disabled = false;
            this.cameraDisconnectButton.disabled = true;
            this.recordingStartButton.disabled = true;
            
            // 显示"未连接摄像头"提示
            this.cameraPreviewElement.innerHTML = '<div class="no-preview">未连接摄像头</div>';
        }
    }
    
    /**
     * 更新录制状态
     * @param {string} status - 状态文本
     * @param {boolean} isRecording - 是否正在录制
     */
    updateRecordingStatus(status, isRecording) {
        this.recordingStatusElement.textContent = status;
        
        if (isRecording) {
            this.recordingStatusElement.classList.add('recording');
            this.recordingStartButton.disabled = true;
            this.recordingStopButton.disabled = false;
        } else {
            this.recordingStatusElement.classList.remove('recording');
            this.recordingStartButton.disabled = false;
            this.recordingStopButton.disabled = true;
        }
    }
    
    /**
     * 更新系统状态
     * @param {Object} systemInfo - 系统信息
     */
    updateSystemStatus(systemInfo) {
        // 更新CPU信息
        const cpuUsage = systemInfo.cpu.usage;
        this.cpuUsageElement.style.width = `${cpuUsage}%`;
        this.cpuUsageElement.textContent = `${cpuUsage.toFixed(1)}%`;
        this.cpuCoresElement.textContent = systemInfo.cpu.cores;
        
        if (systemInfo.cpu.temperature) {
            this.cpuTempElement.textContent = `${systemInfo.cpu.temperature.toFixed(1)}°C`;
        } else {
            this.cpuTempElement.textContent = '未知';
        }
        
        // 更新内存信息
        const memoryUsage = (systemInfo.memory.used / systemInfo.memory.total) * 100;
        this.memoryUsageElement.style.width = `${memoryUsage}%`;
        this.memoryUsageElement.textContent = `${memoryUsage.toFixed(1)}%`;
        this.memoryTotalElement.textContent = this.formatBytes(systemInfo.memory.total);
        this.memoryUsedElement.textContent = this.formatBytes(systemInfo.memory.used);
        
        // 更新存储信息
        if (systemInfo.disks && systemInfo.disks.length > 0) {
            const disk = systemInfo.disks[0]; // 使用第一个磁盘
            const storageUsage = (disk.used / disk.total) * 100;
            this.storageUsageElement.style.width = `${storageUsage}%`;
            this.storageUsageElement.textContent = `${storageUsage.toFixed(1)}%`;
            this.storageTotalElement.textContent = this.formatBytes(disk.total);
            this.storageUsedElement.textContent = this.formatBytes(disk.used);
        }
    }
    
    /**
     * 更新录制文件列表
     * @param {Array} files - 文件列表
     */
    updateRecordingFiles(files) {
        if (!files || files.length === 0) {
            this.recordingFilesContainer.innerHTML = '<div class="no-files">无录制文件</div>';
            return;
        }
        
        let html = '';
        
        for (const file of files) {
            html += `
                <div class="file-item" data-path="${file.path}">
                    <div class="file-info">
                        <div class="file-name">${file.name}</div>
                        <div class="file-size">${this.formatBytes(file.size)}</div>
                    </div>
                    <div class="file-actions">
                        <button class="btn secondary btn-split" data-path="${file.path}">拆分</button>
                        <button class="btn danger btn-delete" data-path="${file.path}">删除</button>
                    </div>
                </div>
            `;
        }
        
        this.recordingFilesContainer.innerHTML = html;
        
        // 添加事件监听器
        const splitButtons = this.recordingFilesContainer.querySelectorAll('.btn-split');
        const deleteButtons = this.recordingFilesContainer.querySelectorAll('.btn-delete');
        
        splitButtons.forEach(button => {
            button.addEventListener('click', (event) => {
                const path = event.target.getAttribute('data-path');
                this.onSplitFile(path);
            });
        });
        
        deleteButtons.forEach(button => {
            button.addEventListener('click', (event) => {
                const path = event.target.getAttribute('data-path');
                this.onDeleteRecordingFile(path);
            });
        });
    }
    
    /**
     * 更新拆分文件夹列表
     * @param {Array} folders - 文件夹列表
     */
    updateSplitFolders(folders) {
        if (!folders || folders.length === 0) {
            this.splitFoldersContainer.innerHTML = '<div class="no-files">无拆分文件夹</div>';
            return;
        }
        
        let html = '';
        
        for (const folder of folders) {
            html += `
                <div class="file-item" data-path="${folder.path}">
                    <div class="file-info">
                        <div class="file-name">${folder.name}</div>
                        <div class="file-details">
                            ${folder.frame_count} 帧 | ${this.formatBytes(folder.total_size)}
                        </div>
                    </div>
                    <div class="file-actions">
                        <button class="btn secondary btn-package" data-path="${folder.path}">打包</button>
                        <button class="btn danger btn-delete" data-path="${folder.path}">删除</button>
                    </div>
                </div>
            `;
        }
        
        this.splitFoldersContainer.innerHTML = html;
        
        // 添加事件监听器
        const packageButtons = this.splitFoldersContainer.querySelectorAll('.btn-package');
        const deleteButtons = this.splitFoldersContainer.querySelectorAll('.btn-delete');
        
        packageButtons.forEach(button => {
            button.addEventListener('click', (event) => {
                const path = event.target.getAttribute('data-path');
                this.onPackageSplitFolder(path);
            });
        });
        
        deleteButtons.forEach(button => {
            button.addEventListener('click', (event) => {
                const path = event.target.getAttribute('data-path');
                this.onDeleteSplitFolder(path);
            });
        });
    }
    
    /**
     * 格式化字节数为可读字符串
     * @param {number} bytes - 字节数
     * @returns {string} - 格式化后的字符串
     */
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
    
    // 事件处理方法（这些方法将在app.js中实现）
    onCameraConnect() {}
    onCameraDisconnect() {}
    onCameraApplySettings() {}
    onRecordingStart() {}
    onRecordingStop() {}
    onRecordingApplySettings() {}
    onSplitStart() {}
    onSplitCancel() {}
    onSplitApplySettings() {}
    onSplitQualityChange() {
        this.splitQualityValueElement.textContent = this.splitQualityInput.value;
    }
    onSplitFile() {}
    onDeleteRecordingFile() {}
    onPackageSplitFolder() {}
    onDeleteSplitFolder() {}
}

// 创建UI管理器实例
const ui = new UiManager();
