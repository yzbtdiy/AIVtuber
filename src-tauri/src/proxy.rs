use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::oneshot;
use url::Url;
use warp::http::StatusCode;
use warp::{Filter, Reply};

#[derive(Debug, Deserialize)]
struct ProxyQuery {
    url: String,
}

#[derive(Debug, Serialize)]
struct ProxyResponse {
    success: bool,
    message: String,
}

pub struct ProxyServer {
    shutdown_sender: Option<oneshot::Sender<()>>,
}

impl ProxyServer {
    pub fn new() -> Self {
        Self {
            shutdown_sender: None,
        }
    }

    pub async fn start(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        self.shutdown_sender = Some(shutdown_sender);

        let client = Arc::new(Client::new());

        // 创建代理路由
        let proxy_route = warp::path("proxy")
            .and(warp::get())
            .and(warp::query::<ProxyQuery>())
            .and(warp::any().map(move || client.clone()))
            .and_then(handle_proxy);

        // 创建健康检查路由
        let health_route = warp::path("health")
            .and(warp::get())
            .map(|| warp::reply::json(&ProxyResponse {
                success: true,
                message: "代理服务正常运行".to_string(),
            }));

        // CORS 支持
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        let routes = proxy_route
            .or(health_route)
            .with(cors)
            .recover(handle_rejection);

        info!("代理服务启动在端口: {}", port);

        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
                shutdown_receiver.await.ok();
            });

        server.await;
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
            info!("代理服务已停止");
        }
    }
}

async fn handle_proxy(
    query: ProxyQuery,
    client: Arc<Client>,
) -> Result<Box<dyn Reply>, warp::Rejection> {
    // 验证URL格式
    let target_url = match Url::parse(&query.url) {
        Ok(url) => url,
        Err(_) => {
            return Ok(Box::new(warp::reply::with_status(
                warp::reply::json(&ProxyResponse {
                    success: false,
                    message: "无效的URL格式".to_string(),
                }),
                StatusCode::BAD_REQUEST,
            )));
        }
    };

    // 安全检查：只允许HTTP和HTTPS协议
    if target_url.scheme() != "http" && target_url.scheme() != "https" {
        return Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&ProxyResponse {
                success: false,
                message: "只支持HTTP和HTTPS协议".to_string(),
            }),
            StatusCode::BAD_REQUEST,
        )));
    }

    // 禁止访问本地地址（安全考虑）
    // if let Some(host) = target_url.host_str() {
    //     if host == "localhost" 
    //         || host == "127.0.0.1" 
    //         || host.starts_with("192.168.")
    //         || host.starts_with("10.")
    //         || host.starts_with("172.16.")
    //         || host.starts_with("172.17.")
    //         || host.starts_with("172.18.")
    //         || host.starts_with("172.19.")
    //         || host.starts_with("172.20.")
    //         || host.starts_with("172.21.")
    //         || host.starts_with("172.22.")
    //         || host.starts_with("172.23.")
    //         || host.starts_with("172.24.")
    //         || host.starts_with("172.25.")
    //         || host.starts_with("172.26.")
    //         || host.starts_with("172.27.")
    //         || host.starts_with("172.28.")
    //         || host.starts_with("172.29.")
    //         || host.starts_with("172.30.")
    //         || host.starts_with("172.31.") {
    //         return Ok(Box::new(warp::reply::with_status(
    //             warp::reply::json(&ProxyResponse {
    //                 success: false,
    //                 message: "禁止访问内网地址".to_string(),
    //             }),
    //             StatusCode::FORBIDDEN,
    //         )));
    //     }
    // }

    info!("代理请求: {}", query.url);

    // 发起代理请求
    match client.get(&query.url).send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            
            match response.bytes().await {
                Ok(body) => {
                    let mut response_builder = warp::http::Response::builder().status(status);
                    
                    // 复制相关的响应头
                    for (name, value) in headers.iter() {
                        if name == "content-type" 
                            || name == "content-length" 
                            || name == "cache-control"
                            || name == "expires"
                            || name == "last-modified"
                            || name == "etag" {
                            response_builder = response_builder.header(name, value);
                        }
                    }

                    // 添加CORS头
                    response_builder = response_builder
                        .header("access-control-allow-origin", "*")
                        .header("access-control-allow-methods", "GET, POST, PUT, DELETE, OPTIONS")
                        .header("access-control-allow-headers", "content-type, authorization");

                    match response_builder.body(body) {
                        Ok(resp) => Ok(Box::new(resp)),
                        Err(e) => {
                            error!("构建响应失败: {}", e);
                            Ok(Box::new(warp::reply::with_status(
                                warp::reply::json(&ProxyResponse {
                                    success: false,
                                    message: "构建响应失败".to_string(),
                                }),
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )))
                        }
                    }
                }
                Err(e) => {
                    error!("读取响应体失败: {}", e);
                    Ok(Box::new(warp::reply::with_status(
                        warp::reply::json(&ProxyResponse {
                            success: false,
                            message: format!("读取响应体失败: {}", e),
                        }),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )))
                }
            }
        }
        Err(e) => {
            error!("代理请求失败: {}", e);
            Ok(Box::new(warp::reply::with_status(
                warp::reply::json(&ProxyResponse {
                    success: false,
                    message: format!("代理请求失败: {}", e),
                }),
                StatusCode::BAD_GATEWAY,
            )))
        }
    }
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "路径未找到";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "请求体格式错误";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "请求方法不被允许";
    } else {
        error!("未处理的拒绝: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "内部服务器错误";
    }

    let json = warp::reply::json(&ProxyResponse {
        success: false,
        message: message.to_string(),
    });

    Ok(warp::reply::with_status(json, code))
}
