# API Router v0.2.0 · Minor Release（Windows + macOS）

> 🪟 **Windows 10/11 x64** · 🍎 **macOS 12+ Universal 2**  
> 本次为小版本更新：CI/Release 流水线完备、健康检查并发化、路由策略重构与监控体验优化。

---

## ✨ 新增 · What's New

### 🚦 GitHub Actions 自动化发布

- 推送 `v*` 标签自动构建 **Windows（NSIS + MSI）** 与 **macOS（Universal 2 DMG）** 安装包，生成 Release 并上传产物。
- CI 全面升级：Rust fmt + clippy（`-D warnings`）、**三平台** `cargo test`、前端 type-check + build、**密钥扫描**门禁、Tauri 三平台构建冒烟。

### ⚡ 健康检查并发化

- 全量提供商探活改为并发执行（最多 8 路在途），结果按原顺序返回 —— 供应商数量多时启动 / 刷新显著提速。
- 探活复用连接池（尊重系统代理与 keepalive 配置），超时统一走 `provider_timeout` 策略；请求头 `User-Agent` 改为 `api-router/<版本号>` 自动生成。

### 🧪 测试与工程规范

- 新增 **Vitest 前端单元测试**（`tests/` + `vitest.config.ts`），关键工具函数已有 25 个回归用例。
- 新增 ESLint 9 + Prettier 工程配置。

### 🎨 使用体验

- **日志增强**：支持 `RUST_LOG` 环境变量控制日志级别（默认 `info`）。
- **启动容错**：分组 / 模型定义 / 服务状态 / 健康检查并行加载，单项失败不再阻塞启动。
- **监控自动刷新优化**：页面隐藏自动暂停轮询、恢复可见自动续刷，刷新间隔可配置。

---

## 🔧 变更 · What's Changed

- **路由策略重构**：顺序优先 / 轮询 / 权重 / 最少占用 / 延迟优先收敛到统一选路实现，分组与模型目标共用同一逻辑。
- **配置写入优化**：改为「克隆-修改-替换」，磁盘 IO 期间不再持有写锁。
- **依赖升级**：`rand` 0.8 → 0.9；`tokio` 全量 feature 裁剪为按需开启。
- **发布体积优化**：Release profile 启用 `lto + opt-level z + strip`，安装包进一步瘦身。
- **组件重组**：路由树组件迁移至 `src/components/routing-tree/`。

---

## ✅ 修复 · What's Fixed

- 移除冗余的 `once_cell` / `thread_rng` 等过时用法，全面适配 rand 0.9 API。
- 健康检查 / 连通性测试超时统一为 `provider_timeout` 策略（原实现各有独立上限）。

---

## 📦 Downloads · 安装包下载

> 🛠 安装包由 GitHub Actions 针对 `v0.2.0` 标签自动构建，请在上方 **Assets** 中选择对应平台下载：

| Platform | Arch | Format | Filename |
|:--------:|:----:|:------:|:---------|
| 🪟 Windows 10/11 | x64 | NSIS .exe（推荐） | `API Router_0.2.0_x64-setup.exe` |
| 🪟 Windows 10/11 | x64 | WiX .msi | `API Router_0.2.0_x64_en-US.msi` |
| 🍎 macOS 12+ | Universal 2 | DMG | `API Router_0.2.0_aarch64.dmg` 等 |

### 系统要求

- **OS**：Windows 10 21H2+ / Windows 11（x64）；macOS 12+
- **WebView2**：Windows 系统已预安装；若缺少，安装过程会自动提示
- **升级方式**：直接覆盖安装即可（自动保留 `%APPDATA%\com.api-router.app` 下的配置）

---

## 🔧 快速上手 · Quick Start

1. 下载安装包并启动。
2. 打开侧边栏 **供应商** → 新增供应商（例如 OpenAI / DeepSeek），填入 Base URL + API Key。
3. 点 **🔍 获取模型**，勾选模型保存 — 自动为每个模型生成本地映射。
4. 直接调用本地端点：

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
| M1 ✅ | v0.1.x | Windows 版 + 核心路由 + 6 页 UI + 文档/Release |
| M2 🏃 | 2026 Q3 · v0.9-beta | macOS 发布线正式启用（本次已 CI 构建）· Tauri Updater |
| M3 📝 | 2026 Q4 · v1.0.0 | Linux deb/rpm/AppImage · 配置迁移 · CSP 收紧 |

---

## 🆘 反馈 & 社区

- 遇到 Bug 或有功能建议：👉 [GitHub Issues](https://github.com/xf2214/api-router/issues/new)
- 喜欢项目记得 **Star ⭐** 支持一下，这是我们迭代最大的动力！

---

*Release generated at 2026-09-02 · Tag `v0.2.0`*