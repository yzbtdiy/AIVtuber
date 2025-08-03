use crate::handlers::openai::{OpenAIMessage, OpenAIRequest, OpenAIResponse};
use crate::handlers::tts::TtsRequest;
use crate::handlers::bilibili::{BilibiliConfigRequest, OpenAIConfig, TtsConfig};
use serde::Serialize;
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::PathBuf;

// 整合对话和TTS的响应结构
#[derive(Debug, Serialize)]
pub struct ChatAndSpeakResponse {
    pub success: bool,
    pub message: String,
    pub chat_content: Option<String>,
    pub audio_data: Option<String>, // base64编码的音频数据
}

// 从配置文件读取OpenAI配置
async fn load_openai_config() -> Result<OpenAIConfig, String> {
    let possible_paths = vec![
        PathBuf::from("config.json"),
        PathBuf::from("src-tauri/config.json"),
        PathBuf::from("config/config.json"),
        PathBuf::from("../config.json"),
    ];
    
    for config_path in possible_paths {
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str::<BilibiliConfigRequest>(&content) {
                        Ok(config) => {
                            if let Some(openai_config) = config.openai {
                                return Ok(openai_config);
                            } else {
                                return Err("配置文件中未找到OpenAI配置".to_string());
                            }
                        }
                        Err(e) => {
                            return Err(format!("解析配置文件失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("读取配置文件失败: {}", e));
                }
            }
        }
    }
    
    Err("未找到配置文件".to_string())
}

// 从配置文件读取TTS配置
async fn load_tts_config() -> Result<TtsConfig, String> {
    let possible_paths = vec![
        PathBuf::from("config.json"),
        PathBuf::from("src-tauri/config.json"),
        PathBuf::from("config/config.json"),
        PathBuf::from("../config.json"),
    ];
    
    for config_path in possible_paths {
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str::<BilibiliConfigRequest>(&content) {
                        Ok(config) => {
                            if let Some(tts_config) = config.indextts {
                                return Ok(tts_config);
                            } else {
                                return Err("配置文件中未找到IndexTTS配置".to_string());
                            }
                        }
                        Err(e) => {
                            return Err(format!("解析配置文件失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("读取配置文件失败: {}", e));
                }
            }
        }
    }
    
    Err("未找到配置文件".to_string())
}

#[tauri::command]
pub async fn chat_and_speak(
    message: String,
) -> Result<ChatAndSpeakResponse, String> {
    // 第一步：从配置文件读取OpenAI配置
    log::info!("开始整合对话和TTS流程，用户消息: {}", message);
    
    let openai_config = match load_openai_config().await {
        Ok(config) => config,
        Err(e) => {
            log::error!("加载OpenAI配置失败: {}", e);
            return Err(format!("加载OpenAI配置失败: {}", e));
        }
    };
    
    // 读取TTS配置
    let tts_config = match load_tts_config().await {
        Ok(config) => config,
        Err(e) => {
            log::error!("加载IndexTTS配置失败: {}", e);
            return Err(format!("加载IndexTTS配置失败: {}", e));
        }
    };
    
    let openai_request = OpenAIRequest {
        model: openai_config.model.clone(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: message,
        }],
    };
    
    // 创建HTTP客户端
    let client = reqwest::Client::new();
    
    log::info!("发送OpenAI API请求，模型: {}", openai_config.model);
    
    // 调用 OpenAI API
    let chat_content = match client
        .post(&openai_config.api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_config.api_key))
        .json(&openai_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<OpenAIResponse>().await {
                    Ok(openai_response) => {
                        log::info!("OpenAI API请求成功");
                        
                        if let Some(choice) = openai_response.choices.first() {
                            log::info!("AI回复: {}", choice.message.content);
                            choice.message.content.clone()
                        } else {
                            return Err("OpenAI API返回空响应".to_string());
                        }
                    }
                    Err(e) => {
                        log::error!("解析OpenAI API响应失败: {}", e);
                        return Err(format!("解析OpenAI API响应失败: {}", e));
                    }
                }
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
                log::error!("OpenAI API请求失败: {} - {}", status, error_text);
                return Err(format!("OpenAI API请求失败: {} - {}", status, error_text));
            }
        }
        Err(e) => {
            log::error!("发送OpenAI API请求失败: {}", e);
            return Err(format!("发送OpenAI API请求失败: {}", e));
        }
    };

    // 第二步：将 AI 回复转换为语音
    log::info!("开始将AI回复转换为语音: {}", chat_content);
    
    let tts_request = TtsRequest {
        text: chat_content.clone(),
        audio_paths: tts_config.audio_paths.clone(),
    };
    
    // 调用 TTS API
    let audio_data = match client
        .post(&tts_config.api_url)
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
                        Some(audio_base64)
                    }
                    Err(e) => {
                        log::error!("读取TTS音频数据失败: {}", e);
                        // TTS失败不影响对话结果，继续返回文本
                        None
                    }
                }
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
                log::error!("TTS请求失败: {} - {}", status, error_text);
                // TTS失败不影响对话结果，继续返回文本
                None
            }
        }
        Err(e) => {
            log::error!("发送TTS请求失败: {}", e);
            // TTS失败不影响对话结果，继续返回文本
            None
        }
    };

    // 返回整合结果
    let success_message = if audio_data.is_some() {
        "对话和语音生成成功".to_string()
    } else {
        "对话成功，但语音生成失败".to_string()
    };

    log::info!("整合流程完成: {}", success_message);

    Ok(ChatAndSpeakResponse {
        success: true,
        message: success_message,
        chat_content: Some(chat_content),
        audio_data,
    })
}
