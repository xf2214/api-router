# 统一本地 API 规范

## 1. 基础信息
- **Base URL**: `http://127.0.0.1:6123`（端口可在设置中修改）
- **协议**: HTTP/1.1 或 HTTP/2（取决于上游）
- **鉴权**: 本地服务默认不强制鉴权；开启后使用 `Authorization: Bearer <local-token>`
- **Content-Type**: `application/json`

## 2. 端点列表

### 2.1 获取可用模型
```http
GET /v1/models
```

**响应示例**
```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt-4o",
      "object": "model",
      "created": 1715367049,
      "owned_by": "openai"
    },
    {
      "id": "claude-3-5-sonnet",
      "object": "model",
      "owned_by": "anthropic"
    }
  ]
}
```

### 2.2 对话补全
```http
POST /v1/chat/completions
```

**请求体**（OpenAI 兼容）
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "stream": false,
  "temperature": 0.7,
  "max_tokens": 512
}
```

**非流式响应**
```json
{
  "id": "chatcmpl-local-xxx",
  "object": "chat.completion",
  "created": 1715367049,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 10,
    "total_tokens": 30
  }
}
```

**流式响应**（`stream: true`）
```text
Content-Type: text/event-stream

data: {"id":"chatcmpl-local-xxx","object":"chat.completion.chunk",...}

data: {"choices":[{"delta":{},"finish_reason":"stop"}],...}

data: [DONE]
```

### 2.3 Embeddings
```http
POST /v1/embeddings
```

**请求体**
```json
{
  "model": "text-embedding-3-small",
  "input": "Hello world"
}
```

### 2.4 Completions
```http
POST /v1/completions
```

**请求体**
```json
{
  "model": "gpt-4o",
  "prompt": "Once upon a time",
  "max_tokens": 100
}
```

## 3. 错误码

| HTTP 状态码 | 含义 | 说明 |
|------------|------|------|
| 200 | 成功 | 正常返回 |
| 400 | 请求参数错误 | 缺少 model、messages 等必要字段 |
| 401 | 鉴权失败 | Local Token 错误 |
| 404 | 模型未找到 | 本地模型别名未配置 |
| 429 | 请求过多 | 超出本地 QPS 限制 |
| 502 | 上游错误 | 所有后端均不可用 |
| 504 | 上游超时 | 后端响应超时 |

**错误响应格式**
```json
{
  "error": {
    "message": "Model 'gpt-4o' is not configured or all backends are down.",
    "type": "router_error",
    "code": "model_unavailable"
  }
}
```

## 4. 本地扩展头
调用本地 API 时，可附加以下头覆盖默认路由行为：

| Header | 说明 |
|--------|------|
| `X-Target-Provider` | 强制指定后端提供商 ID |
| `X-Retry-Count` | 覆盖全局重试次数 |
| `X-Stream-Mode` | `passthrough`（默认）或 `buffer` |

## 5. 与上游 OpenAI 的兼容性
- 请求体字段全部透传，不做裁剪。
- 响应字段原样返回。
- 仅 `model` 字段使用本地别名，实际转发时会映射为上游真实模型名。
