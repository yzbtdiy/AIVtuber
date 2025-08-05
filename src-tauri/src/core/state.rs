//! 全局状态管理模块
//!
//! 定义应用程序的全局状态类型

use crate::services::bilibili::BilibiliClient;
use crate::services::proxy::ProxyServer;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Bilibili客户端状态
pub type ClientState = Arc<Mutex<Option<BilibiliClient>>>;

/// 代理服务器状态
pub type ProxyState = Arc<Mutex<Option<ProxyServer>>>;
