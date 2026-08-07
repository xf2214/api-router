# Changelog / 变更日志

All notable changes to **API Router** will be documented in this file.  
所有**值得关注的变更**都会在此文件中记录。

The format follows [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/lang/zh-CN/).  
格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)。

Release status legend:  / 发布状态图例：

| Badge | Meaning / 含义 |
|-------|----------------|
| 🪟 | Windows x64 (MSI + NSIS) released / Windows x64 已发布 |
| 🍎 | macOS released / macOS 已发布 |
| 🐧 | Linux released / Linux 已发布 |
| 🏃 | Work in progress / In progress 开发中 |
| 📝 | Planned / 计划中 |

---

## [Unreleased] / [未发布]

### Added / 新增
- Placeholder — to be filled in the next iteration.

### Changed / 变更
- None.

### Fixed / 修复
- None.

---

## [0.1.0] - 2026-08-07

> 🪟 **Windows x64 Only** · Alpha Release · First public preview  
> 🍎 macOS — 🏃 In progress (M2 milestone, Week 5)  
> 🐧 Linux — 📝 Planned (post-M3)

### ⚠️ Known Limitations / 已知限制

1. **Windows-only installer.** macOS / Linux builds are not available as binary downloads yet. macOS requires a Mac host to build via `npm run tauri:build`.
   **仅提供 Windows 安装包。** macOS / Linux 暂不提供可下载安装包。macOS 用户需使用 Mac 主机自行执行 `npm run tauri:build` 构建。
2. **Alpha quality.** Core local-server flow is functional, but edge cases around networking, proxy, and multi-threaded burst traffic have not been fully audited.
   **Alpha 质量。** 核心本地服务链路可用，但网络代理、多线程突刺流量等边界情况尚未完全审计。
3. **No Tauri Updater yet.** Auto-update is scheduled for the v1.0.0 (M3) release; upgrades must be applied manually (over-install).
   **暂未启用 Tauri 自动更新。** 自动更新计划在 v1.0.0（M3）上线；目前请手动覆盖安装升级。

### Added / 新增

- **Unified local OpenAI-compatible endpoint** (`http://127.0.0.1:6123`) exposing:
  - 统一本地 OpenAI 兼容端点，暴露以下接口：
  - `GET  /v1/models` — list available locally-mapped models / 列出本地已映射的模型
  - `POST /v1/chat/completions` — streaming + non-streaming chat completion / 流式与非流式对话
  - `POST /v1/completions` — legacy completions / 传统补全
  - `POST /v1/embeddings` — vector embeddings / 向量嵌入
- **Provider management UI** — add / edit / delete / toggle providers; supports:
  - 提供商管理 UI — 增删改查 + 启停，支持：
  - OpenAI · Anthropic · Google Gemini · Azure OpenAI · DeepSeek · 智谱 (Zhipu) · 商汤日日新 (SenseNova) · OpenRouter · any custom OpenAI-compatible Base URL / 任意自定义 OpenAI 兼容 Base URL
  - One-click 🔍 "Fetch Models" via provider's `/models` endpoint / 一键通过提供商 `/models` 接口拉取模型列表
  - Per-provider connectivity health check / 单提供商连通性健康检查
- **Smart model routing strategies** / 智能路由策略：
  - **Priority (层级回退)** — fail through tiers 1 → 2 → 3 on errors
  - **Weighted (加权分配)** — split traffic by configurable weight ratio
  - **Round-Robin (轮询)** — rotate across backends evenly
  - Supports per-model groups and tiered fallback / 支持按模型分组 + 多层级回退
- **High reliability stack** / 高可靠性基础能力：
  - Automatic retry with exponential backoff / 指数退避自动重试
  - Fallback handover to next backend on 429/5xx/network error / 429/5xx/网络错误自动切下一个后端
  - Circuit breaker (consecutive-failure → cool-down) / 熔断器（连续失败 → 冷却期跳过）
  - Per-provider QPS + concurrency throttling / 每提供商 QPS 与并发限流
  - Periodic health check with auto-recovery / 后台健康检查 + 自动恢复
- **Zero-copy SSE streaming passthrough** — byte-level forward without buffering full response in memory / 零拷贝 SSE 流式透传 — 字节级转发，不会在内存中缓存完整响应。
- **Local-first & privacy by default** / 本地优先 & 默认隐私：
  - API keys stored in OS native keyring (Windows Credential Manager / macOS Keychain / Linux Secret Service) / API Key 存储在操作系统原生密钥环
  - Config YAML never contains plaintext keys / 配置 YAML 绝不含明文密钥
  - Zero telemetry — no stats or keys are sent anywhere except your configured providers / 零遥测，除你配置的提供商外不向任何地方发送数据
- **Monitoring dashboard** / 监控看板：
  - Service status + configured model count + 24h request volume + latency distribution / 服务状态 · 已配置模型数 · 24h 请求量 · 延迟分布
  - Per-provider & per-model request count / success rate / token usage / 按提供商和按模型的请求数 · 成功率 · Token 用量
- **Request logs browser** / 请求日志浏览器：
  - Live feed · filter by provider/model/status · text search · CSV export · full detail view / 实时流 · 按提供商/模型/状态过滤 · 全文搜索 · CSV 导出 · 详情展开
- **System settings page** / 系统设置：
  - Listen port · admin API token · connect timeouts · circuit-breaker thresholds · cache size · theme (light/dark) · About / 监听端口 · 管理 Token · 连接超时 · 熔断阈值 · 缓存大小 · 明暗主题 · 关于

### Changed / 变更

- Initial release; nothing to compare against yet.
  首次发布，暂无可对比项。

### Deprecated / 废弃

- None. 无。

### Removed / 移除

- None. 无。

### Fixed / 修复

- Initial release; prior bug list was empty.
  首次发布，修复列表为空。

### Security / 安全

- API Key plaintext is never persisted to disk — only OS keyring entries are kept.
  API Key 明文永不落盘 — 仅通过操作系统密钥环保存。
- CSP set to null for development; production bundles will ship with a tightened CSP before v1.0.0.
  开发版 CSP 设为 null；v1.0.0 前生产包将启用收紧的 CSP。

---

### 安装包下载 · Download Matrix (v0.1.0)

| Platform / 平台 | Arch / 架构 | Format / 格式 | Filename / 文件名 | SHA-256 |
|:---------------:|:-----------:|:-------------:|:-----------------|:--------|
| 🪟 **Windows 10/11** | x64 | **NSIS exe (推荐)** | `API Router_0.1.0_x64-setup.exe` | *(see GitHub Release)* |
| 🪟 **Windows 10/11** | x64 | WiX MSI | `API Router_0.1.0_x64_en-US.msi` | *(see GitHub Release)* |
| 🍎 macOS 12+ | Universal 2 | DMG / .app | *(build from source)* | 🏃 M2 |
| 🐧 Linux | x64 | deb / rpm / AppImage | *(not yet)* | 📝 M3 |

---

[Unreleased]: https://github.com/api-router/api-router/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/api-router/api-router/releases/tag/v0.1.0
