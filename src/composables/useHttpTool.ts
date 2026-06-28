import { invoke } from '@tauri-apps/api/core';

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';
export type BodyType = 'json' | 'form' | 'raw';

export interface KeyValue {
    key: string;
    value: string;
    enabled?: boolean;
}

export interface HttpRequestInput {
    method: HttpMethod;
    url: string;
    headers: KeyValue[];
    query: KeyValue[];
    body?: string | null;
    bodyType?: BodyType | null;
    timeoutMs?: number | null;
}

export interface HttpResponseOutput {
    status: number;
    statusText: string;
    headers: KeyValue[];
    body: string;
    elapsedMs: number;
    sizeBytes: number;
    error: string | null;
}

/** 发起 HTTP 请求 */
export async function httpRequest(req: HttpRequestInput): Promise<HttpResponseOutput> {
    return await invoke<HttpResponseOutput>('http_request', {
        req: {
            method: req.method,
            url: req.url,
            headers: req.headers,
            query: req.query,
            body: req.body ?? null,
            bodyType: req.bodyType ?? null,
            timeoutMs: req.timeoutMs ?? null,
        },
    });
}
