mod api;
mod core;
mod services;

use core::{ClientState, ProxyState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .level(log::LevelFilter::Info)
                .build(),
        )
        .manage(ClientState::default())
        .manage(ProxyState::default())
        .invoke_handler(tauri::generate_handler![
            api::connect_bilibili,
            api::disconnect_bilibili,
            api::get_connection_status,
            api::load_config_from_file,
            api::save_config_to_file,
            api::start_proxy_server,
            api::stop_proxy_server,
            api::get_proxy_status,
            services::text_to_speech,
            services::chat_with_openai,
            api::chat_and_speak
        ])
        .setup(|_app| {
            log::info!("AIVtuber 应用启动完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
