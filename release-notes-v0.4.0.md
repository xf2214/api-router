# API Router v0.4.0 · Minor Release（Windows + macOS）

> 🪟 **Windows 10/11 x64** · 🍎 **macOS 12+ Universal 2**  
> 体验与可靠性打磨版：导入收尾、服务 watchdog、代码健康、六页面三态。

---

## ✨ 新增 · What's New

### 🛡️ 服务 watchdog 自动重启

- 异常退出指数退避重启（1s→30s），连续 5 次熔断停试并上报；显式停机不触发。

### 🖥️ 六页面三态统一

- 空状态 / 加载态 / 错误重试（Monitoring、Overview、Logs、Providers、Routing、RoutingTree、Settings 动作）。

### 🧪 单测补强

- toast 链 mount 测试、导入两态测试、clipboard 降级分支；vitest ≥35。

---

## 🔧 变更 · What's Changed

- **导入体验**：成功改走 `getConfig()` 重水合，不再整页 reload（tab 不再重置）；新增 `settings.includeKeys` / `settings.importConfig` 中英 key。
- **Keyring 测试 hermetic**：`#[cfg(test)]` 内存后端，Linux 无 DBus 也可跑。
- **Stores 精确化**：4 文件 8 处 `any` 改精确类型。

---

## ✅ 修复 · What's Fixed

- 复制端点 Toast 链断裂（OverviewPage/App 转发缺失）与 clipboard 降级泄漏（已在开发中修复）。
- 监控 CTA 硬编码文案改走既有 i18n key。

---

## 📦 Downloads · 安装包下载

> 🛠 安装包由 GitHub Actions 针对 `v0.4.0` 标签自动构建，请在上方 **Assets** 中选择对应平台下载：

| Platform | Arch | Format | Filename |
|:--------:|:----:|:------:|:---------|
| 🪟 Windows 10/11 | x64 | NSIS .exe（推荐） | `API Router_0.4.0_x64-setup.exe` |
| 🪟 Windows 10/11 | x64 | WiX .msi | `API Router_0.4.0_x64_en-US.msi` |
| 🍎 macOS 12+ | Universal 2 | DMG | `API Router_0.4.0_aarch64.dmg` 等 |

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
| M2 🏃 | 2026 Q3 · v0.9-beta | macOS 发布线正式启用（本次已 CI 构建）· Tauri Updater · 配置导入导出已交付（v0.3.0）· 导入体验收尾（v0.4.0） |
| M3 📝 | 2026 Q4 · v1.0.0 | Linux deb/rpm/AppImage · 配置迁移 · CSP 收紧 |

---

## 🆘 反馈 & 社区

- 遇到 Bug 或有功能建议：👉 [GitHub Issues](https://github.com/xf2214/api-router/issues/new)
- 喜欢项目记得 **Star ⭐** 支持一下，这是我们迭代最大的动力！

---

*Release generated at 2026-09-12 · Tag `v0.4.0`*
