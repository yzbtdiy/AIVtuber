use crate::core::ClientState;
use crate::services::bilibili::{BilibiliClient, BilibiliConfig};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub api_url: String,
    pub model: String,
    pub voice: String,
    pub response_format: String,
    pub speed: String,
    pub authorization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub id_code: String,
    pub app_id: u64,
    pub access_key: String,
    pub access_secret: String,
    pub host: String,
    pub openai: Option<OpenAIConfig>,
    pub indextts: Option<TtsConfig>,
}

#[derive(Debug, Serialize)]
pub struct BilibiliResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn connect_bilibili(
    config: AppConfig,
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
                let mut receiver: tokio::sync::mpsc::UnboundedReceiver<
                    crate::core::proto::BilibiliMessage,
                > = receiver;
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
pub async fn disconnect_bilibili(
    client_state: State<'_, ClientState>,
) -> Result<BilibiliResponse, String> {
    let mut client_guard = client_state.lock().await;
    if let Some(mut client) = client_guard.take() {
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
pub async fn get_connection_status(client_state: State<'_, ClientState>) -> Result<bool, String> {
    let client_guard = client_state.lock().await;
    Ok(client_guard.is_some())
}
