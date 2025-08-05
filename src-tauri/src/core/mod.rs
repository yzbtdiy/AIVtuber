//! 核心功能模块
//!
//! 包含应用的核心数据结构、协议定义和基础功能

pub mod proto;
pub mod state;

// 重新导出核心模块
pub use state::*;
pub use proto::*;