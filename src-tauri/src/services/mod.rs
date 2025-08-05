//! 服务层模块
//!
//! 包含各种业务服务，如Bilibili直播、代理服务、TTS、OpenAI等

pub mod bilibili;
pub mod openai;
pub mod proxy;
pub mod tts;

// 重新导出服务模块中的公开函数和类型
pub use openai::*;
pub use tts::*;
