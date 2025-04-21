package services

import (
	"context"
	"io"
	"log"
	"net/http"

	"github.com/wailsapp/wails/v3/pkg/application"
)

type HttpServer struct {
	ctx     context.Context
	options application.ServiceOptions
	mux     *http.ServeMux
}

// Name is the name of the service
func (hs *HttpServer) Name() string {
	return "httpServer"
}

func (hs *HttpServer) OnStartup(ctx context.Context, options application.ServiceOptions) error {
	hs.ctx = ctx
	hs.options = options
	hs.mux = http.NewServeMux()
	hs.mux.HandleFunc("/proxy", hs.ServeProxy)
	hs.mux.Handle("/images/", http.StripPrefix("/images/", http.FileServer(http.Dir("./Pictures"))))

	log.Println("Starting HTTP server on :8080")
	go func() {
		if err := http.ListenAndServe("127.0.0.1:12345", hs.mux); err != nil {
			log.Printf("HTTP server failed: %v", err)
		}
	}()

	return nil
}

func (hs *HttpServer) OnShutdown() error {
	return nil
}

func (hs *HttpServer) ServeProxy(w http.ResponseWriter, r *http.Request) {

	// 获取目标url参数
	target := r.URL.Query().Get("url")
	if target == "" {
		http.Error(w, "url param missing", http.StatusBadRequest)
		return
	}
	// 创建代理请求
	req, err := http.NewRequest("GET", target, nil)
	if err != nil {
		http.Error(w, "invalid url", http.StatusBadRequest)
		return
	}
	// 设置必要的请求头
	// req.Header.Set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0")
	// req.Header.Set("Referer", "https://www.bilibili.com")
	// 发送请求
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("Failed to fetch target: %v", err)
		http.Error(w, "fetch error", http.StatusInternalServerError)
		return
	}
	defer resp.Body.Close()
	// 转发响应头
	for k, vv := range resp.Header {
		for _, v := range vv {
			w.Header().Add(k, v)
		}
	}
	// 返回状态码和响应内容
	w.WriteHeader(resp.StatusCode)
	io.Copy(w, resp.Body)
}
