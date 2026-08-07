# 大模型 API 统一路由客户端 —— 技术架构文档

## 1. 技术选型

| 层级 | 选型 | 理由 |
|------|------|------|
| 桌面框架 | Tauri v2 | 基于 Rust，安装包极小（通常 < 15 MB），内存占用低，支持 Windows / macOS / Linux。 |
| 后端核心 | Rust + Tokio | 异步运行时稳定、性能高、内存安全，适合长期运行的本地服务。 |
| 前端 UI | Vue 3 / React 18 + TypeScript | 组件化开发，生态成熟， bundle 体积可控。 |
| 本地服务 | Axum / Actix-web | 轻量、异步、易于实现 SSE 流式转发。 |
| 配置存储 | JSON / YAML 文件 | 人类可读，便于备份与版本控制。 |
| 密钥存储 | Keyring (macOS Keychain / Windows Credential) | 操作系统原生加密，不落地明文。 |
| 请求/日志数据库 | SQLite (可选，默认关闭) | 零配置、单文件，便于本地查询历史与用量。 |
| 构建与发布 | GitHub Actions | 自动化 cross-compile、签名、notarize、生成更新包。 |

## 2. 总体架构

```
┌─────────────────────────────────────────────┐
│                 前端 UI (Tauri WebView)        │
│     提供商配置 / 模型映射 / 监控 / 日志查看      │
└──────────────────┬──────────────────────────┘
                   │  Tauri IPC / Events
┌──────────────────▼──────────────────────────┐
│              Tauri Rust Core                 │
│  配置管理 │ 密钥管理 │ 状态机 │ 自动升级       │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│            本地 HTTP 服务 (Axum)             │
│   /v1/models  /v1/chat/completions  ...      │
└──────────────────┬──────────────────────────┘
                   │  路由 / 转换 / 重试 / 限流
┌──────────────────▼──────────────────────────┐
│            上游大模型 API                    │
│   OpenAI / Anthropic / Gemini / Azure ...    │
└─────────────────────────────────────────────┘
```

## 3. 核心模块

### 3.1 配置管理模块 (config)
- 负责读取、校验、保存 `config.yaml`。
- 支持热重载：文件变更后自动刷新内存中的配置。
- 配置项：服务端口、提供商列表、模型映射、Fallback 策略、日志级别。

### 3.2 凭据管理模块 (keyring)
- 所有 API Key 通过操作系统凭据服务存储。
- 运行时通过句柄（provider id）获取明文，进程内缓存采用 `secrecy` 类型防止意外打印。

### 3.3 路由引擎 (router)
- 根据请求中的 `model` 字段匹配本地模型别名。
- 解析目标后端列表，按策略选择：
  - `priority`：顺序优先；
  - `weighted`：按权重随机；
  - `round_robin`：轮询。
- 失败时按 `max_retries` 与 `fallback_enabled` 切换后端。

### 3.4 协议转换层 (transform)
- 默认透传 OpenAI 格式请求与响应。
- 对非 OpenAI 兼容后端，通过配置中的 `request_template` 与 `response_template` 做字段映射。
- 流式响应仅做字节级透传，避免阻塞与内存暴涨。

### 3.5 本地 HTTP 服务 (server)
- 基于 Axum，监听 `127.0.0.1:<port>`。
- 路由实现：
  - `GET  /v1/models`
  - `POST /v1/chat/completions`
  - `POST /v1/completions`
  - `POST /v1/embeddings`
- 支持 CORS（默认仅本地），支持 Bearer Token 鉴权（可选）。

### 3.6 监控与日志 (telemetry)
- 使用 `tracing` 记录结构化日志，默认写入本地日志目录。
- 请求维度数据写入 SQLite（可选启用），用于统计面板。

## 4. 数据流

1. 用户在 UI 添加提供商并保存 → Rust 校验配置并写入文件，密钥写入钥匙串。
2. 本地服务启动 → 加载配置 → 构建模型路由表。
3. 客户端应用调用 `http://127.0.0.1:6123/v1/chat/completions`。
4. 服务解析 `model`，选择后端，替换 `Authorization` 头为真实密钥。
5. 转发请求到上游；流式响应通过 SSE 实时回传；非流式响应透传。
6. 记录请求耗时、状态码、Token 用量到本地日志/数据库。
7. UI 通过 Tauri 事件实时刷新监控面板。

## 5. 部署与打包

### 5.1 项目结构
```
api-router/
├── src-tauri/          # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── router.rs
│   │   ├── server.rs
│   │   ├── transform.rs
│   │   └── telemetry.rs
│   └── Cargo.toml
├── src/                # 前端源码
├── docs/               # 说明文档
├── scripts/            # 打包脚本
└── .github/workflows/  # CI/CD
```

### 5.2 构建流程
- `cargo tauri dev`：本地开发。
- `cargo tauri build`：生成 `.msi` / `.dmg` / `.app`。
- GitHub Actions 在 tag push 时触发多平台构建、签名、Notarize、发布 Release。

## 6. 关键设计约束
- 本地服务仅绑定回环地址，默认不暴露到局域网。
- 所有网络请求走系统代理（尊重用户代理设置）。
- 核心进程异常退出时，前端尝试自动重启本地服务并提示用户。
- 不引入外部运行时依赖（如 Java、.NET），确保“绝对轻量”。
