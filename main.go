package main

import (
	"embed"
	"log"
	"time"

	"github.com/wailsapp/wails/v3/pkg/application"

	"github.com/yzbtdiy/AIVtuber/backend/utils"
	"github.com/yzbtdiy/AIVtuber/services"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	utils.WailsApp = application.New(application.Options{
		Name:        "AIVtuber",
		Description: "A live tool",
		Services: []application.Service{
			application.NewService(services.NewBiliBiliLiveService()),
			application.NewService(&services.HttpServer{}),
		},
		Assets: application.AssetOptions{
			Handler: application.AssetFileServerFS(assets),
		},
		Mac: application.MacOptions{
			ApplicationShouldTerminateAfterLastWindowClosed: true,
		},
	})
	utils.WailsApp.NewWebviewWindowWithOptions(application.WebviewWindowOptions{
		Title:      "AIVtuber",
		StartState: application.WindowStateMaximised,
		Mac: application.MacWindow{
			InvisibleTitleBarHeight: 50,
			Backdrop:                application.MacBackdropTranslucent,
			TitleBar:                application.MacTitleBarHiddenInset,
		},
		BackgroundColour: application.NewRGB(27, 38, 54),
		URL:              "/",
	})
	go func() {
		for {
			now := time.Now().Format(time.RFC1123)
			utils.WailsApp.EmitEvent("time", now)
			time.Sleep(time.Second)
		}
	}()
	err := utils.WailsApp.Run()
	if err != nil {
		log.Fatal(err)
	}
}
