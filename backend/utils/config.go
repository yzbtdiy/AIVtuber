package utils

import (
	"fmt"

	"github.com/spf13/viper"
)

// 全局配置
var GlobalConfig = viper.New()

// 配置文件路径按优先级排序
var configFilePaths = []string{
	"./.config.yaml",
	"./config.yaml",
}

func init() {
	configLoaded := false

	// 按优先级尝试读取配置文件
	for _, path := range configFilePaths {
		success := readConfig(path)
		if success {
			fmt.Printf("Config loaded from: %s\n", path)
			configLoaded = true
			break
		}
	}

	if !configLoaded {
		fmt.Println("Failed to load any config file")
	}
}

// 读取配置文件，返回是否成功读取
func readConfig(configPath string) bool {
	GlobalConfig.SetConfigFile(configPath)
	if err := GlobalConfig.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			fmt.Printf("Config file not found: %s\n", configPath)
		} else {
			fmt.Printf("Config file parse error: %s - %v\n", configPath, err)
		}
		return false
	}
	return true
}
