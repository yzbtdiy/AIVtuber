import { invoke } from '@tauri-apps/api/core';

// OpenAI API 相关类型
export interface ChatResponse {
    success: boolean;
    message: string;
    content?: string;
}

// 整合对话和TTS的响应类型
export interface ChatAndSpeakResponse {
    success: boolean;
    message: string;
    chat_content?: string;
    audio_data?: string; // base64编码的音频数据
}

// TTS响应类型
export interface TtsResponse {
    success: boolean;
    message: string;
    audio_data?: string; // base64编码的音频数据
}

// 工具函数：将base64字符串转换为ArrayBuffer
export function base64ToArrayBuffer(base64: string): ArrayBuffer {
    // 移除可能的 data URL 前缀
    const base64Data = base64.replace(/^data:audio\/[^;]+;base64,/, '');
    // 解码base64字符串
    const binaryString = atob(base64Data);
    const length = binaryString.length;
    const bytes = new Uint8Array(length);
    for (let i = 0; i < length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
}

/**
 * 调用 OpenAI API 进行对话
 * @param message 用户消息
 * @param apiKey API密钥
 * @param model 模型名称，默认为 "gpt-4o"
 * @param temperature 温度参数，默认为 0.7
 * @returns 对话响应
 */
export async function chatWithOpenAI(
    message: string,
    apiKey: string,
    model: string = 'gpt-4o',
    temperature: number = 0.7
): Promise<ChatResponse> {
    try {
        const response = await invoke<ChatResponse>('chat_with_openai', {
            message,
            apiKey,
            model,
            temperature,
        });
        return response;
    } catch (error) {
        console.error('调用 OpenAI API 失败:', error);
        return {
            success: false,
            message: error instanceof Error ? error.message : '未知错误',
            content: undefined,
        };
    }
}

/**
 * 文本转语音
 * @param text 要转换的文本
 * @param audioPaths 音频路径数组，可选
 * @returns TTS响应，包含base64编码的音频数据
 */
export async function textToSpeech(
    text: string,
    audioPaths?: string[]
): Promise<TtsResponse> {
    try {
        const response = await invoke<TtsResponse>('text_to_speech', {
            text,
            audioPaths,
        });
        return response;
    } catch (error) {
        console.error('TTS 请求失败:', error);
        return {
            success: false,
            message: error instanceof Error ? error.message : '未知错误',
            audio_data: undefined,
        };
    }
}

/**
 * 文本转语音并返回 ArrayBuffer
 * @param text 要转换的文本
 * @param audioPaths 音频路径数组，可选
 * @returns 音频数据的 ArrayBuffer，失败时返回 null
 */
export async function textToSpeechArrayBuffer(
    text: string,
    audioPaths?: string[]
): Promise<ArrayBuffer | null> {
    const response = await textToSpeech(text, audioPaths);

    if (response.success && response.audio_data) {
        try {
            return base64ToArrayBuffer(response.audio_data);
        } catch (error) {
            console.error('转换 base64 到 ArrayBuffer 失败:', error);
            return null;
        }
    }

    return null;
}

/**
 * 组合使用：通过 OpenAI 生成回复并转换为语音（整合版本）
 * 这个函数在后端完成整个流程，减少前后端通信次数
 * API key 和其他配置从 config.json 文件中读取
 * @param userMessage 用户消息
 * @returns 包含文本回复和音频数据的对象
 */
export async function chatAndSpeakIntegrated(
    userMessage: string
): Promise<ChatAndSpeakResponse> {
    try {
        const response = await invoke<ChatAndSpeakResponse>('chat_and_speak', {
            message: userMessage,
        });
        return response;
    } catch (error) {
        console.error('调用整合对话和TTS失败:', error);
        return {
            success: false,
            message: error instanceof Error ? error.message : '未知错误',
            chat_content: undefined,
            audio_data: undefined,
        };
    }
}

/**
 * 组合使用：通过 OpenAI 生成回复并转换为语音，返回 ArrayBuffer
 * API key 和其他配置从 config.json 文件中读取
 * @param userMessage 用户消息
 * @returns 包含文本回复和音频 ArrayBuffer 的对象
 */
export async function chatAndSpeakArrayBuffer(
    userMessage: string
): Promise<{
    success: boolean;
    message: string;
    chatContent?: string;
    audioBuffer?: ArrayBuffer;
}> {
    const response = await chatAndSpeakIntegrated(userMessage);

    let audioBuffer: ArrayBuffer | undefined;

    if (response.success && response.audio_data) {
        try {
            audioBuffer = base64ToArrayBuffer(response.audio_data);
        } catch (error) {
            console.error('转换 base64 到 ArrayBuffer 失败:', error);
        }
    }

    return {
        success: response.success,
        message: response.message,
        chatContent: response.chat_content,
        audioBuffer,
    };
}
