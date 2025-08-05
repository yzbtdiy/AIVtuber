use crate::core::{BilibiliMessage, Proto, UnknownMessage};
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
use tokio::sync::broadcast;
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
    // 用于控制心跳任务停止的取消令牌
    heartbeat_cancel_tx: Option<broadcast::Sender<()>>,
    // 用于保存WebSocket连接，便于主动关闭
    ws_sink: Option<
        Arc<
            tokio::sync::Mutex<
                futures_util::stream::SplitSink<
                    tokio_tungstenite::WebSocketStream<
                        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                    >,
                    Message,
                >,
            >,
        >,
    >,
}

impl BilibiliClient {
    pub fn new(config: BilibiliConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            game_id: None,
            heartbeat_cancel_tx: None,
            ws_sink: None,
        }
    }

    pub async fn connect(
        &mut self,
    ) -> Result<mpsc::UnboundedReceiver<BilibiliMessage>, BilibiliError> {
        log::info!("=== 开始连接哔哩哔哩直播间 ===");
        let (sender, receiver) = mpsc::unbounded_channel();

        // 获取websocket连接信息
        log::info!("正在获取WebSocket连接信息...");
        let (ws_url, auth_body) = self.get_websocket_info().await?;
        log::info!("WebSocket URL: {}", ws_url);

        // 连接websocket
        log::info!("正在连接WebSocket...");
        let url = Url::parse(&ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        log::info!("WebSocket连接成功！");

        // 启动各种任务
        self.start_tasks(ws_stream, auth_body, sender).await?;

        Ok(receiver)
    }

    async fn start_tasks(
        &mut self,
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        auth_body: String,
        sender: mpsc::UnboundedSender<BilibiliMessage>,
    ) -> Result<(), BilibiliError> {
        let (mut ws_sink, mut ws_stream) = ws_stream.split();

        // 创建心跳取消通道
        let (cancel_tx, _cancel_rx) = broadcast::channel(1);
        self.heartbeat_cancel_tx = Some(cancel_tx.clone());

        // 发送认证
        log::info!("正在发送认证信息...");
        let mut auth_proto = Proto::new();
        auth_proto.body = auth_body.into_bytes();
        auth_proto.op = 7;
        let auth_packet = auth_proto.pack();

        ws_sink.send(Message::Binary(auth_packet)).await?;
        log::info!("认证信息发送成功！");

        // 启动WebSocket心跳任务
        let ws_sink = Arc::new(tokio::sync::Mutex::new(ws_sink));
        self.ws_sink = Some(ws_sink.clone());
        let ws_sink_clone = ws_sink.clone();
        let mut cancel_rx_ws = cancel_tx.subscribe();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(20));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut proto = Proto::new();
                        proto.op = 2;
                        let packet = proto.pack();

                        let mut sink = ws_sink_clone.lock().await;
                        if let Err(e) = sink.send(Message::Binary(packet)).await {
                            log::error!("发送WebSocket心跳失败: {}", e);
                            break;
                        }
                        log::info!("发送WebSocket心跳成功");
                    }
                    _ = cancel_rx_ws.recv() => {
                        log::info!("WebSocket心跳任务已停止");
                        break;
                    }
                }
            }
        });

        // 启动应用心跳任务
        let client = self.client.clone();
        let config = self.config.clone();
        let game_id = self.game_id.clone().unwrap();
        let mut cancel_rx_app = cancel_tx.subscribe();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(20));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let heartbeat_req = HeartbeatRequest {
                            game_id: game_id.clone(),
                        };

                        match Self::send_app_heartbeat(&client, &config, &heartbeat_req).await {
                            Ok(_) => log::info!("发送应用心跳成功"),
                            Err(e) => {
                                log::error!("发送应用心跳失败: {}", e);
                                // 如果心跳失败，也停止任务
                                break;
                            }
                        }
                    }
                    _ = cancel_rx_app.recv() => {
                        log::info!("应用心跳任务已停止");
                        break;
                    }
                }
            }
        });

        // 启动消息接收任务
        let mut cancel_rx_msg = cancel_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = ws_stream.next() => {
                        match msg {
                            Some(Ok(Message::Binary(data))) => {
                                if let Err(e) = Self::handle_message(&data, &sender).await {
                                    log::error!("处理消息失败: {}", e);
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                log::info!("WebSocket连接关闭");
                                break;
                            }
                            Some(Err(e)) => {
                                log::error!("WebSocket错误: {}", e);
                                break;
                            }
                            None => {
                                log::info!("WebSocket连接结束");
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ = cancel_rx_msg.recv() => {
                        log::info!("消息接收任务已停止");
                        break;
                    }
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

        log::debug!(
            "解析协议包: op={}, ver={}, len={}",
            proto.op,
            proto.ver,
            proto.packet_len
        );

        match proto.op {
            3 => {
                // 心跳回复
                log::debug!("收到心跳回复");
            }
            8 => {
                // 认证回复
                let body_str = proto
                    .get_body_string()
                    .map_err(|e| BilibiliError::from(e))?;
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
                let body_str = proto
                    .get_body_string()
                    .map_err(|e| BilibiliError::from(e))?;
                log::info!("收到业务消息: {}", body_str);

                // 首先尝试解析为通用JSON
                match serde_json::from_str::<Value>(&body_str) {
                    Ok(json_value) => {
                        // log::info!("JSON解析成功，完整消息: {}",
                        //     serde_json::to_string_pretty(&json_value).unwrap_or(json_value.to_string()));

                        // 提取消息类型
                        if let Some(cmd) = json_value.get("cmd").and_then(|v| v.as_str()) {
                            log::info!("消息类型: {}", cmd);
                        }

                        // 尝试解析为BilibiliMessage
                        match serde_json::from_str::<BilibiliMessage>(&body_str) {
                            Ok(message) => {
                                log::info!("成功解析为BilibiliMessage: {:?}", message);

                                // 检查是否是交互结束消息
                                if matches!(&message, BilibiliMessage::InteractionEnd { .. }) {
                                    log::info!("收到交互结束消息，连接即将断开");
                                }

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
                                        log::info!(
                                            "解析为未知消息类型: cmd={}, data={}",
                                            unknown_msg.cmd,
                                            unknown_msg.data
                                        );
                                    }
                                    Err(e2) => {
                                        log::error!(
                                            "完全无法解析消息: {}, 原始数据: {}",
                                            e2,
                                            body_str
                                        );
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

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_data: AppStartResponse = serde_json::from_str(&response_text)?;

        if response_data.code != 0 {
            return Err(BilibiliError::from(format!(
                "获取WebSocket信息失败: {}",
                response_data.message
            )));
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

        let response = client.post(&url).headers(headers).body(body).send().await?;

        let response_text = response.text().await?;
        let response_data: Value = serde_json::from_str(&response_text)?;

        if response_data["code"].as_i64() != Some(0) {
            return Err(BilibiliError::from(format!(
                "应用心跳失败: {}",
                response_text
            )));
        }

        Ok(())
    }

    fn sign(&self, params: &str) -> Result<reqwest::header::HeaderMap, BilibiliError> {
        Self::sign_static(&self.config, params)
    }

    fn sign_static(
        config: &BilibiliConfig,
        params: &str,
    ) -> Result<reqwest::header::HeaderMap, BilibiliError> {
        let mut headers = reqwest::header::HeaderMap::new();

        // 计算MD5
        let digest = md5::compute(params);
        let md5_hex = format!("{:x}", digest);

        // 生成时间戳和随机数
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
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

    pub async fn close(&mut self) -> Result<(), BilibiliError> {
        log::info!("=== 开始断开连接流程 ===");

        // 首先停止心跳任务
        if let Some(cancel_tx) = self.heartbeat_cancel_tx.take() {
            log::info!("正在停止心跳任务...");
            let _ = cancel_tx.send(());
        }

        // 主动关闭WebSocket连接
        if let Some(ws_sink) = self.ws_sink.take() {
            log::info!("正在关闭WebSocket连接...");
            let mut sink = ws_sink.lock().await;
            if let Err(e) = sink.close().await {
                log::warn!("关闭WebSocket连接失败: {}", e);
            } else {
                log::info!("WebSocket连接已关闭");
            }
        }

        // 然后关闭应用连接
        if let Some(game_id) = &self.game_id {
            log::info!("正在关闭应用连接...");
            let url = format!("{}/v2/app/end", self.config.host);
            let request = AppEndRequest {
                game_id: game_id.clone(),
                app_id: self.config.app_id,
            };

            let body = serde_json::to_string(&request)?;
            let headers = self.sign(&body)?;

            let response = self
                .client
                .post(&url)
                .headers(headers)
                .body(body)
                .send()
                .await?;

            let response_text = response.text().await?;
            log::info!("关闭应用成功: {}", response_text);
        }

        // 清理game_id
        self.game_id = None;
        log::info!("=== 断开连接流程完成 ===");

        Ok(())
    }
}
