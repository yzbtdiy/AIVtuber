use crate::proxy::ProxyServer;
use crate::handlers::bilibili::BilibiliResponse;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// 全局状态管理
pub type ProxyState = Arc<Mutex<Option<ProxyServer>>>;

#[tauri::command]
pub async fn start_proxy_server(
    proxy_state: State<'_, ProxyState>,
    port: Option<u16>,
) -> Result<BilibiliResponse, String> {
    let port = port.unwrap_or(12345);
    
    let mut proxy_guard = proxy_state.lock().await;
    
    if proxy_guard.is_some() {
        return Ok(BilibiliResponse {
            success: false,
            message: "代理服务已经在运行".to_string(),
        });
    }

    let mut proxy_server = ProxyServer::new();
    
    // 在后台启动代理服务
    tokio::spawn(async move {
        if let Err(e) = proxy_server.start(port).await {
            log::error!("代理服务启动失败: {}", e);
        }
    });

    *proxy_guard = Some(ProxyServer::new());
    
    Ok(BilibiliResponse {
        success: true,
        message: format!("代理服务已启动在端口 {}", port),
    })
}

#[tauri::command]
pub async fn stop_proxy_server(
    proxy_state: State<'_, ProxyState>,
) -> Result<BilibiliResponse, String> {
    let mut proxy_guard = proxy_state.lock().await;
    
    if let Some(mut proxy_server) = proxy_guard.take() {
        proxy_server.stop();
        Ok(BilibiliResponse {
            success: true,
            message: "代理服务已停止".to_string(),
        })
    } else {
        Ok(BilibiliResponse {
            success: false,
            message: "代理服务未运行".to_string(),
        })
    }
}

#[tauri::command]
pub async fn get_proxy_status(
    proxy_state: State<'_, ProxyState>,
) -> Result<bool, String> {
    let proxy_guard = proxy_state.lock().await;
    Ok(proxy_guard.is_some())
}
