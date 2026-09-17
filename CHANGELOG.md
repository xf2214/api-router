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

## [0.4.0] - 2026-09-12

> 🪟 **Windows x64 + 🍎 macOS** · Minor Release
> 体验与可靠性打磨版：导入收尾、服务 watchdog、代码健康、六页面三态。

### Added / 新增
- **服务 watchdog 自动重启**：异常退出指数退避重启（1s→30s），连续 5 次熔断停试并上报；显式停机不触发。
- **六页面三态统一**：空状态 / 加载态 / 错误重试（Monitoring、Overview、Logs、Providers、Routing、RoutingTree、Settings 动作）。
- **单测补强**：toast 链 mount 测试、导入两态测试、clipboard 降级分支；vitest ≥35。

### Changed / 变更
- **导入体验**：成功改走 `getConfig()` 重水合，不再整页 reload（tab 不再重置）；新增 `settings.includeKeys` / `settings.importConfig` 中英 key。
- **Keyring 测试 hermetic**：`#[cfg(test)]` 内存后端，Linux 无 DBus 也可跑。
- **Stores 精确化**：4 文件 8 处 `any` 改精确类型。

### Fixed / 修复
- 复制端点 Toast 链断裂（OverviewPage/App 转发缺失）与 clipboard 降级泄漏（已在开发中修复）。
- 监控 CTA 硬编码文案改走既有 i18n key。

---

## [0.3.0] - 2026-09-04

> 🪟 **Windows x64 + 🍎 macOS** · Minor Release
> 小版本更新：配置导入导出与备份、一键复制端点、Mock 上游集成测试、依赖保守更新与门禁清零。

### Added / 新增
- **配置导入/导出与备份**：导入前自动写时间戳备份；密钥永不进导出文件（掩码 `***`），导入后提示补填。
- **一键复制本地端点**：Overview 面板端点旁复制按钮（clipboard + 降级），成功/失败 Toast。
- **Mock 上游集成测试**：零新依赖，axum 自建 mock 上游覆盖流式与非流式转发。

### Changed / 变更
- 前端依赖 wanted 级更新：plugin-shell 2.3.6、test-utils 2.5.0、typescript-eslint 8.69、vue 3.5.42；Rust 保守 `cargo update`（无 major 跳变）。
- `exportConfig` 支持 `includeKeys` 参数（含密钥/掩码两种导出）。

### Fixed / 修复
- 清零 2 个 eslint errors（`prefer-const`）与 `cargo fmt` 1 处 diff；clippy `-D warnings` 全绿。

---

## [0.2.0] - 2026-09-02

> 🪟 **Windows x64 + 🍎 macOS** · Minor Release  
> 主版本小更新：健壮性增强、CI/Release 流水线完备、路由与监控优化。

### Added / 新增

- **并发健康检查**：全量提供商探活改为并发执行（Semaphore 限流，最多 8 路在途），结果按原顺序返回；大量提供商时启动/刷新显著提速。
- **前端单元测试体系**：引入 Vitest + @vue/test-utils（`tests/` + `vitest.config.ts`），为关键 composables / 工具函数建立回归防护。
- **工程规范落地**：新增 ESLint 9 + Prettier 配置（`eslint.config.js` / `.prettierrc.json`）。
- **日志增强**：新增 `tracing-subscriber` 全局日志初始化，支持 `RUST_LOG` 环境变量控制（默认 `info`）。
- **应用启动容错**：启动时并行加载分组 / 模型定义 / 服务状态 / 健康检查，单项失败不再阻塞整体初始化，并给出明确提示。
- **监控自动刷新优化**：页面隐藏（`document.hidden`）时暂停轮询，恢复可见自动续刷；刷新间隔可配置。
- **国际化补全**：提供商列表新增「直连 / 代理」短标签；模型未找到提示支持 i18n。
- **GitHub Actions Release 工作流**：推送 `v*` tag 自动构建 Windows（NSIS/MSI）与 macOS 安装包，生成 Draft Release 并上传产物。
- **CI 全面升级**：Rust fmt + clippy（`-D warnings`）、三平台 `cargo test`、前端 type-check + build、密钥扫描（gitleaks 风格脚本）、Tauri 三平台构建冒烟。

### Changed / 变更

- **路由策略重构**：顺序优先 / 轮询 / 权重 / 最少占用 / 延迟优先统一收敛到共享选路实现（`pick_candidate_index`），分组与模型目标共用同一套逻辑。
- **健康检查复用连接池**：探活与连通性测试改用 `ClientRegistry` 连接池（尊重系统代理与 keepalive 配置），超时走统一 `provider_timeout` 策略。
- **User-Agent 版本化**：请求头 `User-Agent` 改为 `api-router/<版本号>`，随 `CARGO_PKG_VERSION` 自动生成。
- **配置写入优化**：删除 / 保存配置改为「克隆-修改-替换」，磁盘 IO 期间不再持有写锁。
- **依赖升级**：`rand` 0.8 → 0.9；`tokio` 全量 feature 裁剪为按需开启。
- **发布体积优化**：Release profile 启用 `lto + opt-level z + strip + codegen-units=1`。
- **组件重组**：路由树相关组件迁移至 `src/components/routing-tree/`，删除顶层重复的旧组件。

### Fixed / 修复

- 移除冗余的 `once_cell`、`thread_rng` 等过时用法，全面适配 rand 0.9 API。
- 健康检查 / 测试连通性超时取值统一为 `provider_timeout` 策略（原实现各有独立上限）。

### Security / 安全

- 密钥扫描脚本增强，纳入更多高熵/凭据特征，作为 CI 强制门禁之一。

---

### 下载矩阵 · Download Matrix (v0.2.0)

> 🛠 由 GitHub Actions 在 tag 推送后自动构建，产物见对应 Release 页面。

| Platform / 平台 | Arch / 架构 | Format / 格式 | Status / 状态 |
|:---------------:|:-----------:|:-------------:|:-------------:|
| 🪟 **Windows 10/11** | x64 | NSIS exe + WiX MSI | ✅ CI 构建 |
| 🍎 macOS 12+ | Universal 2 | DMG / .app | ✅ CI 构建 |
| 🐧 Linux | x64 | deb / rpm / AppImage | 📝 规划中 |

## [0.1.1] - 2026-08-16

> 🪟 **Windows x64 Only** · Patch Release  
> 补丁版：修复 Token 统计不计数问题，并完成 Rust 代码质量整改。

### Fixed / 修复

- **修复 Token 统计监控不计数**：
  - 流式请求的 token usage 改为「到包即记」— 上游 usage 包到达时立即记录，不再依赖客户端完整消费 SSE 流（此前 Claude Code / Cursor 等客户端中断连接会导致 usage 丢失，token 统计恒为 0）。
  - 客户端传 `stream_options: {}` 时自动补 `include_usage: true`，确保上游返回 usage 可被统计。
  - SSE 解析兼容 `data:`（无空格）前缀行；`"usage": null` 不再被误计为 0 token。
- **代码质量整改**：`cargo clippy --lib` 14 个警告全部清零（clamp 改写、derive Default、sort_by_key、冗余闭包、needless borrow 等），行为保持不变。

### Changed / 变更

- 无。

### Added / 新增

- 新增 4 个针对 token 统计与 SSE 解析的单元测试（`cargo test --lib` 99 passed / 0 failed）。

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

### 安装包下载 · Download Matrix (v0.1.1)

| Platform / 平台 | Arch / 架构 | Format / 格式 | Filename / 文件名 | SHA-256 |
|:---------------:|:-----------:|:-------------:|:-----------------|:--------|
| 🪟 **Windows 10/11** | x64 | **NSIS exe (推荐)** | `API Router_0.1.1_x64-setup.exe` | `BA980008D4D1872C89930706A5314E2477E1660ECFB739630A7F06E12E6C9C56` |
| 🪟 **Windows 10/11** | x64 | WiX MSI | `API Router_0.1.1_x64_en-US.msi` | `E8F8BD016ED5C26439136AD7DBEE07FD21240DB65D17EFD11A08B744F159AA6E` |
| 🍎 macOS 12+ | Universal 2 | DMG / .app | *(build from source)* | 🏃 M2 |
| 🐧 Linux | x64 | deb / rpm / AppImage | *(not yet)* | 📝 M3 |

---

[Unreleased]: https://github.com/xf2214/api-router/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/xf2214/api-router/releases/tag/v0.4.0
[0.3.0]: https://github.com/xf2214/api-router/releases/tag/v0.3.0
[0.2.0]: https://github.com/xf2214/api-router/releases/tag/v0.2.0
[0.1.1]: https://github.com/xf2214/api-router/releases/tag/v0.1.1
[0.1.0]: https://github.com/xf2214/api-router/releases/tag/v0.1.0
