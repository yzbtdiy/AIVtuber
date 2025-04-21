package zhipuai

import (
	"errors"

	"github.com/yzbtdiy/AIVtuber/backend/utils"
	"resty.dev/v3"
)

const (
	ConfigKeyCogVideoXAPI       = "zhipu_ai.models.video.gen_url"
	ConfigKeyCogVideoXResultAPI = "zhipu_ai.models.video.res_url"
)

type CogVideoXModelClient struct {
	client  *resty.Client
	token   string
	gen_api string
	res_api string
}

func NewCogVideoXModelClient() *CogVideoXModelClient {
	return &CogVideoXModelClient{
		client: resty.New(),
	}
}

func (c *CogVideoXModelClient) Init() error {
	c.gen_api = utils.GlobalConfig.GetString(ConfigKeyCogVideoXAPI)
	c.res_api = utils.GlobalConfig.GetString(ConfigKeyCogVideoXResultAPI)
	c.token = utils.GlobalConfig.GetString(ConfigKeyToken)
	if c.gen_api == "" {
		return errors.New("API URL 未配置")
	}
	if c.res_api == "" {
		return errors.New("API Result URL 未配置")
	}
	if c.token == "" {
		return errors.New("API Token 未配置")
	}
	return nil
}

// 生成视频请求数据
type CogVideoXGenerationRequest struct {
	Model     string `json:"model"`
	Prompt    string `json:"prompt,omitempty"`
	ImageUrl  string `json:"image_url,omitempty"`
	Quality   string `json:"quality,omitempty"`
	WithAudio bool   `json:"with_audio,omitempty"`
	Size      string `json:"size,omitempty"`
	Fps       int    `json:"fps,omitempty"`
	RequestId string `json:"request_id,omitempty"`
	UserId    string `json:"user_id,omitempty"`
}

// 生成视频响应数据
type CogVideoXGenerationResponse struct {
	RequestId  string `json:"request_id"`
	Id         string `json:"id"`
	Model      string `json:"model"`
	TaskStatus string `json:"task_status"`
}

// 生成视频
func (c *CogVideoXModelClient) GenerateVideo(req *CogVideoXGenerationRequest) (*CogVideoXGenerationResponse, error) {
	var responseData *CogVideoXGenerationResponse

	res, err := c.client.R().
		SetHeader("Content-Type", "application/json").
		SetHeader("Authorization", "Bearer "+c.token).
		SetBody(req).
		SetResult(&responseData).
		Post(c.gen_api)

	if err != nil {
		return nil, err
	}

	if res.IsError() {
		return nil, errors.New(res.String())
	}

	return responseData, nil
}

// 查询视频结果
type CogVideoXResultData struct {
	Model       string `json:"model"`
	RequestId   string `json:"request_id"`
	TaskStatus  string `json:"task_status"`
	VideoResult []struct {
		Url           string `json:"url"`
		CoverImageUrl string `json:"cover_image_url"`
	} `json:"video_result"`
}

func (c *CogVideoXModelClient) RetrieveVideo(id string) (*CogVideoXResultData, error) {
	var responseData *CogVideoXResultData

	res, err := c.client.R().
		SetHeader("Content-Type", "application/json").
		SetHeader("Authorization", "Bearer "+c.token).
		SetResult(&responseData).
		Get(c.res_api + "/" + id)

	if err != nil {
		return nil, err
	}

	if res.IsError() {
		return nil, errors.New(res.String())
	}

	return responseData, nil
}

func (c *CogVideoXModelClient) Close() {
	if c.client != nil {
		c.client.Close()
	}
}
