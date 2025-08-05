use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

// TTS相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub audio_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TtsResponse {
    pub success: bool,
    pub message: String,
    pub audio_data: Option<String>, // 改为base64编码的字符串
}

#[tauri::command]
pub async fn text_to_speech(text: String) -> Result<TtsResponse, String> {
    // 从配置文件读取TTS配置
    let tts_config = match crate::api::config::load_tts_config().await {
        Ok(config) => config,
        Err(e) => {
            log::error!("加载IndexTTS配置失败: {}", e);
            return Err(format!("加载IndexTTS配置失败: {}", e));
        }
    };

    let tts_request = TtsRequest {
        text,
        audio_paths: tts_config.audio_paths.clone(),
    };

    // 创建HTTP客户端
    let client = reqwest::Client::new();

    log::info!("发送TTS请求: {:?}", tts_request);

    match client
        .post(&tts_config.api_url)
        // .header("Content-Type", "application/json")
        .json(&tts_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(audio_bytes) => {
                        log::info!("TTS请求成功，接收到 {} 字节的音频数据", audio_bytes.len());

                        // 将音频数据编码为base64字符串
                        let audio_base64 = general_purpose::STANDARD.encode(&audio_bytes);

                        Ok(TtsResponse {
                            success: true,
                            message: "文本转语音成功".to_string(),
                            audio_data: Some(audio_base64),
                        })
                    }
                    Err(e) => {
                        log::error!("读取音频数据失败: {}", e);
                        Err(format!("读取音频数据失败: {}", e))
                    }
                }
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "未知错误".to_string());
                log::error!("TTS请求失败: {} - {}", status, error_text);
                Err(format!("TTS请求失败: {} - {}", status, error_text))
            }
        }
        Err(e) => {
            log::error!("发送TTS请求失败: {}", e);
            Err(format!("发送TTS请求失败: {}", e))
        }
    }
}
