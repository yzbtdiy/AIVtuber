mod proto;
mod bilibili;

use bilibili::{BilibiliClient, BilibiliConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BilibiliConfigRequest {
    id_code: String,
    app_id: u64,
    access_key: String,
    access_secret: String,
    host: String,
}

#[derive(Debug, Serialize)]
struct BilibiliResponse {
    success: bool,
    message: String,
}

// 全局状态管理
type ClientState = Arc<Mutex<Option<BilibiliClient>>>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn connect_bilibili(
    config: BilibiliConfigRequest,
    client_state: State<'_, ClientState>,
    app_handle: tauri::AppHandle,
) -> Result<BilibiliResponse, String> {
    let bili_config = BilibiliConfig {
        id_code: config.id_code,
        app_id: config.app_id,
        access_key: config.access_key,
        access_secret: config.access_secret,
        host: config.host,
    };

    let mut client = BilibiliClient::new(bili_config);
    
    match client.connect().await {
        Ok(receiver) => {
            // 启动消息处理任务
            let app_handle_clone = app_handle.clone();
            tokio::spawn(async move {
                let mut receiver = receiver;
                while let Some(message) = receiver.recv().await {
                    // 发送消息到前端
                    if let Err(e) = app_handle_clone.emit("bilibili-message", &message) {
                        log::error!("发送消息到前端失败: {}", e);
                    }
                }
            });

            // 保存客户端状态
            *client_state.lock().await = Some(client);
            
            Ok(BilibiliResponse {
                success: true,
                message: "连接成功".to_string(),
            })
        }
        Err(e) => {
            log::error!("连接失败: {}", e);
            Err(format!("连接失败: {}", e))
        }
    }
}

#[tauri::command]
async fn disconnect_bilibili(
    client_state: State<'_, ClientState>,
) -> Result<BilibiliResponse, String> {
    let mut client_guard = client_state.lock().await;
    if let Some(client) = client_guard.take() {
        match client.close().await {
            Ok(_) => Ok(BilibiliResponse {
                success: true,
                message: "断开连接成功".to_string(),
            }),
            Err(e) => {
                log::error!("断开连接失败: {}", e);
                Err(format!("断开连接失败: {}", e))
            }
        }
    } else {
        Ok(BilibiliResponse {
            success: true,
            message: "未连接".to_string(),
        })
    }
}

#[tauri::command]
async fn get_connection_status(
    client_state: State<'_, ClientState>,
) -> Result<bool, String> {
    let client_guard = client_state.lock().await;
    Ok(client_guard.is_some())
}

#[tauri::command]
async fn load_config_from_file() -> Result<Option<BilibiliConfigRequest>, String> {
    // 尝试从多个位置读取配置文件
    let possible_paths = vec![
        PathBuf::from("config.json"),                    // 项目根目录
        PathBuf::from("src-tauri/config.json"),          // Tauri 目录
        PathBuf::from("config/config.json"),             // 配置目录
        PathBuf::from("../config.json"),                 // 上级目录
    ];
    
    for config_path in possible_paths {
        if config_path.exists() {
            log::info!("找到配置文件: {:?}", config_path);
            
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str::<BilibiliConfigRequest>(&content) {
                        Ok(config) => {
                            log::info!("成功加载配置文件");
                            return Ok(Some(config));
                        }
                        Err(e) => {
                            log::error!("解析配置文件失败: {}", e);
                            return Err(format!("解析配置文件失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("读取配置文件失败: {}", e);
                    return Err(format!("读取配置文件失败: {}", e));
                }
            }
        }
    }
    
    log::info!("未找到配置文件");
    Ok(None)
}

#[tauri::command]
async fn save_config_to_file(config: BilibiliConfigRequest) -> Result<String, String> {
    let config_path = PathBuf::from("config.json");
    
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, config_json)
        .map_err(|e| format!("保存配置文件失败: {}", e))?;
    
    let absolute_path = config_path.canonicalize()
        .map_err(|e| format!("获取绝对路径失败: {}", e))?;
    
    log::info!("配置文件已保存到: {:?}", absolute_path);
    Ok(format!("配置已保存到: {:?}", absolute_path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ClientState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            connect_bilibili,
            disconnect_bilibili,
            get_connection_status,
            load_config_from_file,
            save_config_to_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
