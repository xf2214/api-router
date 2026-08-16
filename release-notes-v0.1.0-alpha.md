# API Router v0.1.0-alpha · 首次公开发布（Windows x64 Only）

> 🪟 **Windows 10/11 x64** · Alpha Preview · First Public Release  
> 🍎 **macOS** — 🏃 Build scripts ready (requires a Mac host: `npm run tauri:build`)  
> 🐧 **Linux** — 📝 Planned (post v1.0.0)

---

## ⚠️ Known Limitations · 已知限制

1. **仅提供 Windows 安装包。** macOS / Linux 用户需在对应主机上执行 `npm run tauri:build` 自行构建。
2. **Alpha 质量。** 本地 HTTP 服务与路由核心链路可用，但网络代理、多线程突刺流量、极端错误码等边界情况尚未完整审计。
3. **未启用 Tauri 自动更新。** 请手动覆盖安装进行升级（v1.0.0 前启用 updater）。

---

## 🚀 Highlights · 核心特性

- **本地 OpenAI 兼容端点** `http://127.0.0.1:6123`
  - `GET  /v1/models` · 列出本地已映射模型
  - `POST /v1/chat/completions` · 流式 / 非流式 chat
  - `POST /v1/completions` · 传统补全
  - `POST /v1/embeddings` · 向量嵌入
- **9+ 供应商开箱即用**：OpenAI · Anthropic · Google Gemini · Azure OpenAI · DeepSeek · 智谱 · 商汤日日新 · OpenRouter · 任意自定义 OpenAI-compatible Base URL
- **一键 Fetch Models** / 连通性健康检查 / 按供应商禁用系统代理（直连）
- **3 种路由策略**：层级回退 Priority · 加权 Weighted · 轮询 Round-Robin · 支持模型分组与多层级 fallback
- **高可靠栈**：指数退避重试 · 429/5xx/网络错误自动切后端 · 熔断器 · 单供应商 QPS + 并发限流 · 后台健康检查自动恢复
- **零拷贝 SSE 流式透传**（字节级转发，不缓存整段响应）
- **本地优先 & 默认隐私**：API Key 存入 Windows Credential Manager（密钥环），配置 YAML 永不含明文密钥；**零遥测**
- **监控看板**：服务状态 · 模型数 · 24h 请求量 · 延迟分布 · 按供应商 / 模型统计请求数、成功率、Token 用量
- **请求日志**：实时流 · 按供应商/模型/状态过滤 · 全文搜索 · CSV 导出 · 详情展开
- **系统设置**：监听端口 · 管理 Token · 超时/熔断阈值 · 缓存 · 明暗主题 · 关于

---

## 📦 Downloads · 安装包下载

| Platform | Arch | Format | Filename | Size | SHA-256 |
|:--------:|:----:|:------:|:---------|:----:|:--------|
| 🪟 Windows 10/11 | x64 | **NSIS .exe（推荐）** | `API Router_0.1.0_x64-setup.exe` | 5.96 MB | `BD9DBC3E60F2D34FB6ABE13E97710A1D644D417BA6B8D02EFC601236E6A79B52` |
| 🪟 Windows 10/11 | x64 | WiX .msi | `API Router_0.1.0_x64_en-US.msi` | 7.95 MB | `F4211C3CCCC76426D80885DA3E6BC6CD4464EA7DFF689BA6FB1AD16A38F81A94` |

### 校验方法（PowerShell）

```powershell
Get-FileHash "API Router_0.1.0_x64-setup.exe" -Algorithm SHA256
Get-FileHash "API Router_0.1.0_x64_en-US.msi"   -Algorithm SHA256
```

### 系统要求

- **OS**：Windows 10 21H2+ / Windows 11（x64）
- **WebView2**：系统已预安装；若缺少，安装过程会自动提示
- **磁盘**：安装后 ~60 MB（含前端打包 + Rust 后端 + 两个捆绑程序）
- **首次启动**：6123 端口默认未占用即可；如占用，启动后在"设置 → 监听端口"里切换

---

## 🔧 快速上手 · Quick Start

1. 下载 `.exe` 或 `.msi` 安装并启动。
2. 打开侧边栏 **供应商** → 新增一个供应商（例如 OpenAI / DeepSeek），填入 Base URL + API Key，保存。
3. 点 **🔍 获取模型**，勾选想要映射的模型后保存 — 系统会自动为每个模型生成本地映射（local_name = 上游名，路由=顺序优先）。
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
| M1 ✅ | v0.1.0-alpha（本版本） | Windows 版 + 核心路由 + 6 页 UI + 文档/Release 首次发布 |
| M2 🏃 | 2026 Q3 · v0.9-beta | macOS Universal 2 构建 · 一键迁移配置 · Tauri Updater |
| M3 📝 | 2026 Q4 · v1.0.0 | Linux deb/rpm/AppImage · LDAP/RBAC · 集群多节点 · CSP 收紧 |

---

## 🆘 反馈 & 社区

- 遇到 Bug 或有功能建议：👉 [GitHub Issues](https://github.com/xf2214/api-router/issues/new)
- 喜欢项目记得 **Star ⭐** 支持一下，这是我们迭代最大的动力！

---

*Release generated at 2026-08-07 · Commit `4fe8297` · Tag `v0.1.0-alpha`*
