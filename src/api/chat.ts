import { invoke } from '@tauri-apps/api/core';

// 类型定义
export interface ChatResponse {
    success: boolean;
    message: string;
    content?: string;
}

export interface ChatAndSpeakResponse {
    success: boolean;
    message: string;
    chat_content?: string;
    audio_data?: number[];
}

export interface TtsResponse {
    success: boolean;
    message: string;
    audio_data?: number[];
}

// 工具函数：将字节数组转换为ArrayBuffer
export function bytesToArrayBuffer(bytes: number[]): ArrayBuffer {
    return new Uint8Array(bytes).buffer;
}

// 工具函数：将字节数组转换为Blob
export function bytesToBlob(bytes: number[], mimeType: string = 'audio/wav'): Blob {
    return new Blob([new Uint8Array(bytes)], { type: mimeType });
}

/**
 * OpenAI 对话
 */
export async function chatWithOpenAI(
    message: string,
    apiKey: string,
    model: string = 'gpt-4o',
    temperature: number = 0.7
): Promise<ChatResponse> {
    try {
        return await invoke<ChatResponse>('chat_with_openai', {
            message,
            apiKey,
            model,
            temperature,
        });
    } catch (error) {
        return {
            success: false,
            message: error instanceof Error ? error.message : '未知错误',
        };
    }
}

/**
 * 文本转语音
 */
export async function textToSpeech(text: string): Promise<TtsResponse> {
    try {
        return await invoke<TtsResponse>('text_to_speech', { text });
    } catch (error) {
        return {
            success: false,
            message: error instanceof Error ? error.message : '未知错误',
        };
    }
}

/**
 * 对话 + TTS（推荐使用）
 * 后端集成处理，减少通信开销
 */
export async function chatAndSpeak(userMessage: string): Promise<ChatAndSpeakResponse> {
    try {
        return await invoke<ChatAndSpeakResponse>('chat_and_speak', {
            message: userMessage,
        });
    } catch (error) {
        return {
            success: false,
            message: error instanceof Error ? error.message : '未知错误',
        };
    }
}

/**
 * 播放音频字节数组
 */
export async function playAudioBytes(
    audioBytes: number[],
    volume: number = 1.0
): Promise<HTMLAudioElement> {
    const audioBlob = bytesToBlob(audioBytes);
    const audioUrl = URL.createObjectURL(audioBlob);
    
    const audio = new Audio(audioUrl);
    audio.volume = volume;
    
    return new Promise((resolve, reject) => {
        audio.onloadeddata = () => {
            audio.play()
                .then(() => resolve(audio))
                .catch(reject);
        };
        audio.onerror = reject;
    });
}
