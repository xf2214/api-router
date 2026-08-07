# AI Agent 开发规范

本文档约定本项目中人类开发者与 AI Agent 的协作方式、代码规范与质量门禁。任何 Agent 在修改代码或生成内容前，应优先阅读本文档。

## 1. 项目基本原则

1. **绝对轻量**：不引入非必要的运行时依赖、框架或重型库；每个新增依赖都需要在注释中说明理由。
2. **极高可靠**：优先使用编译期保证与显式错误处理；禁止静默吞掉错误；所有网络/IO 操作必须可超时、可取消。
3. **最小可行**：只实现当前需求明确需要的功能，不为“未来可能”做过度抽象。
4. **本地优先**：所有数据默认只留在用户机器，不上传云端；网络请求必须尊重系统代理设置。
5. **跨平台一致**：Windows 与 macOS 共享同一套核心逻辑，平台相关代码必须隔离在最小模块内。

## 2. 技术栈与目录约定

### 2.1 技术栈
- **后端桌面核心**：Rust（Tauri v2）
- **本地 HTTP 服务**：Axum + Tokio
- **前端 UI**：Vue 3 + TypeScript（或 React 18，待项目初始化时确定）
- **配置格式**：YAML
- **构建工具**：Cargo / pnpm / GitHub Actions

### 2.2 目录结构
```
api-router/
├── src-tauri/          # Rust 后端代码
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── router.rs
│   │   ├── server.rs
│   │   ├── transform.rs
│   │   ├── telemetry.rs
│   │   ├── keyring.rs
│   │   └── error.rs
│   └── Cargo.toml
├── src/                # 前端源码
├── public/
├── docs/               # Markdown 文档
├── scripts/            # 构建/发布脚本
├── tests/              # 集成测试与 fixtures
└── .github/workflows/  # CI/CD
```

### 2.3 命名约定
- Rust：snake_case 函数/变量，PascalCase 类型/枚举，SCREAMING_SNAKE_CASE 常量。
- TypeScript：camelCase 变量/函数，PascalCase 组件/类型，UPPER_SNAKE_CASE 常量。
- 文件名与模块名保持小写，单词间用 `-` 或 `_` 分隔。

## 3. 代码规范

### 3.1 Rust
- 使用 `?` 传播错误；自定义错误类型统一放在 `error.rs`。
- 所有 `async` 函数必须考虑取消安全性；长时间任务使用 `tokio::select!` 时优先处理取消信号。
- 敏感数据（API Key、Token）使用 `secrecy::SecretString`，禁止 `println!` / `dbg!` 输出。
- 避免 `unsafe`；如必须使用，需逐行注释说明必要性并通过人工 review。
- 所有公开函数必须带有 rustdoc 注释；私有函数在逻辑不自明时加注释。

### 3.2 TypeScript / Vue
- 所有变量与函数返回类型必须显式声明；禁用 `any`。
- 组件文件使用 SFC（`.vue`），逻辑与模板分离清晰。
- API 调用统一封装在 `services/` 目录，禁止在组件中直接写 `fetch`。
- 错误提示使用统一的 Toast / Notification 封装，禁止裸 `alert`。

### 3.3 配置与状态
- 配置读取必须经过 schema 校验（Rust 端使用 `serde` + 手动校验）。
- 配置热重载时，旧连接应优雅完成，新连接使用新配置。
- 前端状态管理使用轻量级方案（Pinia 或 Composition API），禁止引入 Redux/MobX 等重型库。

## 4. 测试与质量门禁

### 4.1 测试策略
- **单元测试**：Rust 核心逻辑（路由、转换、配置校验）覆盖率目标 ≥ 80%。
- **集成测试**：使用 mock 上游验证端到端转发，覆盖流式与非流式场景。
- **E2E 测试**：Tauri 端关键路径（添加提供商、启动服务、调用本地 API）。

### 4.2 CI 门禁
- 每次 PR 必须通过：
  - `cargo clippy -- -D warnings`
  - `cargo fmt --check`
  - `cargo test`
  - `pnpm lint && pnpm type-check`
- 不允许合并未通过门禁的代码。

### 4.3 提交规范
使用 Conventional Commits：
```
<type>(<scope>): <subject>

<body>
```
常见 type：`feat`、`fix`、`refactor`、`test`、`docs`、`chore`、`perf`。
示例：
```
feat(router): add weighted backend selection

Implement weighted random strategy for multi-backend routing.
```

## 5. 文档要求

- 新增公开 API、配置项或用户可见行为时，同步更新 `API_SPEC.md` 或 `PRD.md`。
- 复杂算法、非显而易见的优化、平台特殊处理必须加代码注释。
- 不主动创建无关文档；文档改动与代码改动放在同一 PR。

## 6. AI Agent 协作约定

### 6.1 开始任务前
1. 阅读 `PRD.md`、`ARCHITECTURE.md`、`API_SPEC.md`、`ROADMAP.md` 与本文档。
2. 明确当前任务属于 ROADMAP 的哪个阶段/里程碑。
3. 若需求存在歧义，优先使用 AskUserQuestion 确认，而非自行假设。

### 6.2 编写代码时
1. 优先编辑现有文件；新增文件必须说明必要性。
2. 每个函数/模块只做一件事；避免一次性提交过大的代码块。
3. 修改后必须运行相关测试或 lint；若测试缺失，应补充测试。
4. 不引入未声明的依赖；新增依赖需在 PR 中说明理由与体积影响。

### 6.3 提交与沟通
1. 不向用户索要无关信息；问题应聚焦在当前任务的歧义点。
2. 完成一个任务后，简要汇报改动点、验证方式与下一步建议。
3. 不代替用户做产品方向的决策；涉及功能取舍时给出选项并等待确认。

### 6.4 禁止项
- 禁止在未告知用户的情况下修改 `.github/workflows`、签名证书或发布配置。
- 禁止在代码中硬编码任何 API Key、密码或测试凭据。
- 禁止生成或提交二进制文件、日志文件或个人配置文件到仓库。

## 7. 版本与发布

- 版本号遵循 SemVer：`MAJOR.MINOR.PATCH`。
- `MAJOR`：破坏性接口或配置变更；`MINOR`：新功能；`PATCH`：bug 修复。
- Release 由 GitHub Actions 在 tag 推送后自动构建、签名与发布；不手动本地打包发布。

## 8. 变更记录

| 日期 | 变更人 | 说明 |
|------|--------|------|
| 2026-07-21 | AI Agent | 初稿 |
