/**
 * API接口封装
 */

// API基础URL
const API_BASE_URL = window.location.origin;

// API端点
const API_ENDPOINTS = {
    // 摄像头相关
    CAMERA_DEVICES: '/api/camera/devices',
    CAMERA_CONNECT: '/api/camera/connect',
    CAMERA_DISCONNECT: '/api/camera/disconnect',
    CAMERA_STATUS: '/api/camera/status',
    CAMERA_CONFIG: '/api/camera/config',
    
    // 录制相关
    RECORDING_START: '/api/recording/start',
    RECORDING_STOP: '/api/recording/stop',
    RECORDING_STATUS: '/api/recording/status',
    RECORDING_CONFIG: '/api/recording/config',
    RECORDING_FILES: '/api/recording/files',
    RECORDING_DELETE: '/api/recording/delete',
    
    // 拆分相关
    SPLIT_START: '/api/split/start',
    SPLIT_CANCEL: '/api/split/cancel',
    SPLIT_STATUS: '/api/split/status',
    SPLIT_CONFIG: '/api/split/config',
    SPLIT_FOLDERS: '/api/split/folders',
    SPLIT_FRAMES: '/api/split/frames',
    SPLIT_DELETE: '/api/split/delete',
    SPLIT_PACKAGE: '/api/split/package',
    
    // 系统相关
    SYSTEM_STATUS: '/api/system/status',
    SYSTEM_INFO: '/api/system/info',
};

/**
 * API客户端
 */
class ApiClient {
    /**
     * 发送GET请求
     * @param {string} endpoint - API端点
     * @param {Object} params - 查询参数
     * @returns {Promise<Object>} - 响应数据
     */
    async get(endpoint, params = {}) {
        const url = new URL(API_BASE_URL + endpoint);
        
        // 添加查询参数
        Object.keys(params).forEach(key => {
            url.searchParams.append(key, params[key]);
        });
        
        try {
            const response = await fetch(url.toString());
            
            if (!response.ok) {
                throw new Error(`HTTP错误: ${response.status}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('API请求失败:', error);
            throw error;
        }
    }
    
    /**
     * 发送POST请求
     * @param {string} endpoint - API端点
     * @param {Object} data - 请求数据
     * @returns {Promise<Object>} - 响应数据
     */
    async post(endpoint, data = {}) {
        try {
            const response = await fetch(API_BASE_URL + endpoint, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(data),
            });
            
            if (!response.ok) {
                throw new Error(`HTTP错误: ${response.status}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('API请求失败:', error);
            throw error;
        }
    }
    
    /**
     * 发送DELETE请求
     * @param {string} endpoint - API端点
     * @param {Object} data - 请求数据
     * @returns {Promise<Object>} - 响应数据
     */
    async delete(endpoint, data = {}) {
        try {
            const response = await fetch(API_BASE_URL + endpoint, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(data),
            });
            
            if (!response.ok) {
                throw new Error(`HTTP错误: ${response.status}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('API请求失败:', error);
            throw error;
        }
    }
    
    // 摄像头相关API
    
    /**
     * 获取可用摄像头设备列表
     * @returns {Promise<Array>} - 设备列表
     */
    async getCameraDevices() {
        return this.get(API_ENDPOINTS.CAMERA_DEVICES);
    }
    
    /**
     * 连接摄像头
     * @param {string} devicePath - 设备路径
     * @returns {Promise<Object>} - 响应数据
     */
    async connectCamera(devicePath) {
        return this.post(API_ENDPOINTS.CAMERA_CONNECT, { device_path: devicePath });
    }
    
    /**
     * 断开摄像头
     * @returns {Promise<Object>} - 响应数据
     */
    async disconnectCamera() {
        return this.post(API_ENDPOINTS.CAMERA_DISCONNECT);
    }
    
    /**
     * 获取摄像头状态
     * @returns {Promise<Object>} - 状态数据
     */
    async getCameraStatus() {
        return this.get(API_ENDPOINTS.CAMERA_STATUS);
    }
    
    /**
     * 更新摄像头配置
     * @param {Object} config - 配置数据
     * @returns {Promise<Object>} - 响应数据
     */
    async updateCameraConfig(config) {
        return this.post(API_ENDPOINTS.CAMERA_CONFIG, config);
    }
    
    // 录制相关API
    
    /**
     * 开始录制
     * @returns {Promise<Object>} - 响应数据
     */
    async startRecording() {
        return this.post(API_ENDPOINTS.RECORDING_START);
    }
    
    /**
     * 停止录制
     * @returns {Promise<Object>} - 响应数据
     */
    async stopRecording() {
        return this.post(API_ENDPOINTS.RECORDING_STOP);
    }
    
    /**
     * 获取录制状态
     * @returns {Promise<Object>} - 状态数据
     */
    async getRecordingStatus() {
        return this.get(API_ENDPOINTS.RECORDING_STATUS);
    }
    
    /**
     * 更新录制配置
     * @param {Object} config - 配置数据
     * @returns {Promise<Object>} - 响应数据
     */
    async updateRecordingConfig(config) {
        return this.post(API_ENDPOINTS.RECORDING_CONFIG, config);
    }
    
    /**
     * 获取录制文件列表
     * @returns {Promise<Array>} - 文件列表
     */
    async getRecordingFiles() {
        return this.get(API_ENDPOINTS.RECORDING_FILES);
    }
    
    /**
     * 删除录制文件
     * @param {string} filePath - 文件路径
     * @returns {Promise<Object>} - 响应数据
     */
    async deleteRecordingFile(filePath) {
        return this.delete(API_ENDPOINTS.RECORDING_DELETE, { file_path: filePath });
    }
    
    // 拆分相关API
    
    /**
     * 开始拆分
     * @param {string} videoPath - 视频文件路径
     * @returns {Promise<Object>} - 响应数据
     */
    async startSplit(videoPath) {
        return this.post(API_ENDPOINTS.SPLIT_START, { video_path: videoPath });
    }
    
    /**
     * 取消拆分
     * @param {string} taskId - 任务ID
     * @returns {Promise<Object>} - 响应数据
     */
    async cancelSplit(taskId) {
        return this.post(API_ENDPOINTS.SPLIT_CANCEL, { task_id: taskId });
    }
    
    /**
     * 获取拆分状态
     * @param {string} taskId - 任务ID
     * @returns {Promise<Object>} - 状态数据
     */
    async getSplitStatus(taskId) {
        return this.get(API_ENDPOINTS.SPLIT_STATUS, { task_id: taskId });
    }
    
    /**
     * 更新拆分配置
     * @param {Object} config - 配置数据
     * @returns {Promise<Object>} - 响应数据
     */
    async updateSplitConfig(config) {
        return this.post(API_ENDPOINTS.SPLIT_CONFIG, config);
    }
    
    /**
     * 获取拆分文件夹列表
     * @returns {Promise<Array>} - 文件夹列表
     */
    async getSplitFolders() {
        return this.get(API_ENDPOINTS.SPLIT_FOLDERS);
    }
    
    /**
     * 获取拆分帧列表
     * @param {string} folderPath - 文件夹路径
     * @returns {Promise<Array>} - 帧列表
     */
    async getSplitFrames(folderPath) {
        return this.get(API_ENDPOINTS.SPLIT_FRAMES, { folder_path: folderPath });
    }
    
    /**
     * 删除拆分文件夹
     * @param {string} folderPath - 文件夹路径
     * @returns {Promise<Object>} - 响应数据
     */
    async deleteSplitFolder(folderPath) {
        return this.delete(API_ENDPOINTS.SPLIT_DELETE, { folder_path: folderPath });
    }
    
    /**
     * 打包拆分文件夹
     * @param {string} folderPath - 文件夹路径
     * @returns {Promise<Object>} - 响应数据
     */
    async packageSplitFolder(folderPath) {
        return this.post(API_ENDPOINTS.SPLIT_PACKAGE, { folder_path: folderPath });
    }
    
    // 系统相关API
    
    /**
     * 获取系统状态
     * @returns {Promise<Object>} - 状态数据
     */
    async getSystemStatus() {
        return this.get(API_ENDPOINTS.SYSTEM_STATUS);
    }
    
    /**
     * 获取系统信息
     * @returns {Promise<Object>} - 系统信息
     */
    async getSystemInfo() {
        return this.get(API_ENDPOINTS.SYSTEM_INFO);
    }
}

// 创建API客户端实例
const api = new ApiClient();
