package zhipuai

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"os"
	"sync"

	"github.com/yzbtdiy/AIVtuber/backend/utils"

	"resty.dev/v3"
)

// 默认配置值
const (
	DefaultMaxContextLength = 10
	DefaultModel            = "glm-4-voice"
)

// 配置键名
const (
	ConfigKeyAPI              = "zhipu_ai.models.voice.url"
	ConfigKeyToken            = "zhipu_ai.token"
	ConfigKeyModel            = "zhipu_ai.models.voice.name"
	ConfigKeySystemPrompt     = "zhipu_ai.models.voice.system_prompt"
	ConfigKeyMaxContextLength = "zhipu_ai.models.voice.max_context_length"
)

type GLM4VoiceModelClient struct {
	client       *resty.Client
	token        string
	api          string
	model        string
	chatContext  []GLM4VoiceMessage
	contextMutex sync.Mutex
}

func NewGLM4VoiceModelClient() *GLM4VoiceModelClient {
	return &GLM4VoiceModelClient{
		client:      resty.New(),
		chatContext: []GLM4VoiceMessage{},
	}
}

// 定义请求的结构体
type GLM4VoiceRequestData struct {
	Model       string             `json:"model"`
	Messages    []GLM4VoiceMessage `json:"messages"`
	RequestId   string             `json:"request_id,omitempty"`
	DoSample    bool               `json:"do_sample,omitempty"`
	Stream      bool               `json:"stream,omitempty"`
	Temperature float64            `json:"temperature,omitempty"`
	TopP        float64            `json:"top_p,omitempty"`
	MaxTokens   int                `json:"max_tokens,omitempty"`
	Stop        []string           `json:"stop,omitempty"`
	UserId      string             `json:"user_id,omitempty"`
}

type GLM4VoiceMessage struct {
	Role    string                `json:"role"`
	Content interface{}           `json:"content,omitempty"`
	Audio   GLM4VoiceMessageAudio `json:"audio,omitempty"`
}

type GLM4VoiceMessageContent struct {
	Type       string `json:"type"`
	Text       string `json:"text,omitempty"`
	InputAudio struct {
		Data   string `json:"data"`
		Format string `json:"format"`
	} `json:"input_audio,omitempty"`
}

type GLM4VoiceMessageAudio struct {
	Id string `json:"id"`
}

// 定义响应的结构体
type GLM4VoiceResponseData struct {
	Id        string `json:"id"`
	Created   int64  `json:"created"`
	Model     string `json:"model"`
	RequestId string `json:"request_id"`
	Choices   []struct {
		Index        int    `json:"index"`
		FinishReason string `json:"finish_reason"`
		Message      struct {
			Role    string `json:"role"`
			Content string `json:"content"`
			Audio   struct {
				Id        string `json:"id"`
				Data      string `json:"data"`
				ExpiresAt int64  `json:"expires_at"`
			} `json:"audio"`
		}
	} `json:"choices"`
	Usage struct {
		PromptTokens     int `json:"prompt_tokens"`
		CompletionTokens int `json:"completion_tokens"`
		TotalTokens      int `json:"total_tokens"`
	} `json:"usage"`
	ContentFilter []struct {
		Role  string `json:"role"`
		Level int    `json:"level"`
	} `json:"content_filter"`
}

func (c *GLM4VoiceModelClient) manageChatContextLength() {
	c.contextMutex.Lock()
	defer c.contextMutex.Unlock()

	maxLength := utils.GlobalConfig.GetInt(ConfigKeyMaxContextLength)
	if maxLength <= 0 {
		maxLength = DefaultMaxContextLength
		log.Printf("使用默认最大上下文长度: %d\n", DefaultMaxContextLength)
	}

	// 计算每轮对话包含用户和助手各一条消息，系统提示占一条
	maxTotalLength := maxLength*2 + 1

	// 如果当前上下文长度超过最大允许长度
	if len(c.chatContext) > maxTotalLength {
		log.Printf("对话上下文长度(%d)超过最大限制(%d)，正在裁剪...\n", len(c.chatContext), maxTotalLength)

		// 保留第一条系统消息
		systemMessage := c.chatContext[0]
		// 保留最近的消息（用户和助手的对话）
		recentMessages := c.chatContext[len(c.chatContext)-maxLength*2:]

		// 重构对话上下文
		newContext := make([]GLM4VoiceMessage, 0, maxTotalLength)
		newContext = append(newContext, systemMessage)
		newContext = append(newContext, recentMessages...)

		c.chatContext = newContext
		log.Printf("裁剪后对话上下文长度: %d\n", len(c.chatContext))
	}
}

// Init initializes global context if empty, then sets up a voice generation request.
func (c *GLM4VoiceModelClient) Init() error {
	c.contextMutex.Lock()
	if len(c.chatContext) == 0 {
		systemPrompt := utils.GlobalConfig.GetString(ConfigKeySystemPrompt)
		if systemPrompt == "" {
			log.Println("警告: 系统提示为空，使用默认值")
			systemPrompt = "从现在开始你将作为一个AI助手进行对话。"
		}
		c.chatContext = []GLM4VoiceMessage{{Role: "system", Content: systemPrompt}}
	}
	c.contextMutex.Unlock()

	// 集中读取配置
	c.api = utils.GlobalConfig.GetString(ConfigKeyAPI)
	c.token = utils.GlobalConfig.GetString(ConfigKeyToken)
	c.model = utils.GlobalConfig.GetString(ConfigKeyModel)

	// 验证必要配置
	if c.api == "" {
		return errors.New("API URL 未配置")
	}
	if c.token == "" {
		return errors.New("API Token 未配置")
	}

	// 使用默认值
	if c.model == "" {
		c.model = DefaultModel
		log.Printf("使用默认模型: %s\n", DefaultModel)
	}

	return nil
}

// Chat sends the request to the GLM4Voice API and retrieves the response.
func (c *GLM4VoiceModelClient) Chat(question string) (*GLM4VoiceResponseData, error) {
	// 先将用户消息加入上下文
	c.contextMutex.Lock()
	c.chatContext = append(c.chatContext, GLM4VoiceMessage{
		Role:    "user",
		Content: []GLM4VoiceMessageContent{{Type: "text", Text: question}},
	})
	c.contextMutex.Unlock()

	// 管理上下文长度
	c.manageChatContextLength()

	// 构造请求
	questionData := &GLM4VoiceRequestData{
		Model:    c.model,
		Messages: c.chatContext,
	}

	var responseData *GLM4VoiceResponseData

	// 记录请求日志
	log.Printf("发送请求到 %s，使用模型: %s\n", c.api, questionData.Model)

	res, err := c.client.R().
		EnableTrace().
		SetHeader("Content-Type", "application/json").
		SetHeader("Authorization", "Bearer "+c.token).
		SetBody(questionData).
		SetResult(&responseData).
		Post(c.api)

	if err != nil {
		log.Printf("请求失败: %v\n", err)
		return nil, fmt.Errorf("API请求失败: %w", err)
	}

	// 检查HTTP响应状态
	if res.IsError() {
		log.Printf("服务器返回错误: %d - %s\n", res.StatusCode(), res.String())
		return nil, fmt.Errorf("API返回错误代码 %d: %s", res.StatusCode(), res.String())
	}

	if responseData == nil {
		return nil, errors.New("响应解析失败")
	}

	// 记录必要的响应信息
	log.Printf("请求成功, RequestID: %s, 使用的token: %d\n",
		responseData.RequestId, responseData.Usage.TotalTokens)

	// 在收到响应后追加“assistant”消息
	c.contextMutex.Lock()
	for _, choice := range responseData.Choices {
		c.chatContext = append(c.chatContext, GLM4VoiceMessage{
			Role:  "assistant",
			Audio: GLM4VoiceMessageAudio{Id: choice.Message.Audio.Id},
		})
	}
	c.contextMutex.Unlock()
	return responseData, nil
}

func (c *GLM4VoiceModelClient) Close() {
	if c.client != nil {
		c.client.Close()
	}
}

// 为了保持向后兼容，提供一个简化版Chat方法
// func (c *GLM4VoiceModelClient) ChatCompat(questionData *GLM4VoiceRequestData) *GLM4VoiceResponseData {
// 	resp, err := c.Chat(questionData)
// 	if err != nil {
// 		log.Printf("聊天请求失败: %v\n", err)
// 		return nil
// 	}
// 	return resp
// }


// Faker 用于测试，从文件读取模拟响应
// TODO: 将此函数移动到测试文件中
func Faker(data GLM4VoiceRequestData) *GLM4VoiceResponseData {
	// 打开 JSON 文件
	file, err := os.Open("result.json")
	if err != nil {
		fmt.Println("打开文件失败:", err)
		return nil
	}
	defer file.Close()

	decoder := json.NewDecoder(file)
	// 创建一个 Person 对象用于存储解析后的数据
	var respdata *GLM4VoiceResponseData

	// 创建一个解码器用于解析 JSON 数据
	err = decoder.Decode(&respdata)
	if err != nil {
		fmt.Println("解析 JSON 失败:", err)
		return nil
	}
	return respdata
}
