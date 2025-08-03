mod proto;
mod bilibili;
mod proxy;
mod handlers;

use handlers::{ClientState, ProxyState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志，设置日志级别
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    log::info!("AIVtuber 应用启动中...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ClientState::default())
        .manage(ProxyState::default())
        .invoke_handler(tauri::generate_handler![
            handlers::connect_bilibili,
            handlers::disconnect_bilibili,
            handlers::get_connection_status,
            handlers::load_config_from_file,
            handlers::save_config_to_file,
            handlers::start_proxy_server,
            handlers::stop_proxy_server,
            handlers::get_proxy_status,
            handlers::text_to_speech,
            handlers::chat_with_openai,
            handlers::chat_and_speak
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
