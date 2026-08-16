# API Router v0.1.1 · 补丁版（Windows x64 Only）

> 🪟 **Windows 10/11 x64** · Patch Release  
> 本次为补丁版：修复 Token 统计监控不计数问题，并完成 Rust 代码质量整改。  
> 🍎 **macOS** — 🏃 Build scripts ready (requires a Mac host: `npm run tauri:build`)  
> 🐧 **Linux** — 📝 Planned (post v1.0.0)

---

## 🐛 本次修复 · What's Fixed

### 1. Token 统计监控不计数（核心修复）

此前流式请求（默认路径）的 Token 用量只在响应流**完全结束**时才记录；当客户端（Claude Code / Cursor / Cline 等）在流中途断开连接时，Token 统计会永久丢失，监控页 Token 列恒为 0。

本次修复：

- **Token usage「到包即记」**：上游的 usage 包到达时立即记录，不再依赖客户端把流读完；
- **自动补齐 `include_usage`**：客户端传 `stream_options: {}` 时自动注入 `include_usage: true`，确保上游返回 usage；
- **SSE 解析加固**：兼容 `data:`（无空格）前缀行；`"usage": null` 不再被误计为 0 token。

### 2. 代码质量整改

- `cargo clippy --lib` **14 个警告全部清零**（clamp 改写、derive Default、sort_by_key、冗余闭包、needless borrow 等），所有修改均为行为保持的等价改写。
- 新增 4 个针对 Token 统计与 SSE 解析的单元测试；`cargo test --lib` 99 passed / 0 failed。

---

## 🚀 核心特性回顾（v0.1.0-alpha 起）

- **本地 OpenAI 兼容端点** `http://127.0.0.1:6123`
  - `GET  /v1/models` · `POST /v1/chat/completions`（流式/非流式）· `POST /v1/completions` · `POST /v1/embeddings`
- **9+ 供应商开箱即用**：OpenAI · Anthropic · Google Gemini · Azure OpenAI · DeepSeek · 智谱 · 商汤日日新 · OpenRouter · 任意自定义 OpenAI-compatible Base URL
- **一键 Fetch Models** / 连通性健康检查 / 按供应商禁用系统代理（直连）
- **3 种路由策略**：层级回退 Priority · 加权 Weighted · 轮询 Round-Robin · 模型分组与多层级 fallback
- **高可靠栈**：指数退避重试 · 429/5xx/网络错误自动切后端 · 熔断器 · 单供应商 QPS + 并发限流 · 后台健康检查自动恢复
- **零拷贝 SSE 流式透传**（字节级转发，不缓存整段响应）
- **本地优先 & 默认隐私**：API Key 存入 Windows Credential Manager（密钥环），配置 YAML 永不含明文密钥；**零遥测**
- **监控看板**：服务状态 · 模型数 · 24h 请求量 · 延迟分布 · 按供应商/模型统计请求数、成功率、**Token 用量（本次修复）**
- **请求日志**：实时流 · 按供应商/模型/状态过滤 · 全文搜索 · CSV 导出 · 详情展开
- **系统设置**：监听端口 · 管理 Token · 超时/熔断阈值 · 缓存 · 明暗主题 · 关于

---

## 📦 Downloads · 安装包下载

| Platform | Arch | Format | Filename | Size | SHA-256 |
|:--------:|:----:|:------:|:---------|:----:|:--------|
| 🪟 Windows 10/11 | x64 | **NSIS .exe（推荐）** | `API Router_0.1.1_x64-setup.exe` | 4.14 MB | `BA980008D4D1872C89930706A5314E2477E1660ECFB739630A7F06E12E6C9C56` |
| 🪟 Windows 10/11 | x64 | WiX .msi | `API Router_0.1.1_x64_en-US.msi` | 5.99 MB | `E8F8BD016ED5C26439136AD7DBEE07FD21240DB65D17EFD11A08B744F159AA6E` |

### 校验方法（PowerShell）

```powershell
Get-FileHash "API Router_0.1.1_x64-setup.exe" -Algorithm SHA256
Get-FileHash "API Router_0.1.1_x64_en-US.msi"   -Algorithm SHA256
```

### 系统要求

- **OS**：Windows 10 21H2+ / Windows 11（x64）
- **WebView2**：系统已预安装；若缺少，安装过程会自动提示
- **升级方式**：直接覆盖安装即可（自动保留 `%APPDATA%\com.api-router.app` 下的配置）

---

## 🔧 快速上手 · Quick Start

1. 下载 `.exe` 或 `.msi` 安装并启动。
2. 打开侧边栏 **供应商** → 新增一个供应商（例如 OpenAI / DeepSeek），填入 Base URL + API Key，保存。
3. 点 **🔍 获取模型**，勾选想要映射的模型后保存 — 系统会自动为每个模型生成本地映射。
4. 切换到 **概览** 启动本地服务，或直接使用默认端点：

```bash
curl http://127.0.0.1:6123/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Hello, API Router!"}],
    "stream": false
  }'
```

5. 打开 **监控统计** 查看实时成功率、延迟分布与 Token 用量 📊。

---

## 🛣️ Roadmap

| Milestone | Target | Key Features |
|:---------:|:------:|:-------------|
| M1 ✅ | v0.1.x（本版本线） | Windows 版 + 核心路由 + 6 页 UI + 文档/Release |
| M2 🏃 | 2026 Q3 · v0.9-beta | macOS Universal 2 构建 · 一键迁移配置 · Tauri Updater |
| M3 📝 | 2026 Q4 · v1.0.0 | Linux deb/rpm/AppImage · LDAP/RBAC · 集群多节点 · CSP 收紧 |

---

## 🆘 反馈 & 社区

- 遇到 Bug 或有功能建议：👉 [GitHub Issues](https://github.com/xf2214/api-router/issues/new)
- 喜欢项目记得 **Star ⭐** 支持一下，这是我们迭代最大的动力！

---

*Release generated at 2026-08-16 · Tag `v0.1.1`*
