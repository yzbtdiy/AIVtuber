package bilibili

import (
	"crypto/hmac"
	"crypto/md5"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"time"

	"resty.dev/v3"
)

const (
	AcceptHeader              = "Accept"
	ContentTypeHeader         = "Content-Type"
	AuthorizationHeader       = "Authorization"
	JsonType                  = "application/json"
	BiliVersion               = "1.0"
	HmacSha256                = "HMAC-SHA256"
	BiliTimestampHeader       = "x-bili-timestamp"
	BiliSignatureMethodHeader = "x-bili-signature-method"
	BiliSignatureNonceHeader  = "x-bili-signature-nonce"
	BiliAccessKeyIdHeader     = "x-bili-accesskeyid"
	BiliSignVersionHeader     = "x-bili-signature-version"
	BiliContentMD5Header      = "x-bili-content-md5"
)

// APIClient API请求客户端
type APIClient struct {
	AccessKey       string
	AccessKeySecret string
	APIHost         string
	httpClient      *resty.Client
}

// NewAPIClient 创建新的API客户端
func NewAPIClient(accessKey, accessKeySecret, apiHost string) *APIClient {
	return &APIClient{
		AccessKey:       accessKey,
		AccessKeySecret: accessKeySecret,
		APIHost:         apiHost,
		httpClient:      resty.New(),
	}
}

// Request 发送HTTP请求
func (c *APIClient) Request(reqJson, requestUrl string) (resp BaseResp, err error) {
	resp = BaseResp{}
	header := NewCommonHeader(c.AccessKey, reqJson)
	header.Authorization = header.CreateSignature(c.AccessKeySecret)

	cliResp, err := c.httpClient.R().
		SetHeaders(header.ToMap()).
		SetBody(reqJson).
		SetResult(&resp).
		Post(fmt.Sprintf("%s%s", c.APIHost, requestUrl))

	if cliResp.IsError() && err != nil {
		return resp, fmt.Errorf("API请求失败: %w", err)
	}
	return
}

// Close 关闭API客户端
func (c *APIClient) Close() {
	c.httpClient.Close()
}

// NewCommonHeader 创建新的HTTP请求头
func NewCommonHeader(accessKey, content string) *CommonHeader {
	return &CommonHeader{
		ContentType:       JsonType,
		ContentAcceptType: JsonType,
		Timestamp:         strconv.FormatInt(time.Now().Unix(), 10),
		SignatureMethod:   HmacSha256,
		SignatureVersion:  BiliVersion,
		Authorization:     "",
		Nonce:             strconv.FormatInt(time.Now().UnixNano(), 10),
		AccessKeyId:       accessKey,
		ContentMD5:        Md5(content),
	}
}

// CreateSignature 生成Authorization加密串
func (h *CommonHeader) CreateSignature(accessKeySecret string) string {
	sStr := h.ToSortedString()
	return HmacSHA256(accessKeySecret, sStr)
}

// ToMap 所有字段转map<string, string>
func (h *CommonHeader) ToMap() map[string]string {
	return map[string]string{
		BiliTimestampHeader:       h.Timestamp,
		BiliSignatureMethodHeader: h.SignatureMethod,
		BiliSignatureNonceHeader:  h.Nonce,
		BiliAccessKeyIdHeader:     h.AccessKeyId,
		BiliSignVersionHeader:     h.SignatureVersion,
		BiliContentMD5Header:      h.ContentMD5,
		AuthorizationHeader:       h.Authorization,
		ContentTypeHeader:         h.ContentType,
		AcceptHeader:              h.ContentAcceptType,
	}
}

// ToSortMap 参与加密的字段转map<string, string>
func (h *CommonHeader) ToSortMap() map[string]string {
	return map[string]string{
		BiliTimestampHeader:       h.Timestamp,
		BiliSignatureMethodHeader: h.SignatureMethod,
		BiliSignatureNonceHeader:  h.Nonce,
		BiliAccessKeyIdHeader:     h.AccessKeyId,
		BiliSignVersionHeader:     h.SignatureVersion,
		BiliContentMD5Header:      h.ContentMD5,
	}
}

// ToSortedString 生成需要加密的文本
func (h *CommonHeader) ToSortedString() (sign string) {
	hMap := h.ToSortMap()
	var hSil []string
	for k := range hMap {
		hSil = append(hSil, k)
	}
	sort.Strings(hSil)
	for _, v := range hSil {
		sign += v + ":" + hMap[v] + "\n"
	}
	sign = strings.TrimRight(sign, "\n")
	return
}

// 以下是工具函数
// Md5 md5加密
func Md5(str string) (md5str string) {
	data := []byte(str)
	has := md5.Sum(data)
	md5str = fmt.Sprintf("%x", has)
	return md5str
}

// HmacSHA256 HMAC-SHA256算法
func HmacSHA256(key string, data string) string {
	mac := hmac.New(sha256.New, []byte(key))
	mac.Write([]byte(data))
	return hex.EncodeToString(mac.Sum(nil))
}
