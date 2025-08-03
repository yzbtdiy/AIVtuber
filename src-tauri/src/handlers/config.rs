use crate::handlers::bilibili::BilibiliConfigRequest;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn load_config_from_file() -> Result<Option<BilibiliConfigRequest>, String> {
    // 尝试从多个位置读取配置文件
    let possible_paths = vec![
        PathBuf::from("config.json"),                    // 项目根目录
        PathBuf::from("src-tauri/config.json"),          // Tauri 目录
        PathBuf::from("config/config.json"),             // 配置目录
        PathBuf::from("../config.json"),                 // 上级目录
    ];
    
    for config_path in possible_paths {
        if config_path.exists() {
            log::info!("找到配置文件: {:?}", config_path);
            
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str::<BilibiliConfigRequest>(&content) {
                        Ok(config) => {
                            log::info!("成功加载配置文件");
                            return Ok(Some(config));
                        }
                        Err(e) => {
                            log::error!("解析配置文件失败: {}", e);
                            return Err(format!("解析配置文件失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("读取配置文件失败: {}", e);
                    return Err(format!("读取配置文件失败: {}", e));
                }
            }
        }
    }
    
    log::info!("未找到配置文件");
    Ok(None)
}

#[tauri::command]
pub async fn save_config_to_file(config: BilibiliConfigRequest) -> Result<String, String> {
    let config_path = PathBuf::from("config.json");
    
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, config_json)
        .map_err(|e| format!("保存配置文件失败: {}", e))?;
    
    let absolute_path = config_path.canonicalize()
        .map_err(|e| format!("获取绝对路径失败: {}", e))?;
    
    log::info!("配置文件已保存到: {:?}", absolute_path);
    Ok(format!("配置已保存到: {:?}", absolute_path))
}
