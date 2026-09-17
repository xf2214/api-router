# API Router v0.3.0 · Minor Release（Windows + macOS）

> 🪟 **Windows 10/11 x64** · 🍎 **macOS 12+ Universal 2**  
> 小版本更新：配置导入导出与备份、一键复制端点、Mock 上游集成测试、依赖保守更新与门禁清零。

---

## ✨ 新增 · What's New

### 📦 配置导入/导出与备份

- 导入前自动写时间戳备份；密钥永不进导出文件（掩码 `***`），导入后提示补填。
- `exportConfig` 支持 `includeKeys` 参数（含密钥/掩码两种导出）。

### 📋 一键复制本地端点

- Overview 面板端点旁复制按钮（clipboard + 降级），成功/失败 Toast。

### 🧪 Mock 上游集成测试

- 零新依赖，axum 自建 mock 上游覆盖流式与非流式转发。

---

## 🔧 变更 · What's Changed

- 前端依赖 wanted 级更新：plugin-shell 2.3.6、test-utils 2.5.0、typescript-eslint 8.69、vue 3.5.42；Rust 保守 `cargo update`（无 major 跳变）。
- `exportConfig` 支持 `includeKeys` 参数（含密钥/掩码两种导出）。

---

## ✅ 修复 · What's Fixed

- 清零 2 个 eslint errors（`prefer-const`）与 `cargo fmt` 1 处 diff；clippy `-D warnings` 全绿。

---

## 📦 Downloads · 安装包下载

> 🛠 安装包由 GitHub Actions 针对 `v0.3.0` 标签自动构建，请在上方 **Assets** 中选择对应平台下载：

| Platform | Arch | Format | Filename |
|:--------:|:----:|:------:|:---------|
| 🪟 Windows 10/11 | x64 | NSIS .exe（推荐） | `API Router_0.3.0_x64-setup.exe` |
| 🪟 Windows 10/11 | x64 | WiX .msi | `API Router_0.3.0_x64_en-US.msi` |
| 🍎 macOS 12+ | Universal 2 | DMG | `API Router_0.3.0_aarch64.dmg` 等 |

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
| M2 🏃 | 2026 Q3 · v0.9-beta | macOS 发布线正式启用（本次已 CI 构建）· Tauri Updater · 配置导入导出已交付（v0.3.0） |
| M3 📝 | 2026 Q4 · v1.0.0 | Linux deb/rpm/AppImage · 配置迁移 · CSP 收紧 |

---

## 🆘 反馈 & 社区

- 遇到 Bug 或有功能建议：👉 [GitHub Issues](https://github.com/xf2214/api-router/issues/new)
- 喜欢项目记得 **Star ⭐** 支持一下，这是我们迭代最大的动力！

---

*Release generated at 2026-09-04 · Tag `v0.3.0`*
