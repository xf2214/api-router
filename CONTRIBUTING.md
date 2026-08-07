# 贡献指南

<p align="center">
  中文 | <a href="CONTRIBUTING.en-US.md">English</a>
</p>

感谢你对 API Router 的关注！我们欢迎任何形式的贡献，包括但不限于：

- 提交 Issue 报告 bug 或提出功能建议
- 提交 Pull Request 修复问题或实现功能
- 改进文档
- 分享使用经验

## 行为准则

请遵守 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) 中的社区行为准则，保持友善、尊重和包容。

## 如何开始

1. Fork 本仓库。
2. 克隆你的 Fork 到本地。
3. 创建一个新的分支：`git checkout -b feat/your-feature-name` 或 `fix/your-bug-description`。
4. 按照 [README.md](README.md) 中的说明配置开发环境。

## 开发规范

### 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范。提交信息格式如下：

```
<type>(<scope>): <subject>

<body>
```

常见 `type`：

- `feat`：新功能
- `fix`：Bug 修复
- `refactor`：重构
- `test`：测试
- `docs`：文档
- `chore`：构建/工具
- `perf`：性能优化

示例：

```
feat(router): add weighted backend selection

Implement weighted random strategy for multi-backend routing.
```

### Rust 代码规范

- 函数/变量使用 `snake_case`，类型/枚举使用 `PascalCase`，常量使用 `SCREAMING_SNAKE_CASE`。
- 使用 `?` 传播错误；自定义错误类型统一放在 `src-tauri/src/error.rs`。
- 所有 `async` 函数必须考虑取消安全性。
- 敏感数据使用 `secrecy::SecretString`，禁止 `println!` / `dbg!` 输出。
- 避免 `unsafe`；如必须使用，需逐行注释说明必要性。

### TypeScript / Vue 代码规范

- 变量/函数使用 `camelCase`，组件/类型使用 `PascalCase`，常量使用 `UPPER_SNAKE_CASE`。
- 所有变量与函数返回类型必须显式声明；禁用 `any`。
- 组件文件使用 SFC（`.vue`）。
- API 调用统一封装在 `src/services/` 目录，禁止在组件中直接写 `fetch`。
- 错误提示使用统一的 Toast / Notification 封装，禁止裸 `alert`。

## 质量门禁

提交 PR 前，请确保以下检查全部通过：

```bash
# Rust
cd src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# 前端
cd ..
npm run type-check
npm run build
```

## 提交 Pull Request

1. 确保你的分支基于最新的 `main` 分支。
2. 在 PR 描述中清楚说明变更内容、动机和测试方式。
3. 关联相关的 Issue（如果有）。
4. 等待维护者 review。

## 报告 Bug

提交 Issue 时，请尽可能提供以下信息：

- 操作系统及版本
- Rust 版本：`rustc --version`
- Node.js 版本：`node --version`
- 复现步骤
- 期望行为与实际行为
- 相关日志或截图

## 提出功能建议

我们欢迎功能建议！请在 Issue 中描述：

- 功能场景与动机
- 期望的接口或行为
- 是否愿意自己实现

## 许可证

通过向本项目提交代码，你同意你的贡献将在 [MIT 许可证](LICENSE) 下发布。
