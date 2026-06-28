import { invoke } from '@tauri-apps/api/core';

export interface KeyValue {
    key: string;
    value: string;
    enabled?: boolean;
}

export interface WsEvent {
    connectionId: string;
    kind: string; // "open" | "message" | "close" | "error"
    data: string;
    ts: number;
}

/** 建立 WebSocket 连接, 返回 connectionId */
export async function wsConnect(
    url: string,
    headers: KeyValue[] = [],
    protocols: string[] = [],
): Promise<string> {
    return await invoke<string>('ws_connect', {
        url,
        headers: headers.length ? headers : null,
        protocols: protocols.length ? protocols : null,
    });
}

/** 发送消息 */
export async function wsSend(connectionId: string, message: string): Promise<void> {
    await invoke('ws_send', { connectionId, message });
}

/** 断开连接 */
export async function wsDisconnect(connectionId: string): Promise<void> {
    await invoke('ws_disconnect', { connectionId });
}
