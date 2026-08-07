# API Router macOS 构建验证文档

本文档说明如何在 macOS 上完成 API Router 项目的本地开发、构建通用二进制、绕过 Gatekeeper 以及配置代码签名与公证。

---

## 1. 环境准备

- **操作系统**：macOS 12 Monterey 或更高版本
- **Xcode Command Line Tools**：用于编译 Rust 与 Tauri 所需的系统依赖
  - 安装命令：`xcode-select --install`
- **Node.js 20+ 和 npm**：用于前端开发与运行 Tauri CLI
  - 推荐通过 [Node.js 官网](https://nodejs.org/) 或 `nvm` 安装
- **Rust 稳定版**：Tauri 后端的运行基础
  - 安装命令：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Tauri CLI**：
  - 全局安装：`npm install -g @tauri-apps/cli`
  - 或直接使用项目本地版本，无需全局安装

---

## 2. 本地开发

在项目根目录执行：

```bash
npm install
npm run tauri dev
```

`npm run tauri dev` 会同时启动前端 Vite 开发服务器和 Tauri 后端，并打开应用窗口。

---

## 3. 构建通用二进制

### 使用脚本

项目已提供通用二进制构建脚本：

```bash
chmod +x scripts/build-macos-universal.sh
./scripts/build-macos-universal.sh
```

### 直接运行 Tauri 构建

也可以不通过脚本，直接执行：

```bash
npm run tauri:build -- --target universal-apple-darwin
```

### 输出位置

构建产物位于：

```
src-tauri/target/universal-apple-darwin/release/bundle/
```

该目录下会生成 `.app` 应用程序包及 `.dmg` 安装镜像。

---

## 4. 绕过 Gatekeeper（本地未签名运行）

本地未签名的 `.app` 默认会被 macOS Gatekeeper 拦截，可采用以下任一方式绕过：

### 方式一：右键打开

在 Finder 中找到 `.app`，按住 `Control` 键并点击应用图标，选择**打开**，然后在弹出的安全提示中确认。

### 方式二：清除扩展属性

在终端执行：

```bash
xattr -cr "/path/to/API Router.app"
```

### 方式三：临时关闭 Gatekeeper（谨慎使用）

```bash
sudo spctl --master-disable
```

> ⚠️ 此命令会降低系统安全策略，仅建议在完全可信的本地构建环境中临时使用，验证完成后应及时恢复：`sudo spctl --master-enable`。

---

## 5. 代码签名与公证（可选）

如需分发，应进行正式代码签名与公证。

### 环境变量配置

参考项目根目录的 `.env.example` 文件，设置以下环境变量：

| 变量名 | 用途 |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64 编码的 Apple Developer ID Application 证书（.p12） |
| `APPLE_CERTIFICATE_PASSWORD` | 导出 .p12 证书时设置的密码 |
| `APPLE_ID` | 用于登录 Apple Developer 的 Apple ID |
| `APPLE_PASSWORD` | Apple ID 的 App 专用密码 |
| `APPLE_TEAM_ID` | Apple Developer Team ID（10 位字符） |

### 配置后构建

变量设置完成后，重新运行构建命令即可自动完成签名与公证：

```bash
npm run tauri:build -- --target universal-apple-darwin
```

---

## 6. CI 集成

项目已配置 GitHub Actions：`.github/workflows/ci.yml`，其中包含 macOS universal binary 的构建步骤。

要在 CI 中启用自动签名与公证，只需在 GitHub 仓库的 **Settings > Secrets and variables > Actions** 中添加上述 Apple 相关环境变量即可。

---

## 7. 常见问题

### 构建提示找不到目标

如果 Rust 缺少 Apple Silicon 或 Intel 目标，运行：

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

### `npm run tauri:build` 命令不存在

请检查项目根目录 `package.json` 中的 `scripts` 字段，确认是否存在 `tauri:build` 脚本。如果没有，可使用：

```bash
npx tauri build --target universal-apple-darwin
```

---
