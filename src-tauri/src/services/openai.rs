use serde::{Deserialize, Serialize};

// OpenAI API 相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    // pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
    pub finish_reason: Option<String>,
    pub index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub success: bool,
    pub message: String,
    pub content: Option<String>,
}

#[tauri::command]
pub async fn chat_with_openai(
    message: String,
    // temperature: Option<f32>,
) -> Result<ChatResponse, String> {
    // 从配置文件读取OpenAI配置
    let openai_config = match crate::api::config::load_openai_config().await {
        Ok(config) => config,
        Err(e) => {
            log::error!("加载OpenAI配置失败: {}", e);
            return Err(format!("加载OpenAI配置失败: {}", e));
        }
    };

    // let temperature = temperature.unwrap_or(0.7);

    let openai_request = OpenAIRequest {
        model: openai_config.model.clone(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: message,
        }],
        // temperature,
    };

    // 创建HTTP客户端
    let client = reqwest::Client::new();

    log::info!("发送OpenAI API请求: {:?}", openai_request);

    match client
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
                            Ok(ChatResponse {
                                success: true,
                                message: "对话成功".to_string(),
                                content: Some(choice.message.content.clone()),
                            })
                        } else {
                            Err("OpenAI API返回空响应".to_string())
                        }
                    }
                    Err(e) => {
                        log::error!("解析OpenAI API响应失败: {}", e);
                        Err(format!("解析OpenAI API响应失败: {}", e))
                    }
                }
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "未知错误".to_string());
                log::error!("OpenAI API请求失败: {} - {}", status, error_text);
                Err(format!("OpenAI API请求失败: {} - {}", status, error_text))
            }
        }
        Err(e) => {
            log::error!("发送OpenAI API请求失败: {}", e);
            Err(format!("发送OpenAI API请求失败: {}", e))
        }
    }
}
