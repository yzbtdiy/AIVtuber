use crate::proto::{Proto, BilibiliMessage, UnknownMessage};
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct BilibiliError {
    message: String,
}

impl fmt::Display for BilibiliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BilibiliError {}

impl From<String> for BilibiliError {
    fn from(message: String) -> Self {
        BilibiliError { message }
    }
}

impl From<&str> for BilibiliError {
    fn from(message: &str) -> Self {
        BilibiliError {
            message: message.to_string(),
        }
    }
}

impl From<serde_json::Error> for BilibiliError {
    fn from(err: serde_json::Error) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for BilibiliError {
    fn from(err: reqwest::Error) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for BilibiliError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

impl From<url::ParseError> for BilibiliError {
    fn from(err: url::ParseError) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

impl From<hmac::digest::InvalidLength> for BilibiliError {
    fn from(err: hmac::digest::InvalidLength) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

impl From<std::time::SystemTimeError> for BilibiliError {
    fn from(err: std::time::SystemTimeError) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::header::InvalidHeaderValue> for BilibiliError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        BilibiliError {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BilibiliConfig {
    pub id_code: String,
    pub app_id: u64,
    pub access_key: String,
    pub access_secret: String,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppStartRequest {
    code: String,
    app_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppStartResponse {
    code: i32,
    message: String,
    data: AppStartData,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppStartData {
    game_info: GameInfo,
    websocket_info: WebSocketInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameInfo {
    game_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketInfo {
    wss_link: Vec<String>,
    auth_body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HeartbeatRequest {
    game_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppEndRequest {
    game_id: String,
    app_id: u64,
}

pub struct BilibiliClient {
    config: BilibiliConfig,
    client: Client,
    game_id: Option<String>,
}

impl BilibiliClient {
    pub fn new(config: BilibiliConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            game_id: None,
        }
    }

    pub async fn connect(&mut self) -> Result<mpsc::UnboundedReceiver<BilibiliMessage>, BilibiliError> {
        let (sender, receiver) = mpsc::unbounded_channel();

        // 获取websocket连接信息
        let (ws_url, auth_body) = self.get_websocket_info().await?;
        
        // 连接websocket
        let url = Url::parse(&ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        
        // 启动各种任务
        self.start_tasks(ws_stream, auth_body, sender).await?;
        
        Ok(receiver)
    }

    async fn start_tasks(
        &self,
        ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        auth_body: String,
        sender: mpsc::UnboundedSender<BilibiliMessage>,
    ) -> Result<(), BilibiliError> {
        let (mut ws_sink, mut ws_stream) = ws_stream.split();

        // 发送认证
        let mut auth_proto = Proto::new();
        auth_proto.body = auth_body.into_bytes();
        auth_proto.op = 7;
        let auth_packet = auth_proto.pack();
        
        ws_sink.send(Message::Binary(auth_packet)).await?;

        // 启动心跳任务
        let ws_sink = Arc::new(tokio::sync::Mutex::new(ws_sink));
        let ws_sink_clone = ws_sink.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(20));
            loop {
                interval.tick().await;
                let mut proto = Proto::new();
                proto.op = 2;
                let packet = proto.pack();
                
                let mut sink = ws_sink_clone.lock().await;
                if let Err(e) = sink.send(Message::Binary(packet)).await {
                    log::error!("发送心跳失败: {}", e);
                    break;
                }
                log::info!("发送WebSocket心跳成功");
            }
        });

        // 启动应用心跳任务
        let client = self.client.clone();
        let config = self.config.clone();
        let game_id = self.game_id.clone().unwrap();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(20));
            loop {
                interval.tick().await;
                let heartbeat_req = HeartbeatRequest {
                    game_id: game_id.clone(),
                };
                
                match Self::send_app_heartbeat(&client, &config, &heartbeat_req).await {
                    Ok(_) => log::info!("发送应用心跳成功"),
                    Err(e) => log::error!("发送应用心跳失败: {}", e),
                }
            }
        });

        // 启动消息接收任务
        tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Binary(data)) => {
                        if let Err(e) = Self::handle_message(&data, &sender).await {
                            log::error!("处理消息失败: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        log::info!("WebSocket连接关闭");
                        break;
                    }
                    Err(e) => {
                        log::error!("WebSocket错误: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn handle_message(
        data: &[u8],
        sender: &mpsc::UnboundedSender<BilibiliMessage>,
    ) -> Result<(), BilibiliError> {
        log::debug!("收到原始数据，长度: {}", data.len());
        
        let mut proto = Proto::new();
        proto.unpack(data).map_err(|e| BilibiliError::from(e))?;

        log::debug!("解析协议包: op={}, ver={}, len={}", proto.op, proto.ver, proto.packet_len);

        match proto.op {
            3 => {
                // 心跳回复
                log::debug!("收到心跳回复");
            }
            8 => {
                // 认证回复
                let body_str = proto.get_body_string().map_err(|e| BilibiliError::from(e))?;
                log::info!("认证回复: {}", body_str);
                let auth_resp: Value = serde_json::from_str(&body_str)?;
                if auth_resp["code"].as_i64() == Some(0) {
                    log::info!("认证成功");
                } else {
                    log::error!("认证失败: {}", body_str);
                }
            }
            5 => {
                // 业务消息
                let body_str = proto.get_body_string().map_err(|e| BilibiliError::from(e))?;
                log::info!("收到业务消息: {}", body_str);
                
                // 首先尝试解析为通用JSON
                match serde_json::from_str::<Value>(&body_str) {
                    Ok(json_value) => {
                        log::info!("JSON解析成功，完整消息: {}", 
                            serde_json::to_string_pretty(&json_value).unwrap_or(json_value.to_string()));
                        
                        // 提取消息类型
                        if let Some(cmd) = json_value.get("cmd").and_then(|v| v.as_str()) {
                            log::info!("消息类型: {}", cmd);
                        }
                        
                        // 尝试解析为BilibiliMessage
                        match serde_json::from_str::<BilibiliMessage>(&body_str) {
                            Ok(message) => {
                                log::info!("成功解析为BilibiliMessage: {:?}", message);
                                if sender.send(message).is_err() {
                                    log::error!("发送消息到通道失败");
                                } else {
                                    log::info!("消息发送到前端成功");
                                }
                            }
                            Err(e) => {
                                log::warn!("无法解析为BilibiliMessage: {}", e);
                                
                                // 尝试解析为通用消息
                                match serde_json::from_str::<UnknownMessage>(&body_str) {
                                    Ok(unknown_msg) => {
                                        log::info!("解析为未知消息类型: cmd={}, data={}", unknown_msg.cmd, unknown_msg.data);
                                    }
                                    Err(e2) => {
                                        log::error!("完全无法解析消息: {}, 原始数据: {}", e2, body_str);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("JSON解析失败: {}, 原始数据: {}", e, body_str);
                    }
                }
            }
            _ => {
                log::debug!("未知操作码: {}, 数据长度: {}", proto.op, proto.body.len());
                if !proto.body.is_empty() {
                    match proto.get_body_string() {
                        Ok(body_str) => log::debug!("未知操作码内容: {}", body_str),
                        Err(e) => log::debug!("无法解析未知操作码内容: {}", e),
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_websocket_info(&mut self) -> Result<(String, String), BilibiliError> {
        let url = format!("{}/v2/app/start", self.config.host);
        let request = AppStartRequest {
            code: self.config.id_code.clone(),
            app_id: self.config.app_id,
        };
        
        let body = serde_json::to_string(&request)?;
        let headers = self.sign(&body)?;
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;
        
        let response_text = response.text().await?;
        let response_data: AppStartResponse = serde_json::from_str(&response_text)?;
        
        if response_data.code != 0 {
            return Err(BilibiliError::from(format!("获取WebSocket信息失败: {}", response_data.message)));
        }
        
        self.game_id = Some(response_data.data.game_info.game_id);
        
        Ok((
            response_data.data.websocket_info.wss_link[0].clone(),
            response_data.data.websocket_info.auth_body,
        ))
    }

    async fn send_app_heartbeat(
        client: &Client,
        config: &BilibiliConfig,
        request: &HeartbeatRequest,
    ) -> Result<(), BilibiliError> {
        let url = format!("{}/v2/app/heartbeat", config.host);
        let body = serde_json::to_string(request)?;
        let headers = Self::sign_static(config, &body)?;
        
        let response = client
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;
        
        let response_text = response.text().await?;
        let response_data: Value = serde_json::from_str(&response_text)?;
        
        if response_data["code"].as_i64() != Some(0) {
            return Err(BilibiliError::from(format!("应用心跳失败: {}", response_text)));
        }
        
        Ok(())
    }

    fn sign(&self, params: &str) -> Result<reqwest::header::HeaderMap, BilibiliError> {
        Self::sign_static(&self.config, params)
    }

    fn sign_static(config: &BilibiliConfig, params: &str) -> Result<reqwest::header::HeaderMap, BilibiliError> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // 计算MD5
        let digest = md5::compute(params);
        let md5_hex = format!("{:x}", digest);
        
        // 生成时间戳和随机数
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let nonce = rand::random::<u64>();
        
        // 构建header map
        let mut header_map = HashMap::new();
        header_map.insert("x-bili-timestamp", timestamp.to_string());
        header_map.insert("x-bili-signature-method", "HMAC-SHA256".to_string());
        header_map.insert("x-bili-signature-nonce", nonce.to_string());
        header_map.insert("x-bili-accesskeyid", config.access_key.clone());
        header_map.insert("x-bili-signature-version", "1.0".to_string());
        header_map.insert("x-bili-content-md5", md5_hex.clone());
        
        // 排序并构建签名字符串
        let mut sorted_keys: Vec<_> = header_map.keys().collect();
        sorted_keys.sort();
        
        let mut header_str = String::new();
        for key in sorted_keys {
            header_str.push_str(&format!("{}:{}\n", key, header_map[key]));
        }
        header_str = header_str.trim_end_matches('\n').to_string();
        
        // 生成HMAC-SHA256签名
        let mut mac = HmacSha256::new_from_slice(config.access_secret.as_bytes())?;
        mac.update(header_str.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        
        // 设置HTTP headers
        headers.insert("x-bili-timestamp", timestamp.to_string().parse()?);
        headers.insert("x-bili-signature-method", "HMAC-SHA256".parse()?);
        headers.insert("x-bili-signature-nonce", nonce.to_string().parse()?);
        headers.insert("x-bili-accesskeyid", config.access_key.parse()?);
        headers.insert("x-bili-signature-version", "1.0".parse()?);
        headers.insert("x-bili-content-md5", md5_hex.parse()?);
        headers.insert("Authorization", signature.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("Accept", "application/json".parse()?);
        
        Ok(headers)
    }

    pub async fn close(&self) -> Result<(), BilibiliError> {
        if let Some(game_id) = &self.game_id {
            let url = format!("{}/v2/app/end", self.config.host);
            let request = AppEndRequest {
                game_id: game_id.clone(),
                app_id: self.config.app_id,
            };
            
            let body = serde_json::to_string(&request)?;
            let headers = self.sign(&body)?;
            
            let response = self.client
                .post(&url)
                .headers(headers)
                .body(body)
                .send()
                .await?;
            
            let response_text = response.text().await?;
            log::info!("关闭应用成功: {}", response_text);
        }
        
        Ok(())
    }
}
