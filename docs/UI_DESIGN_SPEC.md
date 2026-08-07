# API Router —— 功能实现与 UI 设计对照文档

> 目的：为 UI 设计/重构提供一份准确的「后端已实现能力」清单，并指出当前 UI 与后端能力的对应关系、缺口与建议。

---

## 1. 产品概述

API Router 是一款本地运行的轻量级桌面应用，把多个大模型厂商的 API 统一封装成本地 OpenAI 兼容端点。用户配置一次后，即可通过 `http://127.0.0.1:6123` 调用不同模型，并在后端自动完成路由、重试、降级、限流、缓存、熔断等逻辑。

- **技术栈**：Tauri v2 + Rust + Vue 3 + TypeScript
- **本地服务端点**：`/v1/models`、`/v1/chat/completions`、`/v1/completions`、`/v1/embeddings`
- **配置存储**：`%APPDATA%\api-router\config.yaml`（Windows）或对应系统配置目录
- **密钥存储**：系统钥匙串 / Windows Credential（keyring 库）

---

## 2. 当前 UI 布局（已实现）

当前前端位于 `src/App.vue`，已改为左侧导航 + 顶部栏 + 单内容区的 Tab Shell：

| 导航入口 | 对应功能 | 状态 |
|---------|---------|------|
| **概览** | 服务状态、核心指标、Provider 健康摘要、映射摘要、快捷操作 | 已实现 |
| **API 提供商** | 增删改查 API 提供商、健康检查、获取模型列表 | 已实现 |
| **模型映射** | 本地模型名 ↔ 上游目标的映射、路由策略、连通测试 | 已实现 |
| **监控统计** | 按 provider/model 聚合的用量、成功率、延迟、Token | 已实现 |
| **请求日志** | 最近请求明细、筛选、搜索、导出、详情面板 | 已实现 |
| **设置** | 端口、本地 Token、重试/超时、自动健康检查、缓存、熔断器、日志、关于 | 已实现 |

布局结构：左侧 248px 导航栏（含 6 个入口与服务状态胶囊），顶部栏展示页面标题/面包屑、搜索入口（⌘K）、密度切换、主题切换；右侧内容区展示对应页面。主题切换与密度切换通过 `data-theme` / `data-density` 属性作用于全局样式。

---

## 3. 后端已实现功能清单

### 3.1 配置与持久化（config.rs）

- 配置以 YAML 保存在本地：`config.yaml`
- 启动时自动加载，保存时校验
- 校验规则：
  - provider id / base_url 不能为空
  - model mapping 的目标 provider 必须存在
- 支持字段：
  - `port`、`local_api_token`、`enable_logging`
  - `providers[]`、`models[]`
  - `fallback`（默认重试次数、默认超时、默认熔断器）
  - `enable_auto_health_check`、`health_check_interval_seconds`
  - `cache`（启用、模式、TTL、最大条目数）

### 3.2 提供商管理（commands.rs / config.rs）

每个 provider 包含：

| 字段 | 说明 | 当前 UI 是否暴露 |
|------|------|-----------------|
| `id` | 唯一标识 | 是 |
| `name` | 显示名称 | 是 |
| `base_url` | 上游 API Base URL | 是 |
| `timeout_seconds` | 超时（秒） | 是 |
| `qps_limit` | 每秒请求数限制（0=不限） | 是 |
| `concurrency_limit` | 并发限制（0=不限） | 是 |
| `enabled` | 是否启用 | 是 |
| `default_models` | 默认模型列表 | 是 |
| `extra_headers` | 额外请求头 KV | 是 |
| `disable_proxy` | 禁用系统代理，直连该供应商 | 是 |
| `model_info` | 从 `/models` 接口解析的上下文长度、最大输出 | 仅展示，不可编辑 |

**已有命令**：
- `get_config` / `save_config` / `delete_provider`
- `fetch_provider_models`：从上游 `/models` 拉取模型列表与元数据
- `check_provider_health` / `check_all_providers_health` / `get_health_status`
- `test_provider_target`：不保存即可测试某个 provider + model

### 3.3 模型映射与路由（router.rs / config.rs）

每个 model mapping 包含：

| 字段 | 说明 | 当前 UI 是否暴露 |
|------|------|-----------------|
| `local_name` | 本地模型名（客户端请求的 model） | 是 |
| `strategy` | 路由策略：`priority` / `weighted` / `round_robin` | 是 |
| `fallback_enabled` | 是否启用失败切换 | 是 |
| `max_retries` | 每个目标最大重试次数 | 是（仅数字） |
| `targets` | 上游目标列表（provider + model + weight + override_params） | 部分 |
| `group` | 分组名 | 是 |
| `context_length` / `max_tokens` | 模型元数据 | 是（可编辑） |
| `cb_config` | 本映射专用熔断器配置 | **未暴露** |
| `retry` | 本映射专用重试策略（状态码、Retry-After 头等） | **未暴露** |

**targets 中每个目标包含**：
- `provider_id`、`model_name`、`weight`（weighted 策略时使用）
- `override_params`：可覆盖请求体中的任意参数（如 temperature、max_tokens）——**当前 UI 未提供编辑入口**

**路由逻辑**（已实现）：
1. 按 `local_name` 找 mapping
2. 过滤已禁用的 provider
3. 结合健康状态：最近一次健康检查失败且在 60s 冷却期内的 provider 被跳过
4. 结合熔断器：熔断打开的目标被跳过
5. 无可选目标时退化为「让请求自行探测」，而不是直接报错
6. 策略选择：顺序优先 / 按权重随机 / 轮询

### 3.4 请求转发与重试（server.rs / client.rs）

- 单次请求转发：`forward_request_once`
- 失败自动重试：`forward_with_retry`
  - 可重试错误：408/425/429/5xx、网络错误、超时
  - 退避策略：优先读取上游 `Retry-After` / `Retry-After-Ms`，否则指数退避 + 抖动
  - 重试次数来源：mapping 的 `max_retries` → mapping 的 `retry.attempts` → 全局 `fallback.default_retries`
- QPS 限流：简单令牌桶
- 并发限流：Semaphore
- 熔断器失败/成功记录

### 3.5 熔断器（circuit.rs）

按 `provider_id` 维护熔断状态：
- 触发条件（满足任一）：
  - 连续失败次数 ≥ `failure_threshold`
  - 失败率 ≥ `failure_threshold_percentage`（且总请求数 ≥ `minimum_requests`）
- 默认视为失败的状态码：5xx 与 429
- 可配置 `failure_status_codes` 覆盖默认
- 熔断打开后进入 `cooldown_interval_ms` 冷却期，之后允许半开试探
- 成功后自动关闭

**当前问题**：UI 完全没有熔断器的查看与配置入口，但后端已完整实现。

### 3.6 请求/响应转换（transform.rs）

- 将请求体中的 `model` 替换为上游实际模型名
- 应用 target 级 `override_params` 覆盖/合并参数
- 清理请求体：移除 null、空数组、空字符串、`max_tokens=0`
- 响应中的 `model` 字段写回本地模型名
- 流式 SSE chunk 同样会做模型名回写

### 3.7 缓存（cache.rs）

- 仅对**非流式**请求生效
- 按 `(endpoint, body)` 精确匹配
- 支持 TTL、最大条目数、LRU 淘汰
- 模式当前只有 `simple`

**当前 UI**：仅暴露启用开关与 TTL，未展示缓存命中率/清空操作。

### 3.8 监控与统计（metrics.rs / commands.rs）

- 统计始终维护，详细日志受 `enable_logging` 控制
- 记录字段：request_id、local_model、provider_id、upstream_model、endpoint、stream、时间、耗时、状态码、成功与否、错误、重试次数、是否降级、token 用量
- 聚合维度：`(provider_id, upstream_model)`
- 命令：`get_request_logs`、`get_request_stats`、`clear_request_logs`

### 3.9 健康检查（state.rs / commands.rs）

- 手动检查：单 provider / 全部 provider
- 自动检查：启动后台任务，按 `health_check_interval_seconds`（默认 300s）周期执行
- 健康结果影响路由决策（60s 离线冷却期）
- 检查方式：调用上游 `/models` 接口

### 3.10 本地 HTTP 服务（server.rs / lib.rs）

- 应用启动时自动启动本地服务
- 支持 CORS
- 可选本地 Bearer Token 鉴权
- 端点：
  - `GET  /v1/models`
  - `POST /v1/chat/completions`
  - `POST /v1/completions`
  - `POST /v1/embeddings`
  - `GET  /health`

---

## 4. 当前 UI 与后端能力对照表

### 4.1 完全对应/已可用的功能

| 功能 | 后端 | 前端 | 状态 |
|------|------|------|------|
| 新增/编辑/删除 provider | ✅ | ✅ | 可用 |
| provider 启用/禁用 | ✅ | ✅ | 可用 |
| 获取上游模型列表 | ✅ | ✅ | 可用 |
| provider 健康检查（单/全部） | ✅ | ✅ | 可用 |
| 自动健康检查开关 | ✅ | ✅ | 可用 |
| 本地服务启动/停止/状态 | ✅ | ✅ | 可用 |
| 模型映射 CRUD | ✅ | ✅ | 可用 |
| 路由策略选择 | ✅ | ✅ | 可用 |
| 失败切换开关 + 最大重试 | ✅ | ✅ | 可用 |
| 模型连通测试 | ✅ | ✅ | 可用 |
| 端口/本地 Token 设置 | ✅ | ✅ | 可用 |
| 请求日志与统计展示 | ✅ | ✅ | 可用 |
| 缓存开关与 TTL | ✅ | ✅ | 可用 |
| provider 级 disable_proxy | ✅ | ✅ | 可用 |
| provider 级 extra_headers | ✅ | ✅ | 可用 |
| provider 级 timeout/qps/concurrency | ✅ | ✅ | 可用 |

### 4.2 后端已实现但前端未暴露/未充分利用的功能

| 功能 | 后端 | 前端 | 缺口说明 |
|------|------|------|---------|
| 熔断器全局配置（`fallback.cb_config`） | ✅ | ✅ | 设置页已提供全局熔断器默认配置 |
| 熔断器单映射配置（`mapping.cb_config`） | ✅ | ❌ | 模型映射表单仍无熔断器选项 |
| 单映射高级重试策略（`retry.on_status_codes`、`use_retry_after_headers`） | ✅ | ❌ | 只有 `max_retries` 数字输入 |
| 目标级参数覆盖（`target.override_params`） | ✅ | ✅ | ModelForm 每个 target 提供 JSON 覆盖参数编辑 |
| 缓存命中率/清空缓存 | ✅ | ✅ | 设置页缓存卡片展示条目数/命中率/最后清理，并支持清空 |
| 熔断器状态可视化 | ✅ | ❌ | 无法直观看到哪些 provider 被熔断 |
| 健康检查错误详情常驻展示 | ✅ | ⚠️ | Provider 列表展示在线/离线状态，错误详情可进一步常驻 |
| 日志筛选/搜索/导出 | ✅ | ✅ | 请求日志页提供搜索、模型/状态/流式/时间筛选与导出 |
| 模型分组批量操作 | ✅ | ⚠️ | 仅有分组筛选，无批量管理 |

### 4.3 当前 UI 展示但不确认后端行为的功能

| UI 展示 | 后端实际 | 说明 |
|--------|---------|------|
| 设置页「默认失败重试次数」 | 仅作为 fallback 默认值 | 单 mapping 的 `max_retries` 优先级更高，但 UI 未说明 |
| 模型映射「最大重试」 | 映射到 `max_retries` | 实际重试次数还受 `retry.attempts` 影响，UI 未暴露 |
| 日志/统计自动刷新 | 5s 轮询 | 符合预期 |

---

## 5. UI 设计建议

### 5.1 布局：当前实现

本次重构后，主界面采用左侧导航 + 单内容区的 Tab Shell（参见第 2 节）。三列并排布局（提供商 / 模型映射 / 设置）仍可作为远期演进方向，但当前已通过独立 Tab 页完整承载所有功能。

- **概览页**：服务状态、核心指标、Provider 健康摘要、映射摘要、快捷操作
- **提供商页**：卡片列表 + 新增/编辑弹窗，显示健康状态、延迟、默认模型
- **模型映射页**：卡片列表 + 编辑抽屉，显示策略、目标、分组
- **监控统计页**：指标卡片 + 聚合表 + 失败/延迟分析
- **请求日志页**：筛选 + 列表 + 详情面板
- **设置页**：服务、路由默认、缓存、熔断器默认值、健康检查、日志、关于

### 5.2 新增/强化的面板

#### A. 熔断器配置面板（全局已实现，单映射待补充）

设置页已暴露全局默认值，建议保持并继续补充单映射配置。字段清单：

- `failure_threshold`：连续失败次数阈值
- `failure_threshold_percentage`：失败率百分比阈值
- `minimum_requests`：计算失败率前的最少请求数
- `cooldown_interval_ms`：熔断后冷却时长
- `failure_status_codes`：视为失败的状态码列表

交互建议：
- 设置页提供「全局熔断器默认」
- ModelForm 高级选项提供「使用全局 / 自定义」切换
- provider 行或模型行上增加熔断状态指示（Closed / Open）

#### B. 目标级参数覆盖（override_params）

ModelForm 的每个 target 区块已提供 JSON 覆盖参数输入框，可编辑 `override_params`；建议后续增强为 KV 编辑器，并补充常见参数快捷按钮（`temperature`、`max_tokens`、`top_p` 等）与说明文字。

#### C. 缓存状态面板

设置页已提供缓存状态卡片：

- 显示：是否启用、当前条目数、命中率、最后清理时间
- 操作：清空缓存
- 命中率为前端本地估算展示；后端 `ResponseCache` 暂缺专用查询命令，可后续补充以显示真实命中率

#### D. 重试策略精细化

ModelForm 高级选项中：

- `max_retries` 保留
- 增加 `on_status_codes`：仅在这些状态码上重试
- 增加 `use_retry_after_headers`：是否优先使用上游 Retry-After

### 5.3 当前表单可优化点

| 位置 | 问题 | 建议 |
|------|------|------|
| ProviderForm「默认模型」 | 下拉框层级深，批量操作不够直观 | 提供「获取模型 → 批量勾选 → 一键创建映射」的明确流程 |
| ProviderForm「高级设置」 | `disable_proxy` 已存在，但缺少说明场景 | 保留并增加提示：系统代理未运行或该供应商在 NO_PROXY 时才需勾选 |
| ModelForm「上游目标」 | 无 override_params | 增加参数覆盖编辑器 |
| ModelForm「路由策略」 | weighted 时才显示 weight | 保持，但增加策略说明 tooltip |
| App.vue「日志统计」Tab | 独立 Tab 与三列约定冲突 | 改为右侧面板或抽屉 |

### 5.4 状态可视化建议

- **provider 健康**：绿/红/灰点 + 延迟数字 + 最后检查时间
- **熔断状态**：在 provider 行增加「熔断中」标签（当 `circuit_breaker.is_open` 为 true）
- **本地服务状态**：顶部常驻，当前已实现，保持
- **缓存状态**：设置页增加小卡片展示

### 5.5 添加/编辑供应商表单（ProviderForm）

`ProviderForm.vue` 是新增/编辑供应商的弹窗表单，建议保持现有两 Tab 结构（基础设置 / 高级设置），并补全以下功能说明与设计细节。

#### A. 表单字段与后端对应关系

| 字段 | 后端字段 | 所在 Tab | 说明 | 当前状态 |
|------|---------|---------|------|---------|
| 选择预设供应商 | - | 基础设置 | 自动填充 ID / 名称 / Base URL | 已实现 |
| ID | `id` | 基础设置 | 唯一标识，编辑时不可改，保存后作为 `provider_id` 被映射引用 | 已实现 |
| 显示名称 | `name` | 基础设置 | 列表中展示的名称 | 已实现 |
| Base URL | `base_url` | 基础设置 | 上游 API 地址，保存时自动去除末尾 `/` | 已实现 |
| API Key | keyring | 基础设置 | 不写入 config.yaml，通过 `save_config` 的 `keys` 参数存钥匙串 | 已实现 |
| 默认模型 | `default_models` | 基础设置 | 该 provider 暴露的模型列表，支持从 `/models` 获取或手动添加 | 已实现 |
| 启用该提供商 | `enabled` | 基础设置 | 禁用后路由不再选择该 provider | 已实现 |
| 超时（秒） | `timeout_seconds` | 高级设置 | 单个请求超时，默认 60 | 已实现 |
| QPS 限制 | `qps_limit` | 高级设置 | 0=不限 | 已实现 |
| 并发限制 | `concurrency_limit` | 高级设置 | 0=不限 | 已实现 |
| 禁用系统代理 | `disable_proxy` | 高级设置 | 勾选后该 provider 使用 `.no_proxy()` 直连 | 已实现 |
| 额外请求头 | `extra_headers` | 高级设置 | 每条请求都会带上的自定义 Header | 已实现 |

> **注意**：后端 `ProviderConfig` 的 `model_info` 字段由「获取模型」自动写入，用于保存上下文长度、最大输出等元数据，**不应由用户直接编辑**。

#### B. 推荐交互流程

```
选择预设（可选）
  ↓
填写 Base URL + API Key
  ↓
点击「获取模型」→ 调用 fetch_provider_models
  ↓
模型下拉框弹出，展示模型名 + 上下文/最大输出
  ↓
批量勾选默认模型（支持全选/清空/手动添加）
  ↓
点击「测试连通」→ 用第一个已选模型调用 test_provider_target
  ↓
保存 → 后端存配置 + API Key；前端自动为每个已选模型创建本地映射
```

#### C. 保存后的自动映射行为

当前 `App.vue` 已实现：
- 新增供应商时，若 `default_models` 非空，自动为每个模型名创建 `ModelMapping`
- 默认映射规则：
  - `local_name` = 上游模型名
  - `strategy` = `priority`（顺序优先）
  - `targets[0]` = `{ provider_id: 当前 provider, model_name: 上游模型名, weight: 1 }`
  - `fallback_enabled` = true
  - `max_retries` = 2
  - `group` = "默认"

**设计建议**：
- 在「默认模型」区块增加说明：保存后会自动创建对应的本地模型映射
- 提供开关「保存时自动创建映射」（默认开启），允许高级用户只配置 provider 而不生成映射
- 若模型名已存在映射，提示「已跳过重复的本地模型名」

#### D. 校验与错误反馈

| 校验项 | 来源 | 反馈方式 |
|--------|------|---------|
| ID 不能为空 | 后端 `validate` | 保存时弹窗提示 |
| Base URL 不能为空 | 后端 `validate` | 保存时弹窗提示 |
| Base URL 去尾斜杠 | 前端 `normalizeBaseUrl` | blur 时自动处理 |
| 新增时 API Key 必填（获取模型/测试） | 前端 | 按钮点击时提示 |
| 测试连通失败 | 后端 `testProviderTarget` | 在 API Key 下方显示红色结果 |
| 获取模型失败 | 后端 `fetchProviderModels` | 在 API Key 下方显示红色错误 |

#### E. 建议补全的交互细节

1. **预设模板扩展**
   - 当前预设：OpenAI、Anthropic、DeepSeek、Moonshot、智谱、SiliconFlow、OpenRouter、SenseNova、Azure
   - 建议补充：Groq、X.ai、Mistral、Cohere、Together、Fireworks 等常用平台

2. **获取模型后自动选中默认模型**
   - 当前获取后需要手动勾选；建议提供「获取后全选默认模型」的辅助按钮
   - 或在首次获取时默认勾选全部

3. **模型信息展示优化**
   - 当前 `modelInfoLabel` 在选项中显示上下文/最大输出，但下拉框较窄
   - 建议：选项右侧增加小字标签（如 128k / 8k），失败或缺失时显示 `-`

4. **测试连通前置校验**
   - 当前若未获取模型就点击测试，会提示「请先获取模型或选择默认模型」
   - 建议：当 `default_models` 为空且未获取模型时，测试按钮置灰并 tooltip 提示

5. **API Key 安全提示**
   - 增加说明：API Key 将保存到系统钥匙串，不会明文写入配置文件
   - 编辑时输入框 placeholder 已提示「留空则保持不变」，保持即可

6. **额外请求头增强**
   - 当前仅支持 key/value 文本输入
   - 建议：提供常用 Header 快捷填充（如 `Authorization`、`X-API-Version`）

---

### 5.6 概览面板（新增）

作为应用启动后的默认 Tab，概览应提供「一眼看清全局」的信息密度，避免进入具体配置页才能了解状态。

**建议展示内容**：

| 模块 | 展示项 | 数据来源 |
|------|--------|---------|
| **服务状态卡片** | 本地服务运行/停止、监听端口、本地 API 地址 | `get_server_status` |
| **核心指标卡片** | 总请求数、成功率、平均延迟、累计 Token | `get_request_stats` |
| **Provider 健康摘要** | 在线数 / 总数、最近离线 provider、平均延迟 | `get_health_status` |
| **模型映射摘要** | 已配置本地模型数、启用 provider 数 | `get_config` |
| **快捷操作** | 启动/停止服务、检查全部 provider、打开日志 Tab | 前端路由切换 |

**设计要点**：
- 使用大字号指标卡片，颜色区分状态（绿/红/灰）
- 点击指标卡片可跳转到对应 Tab（如成功率 → 监控统计）
- provider 健康摘要使用紧凑列表，显示最近检查时间和延迟

### 5.7 监控统计面板（从「日志统计」拆分）

该 Tab 专注于**聚合统计**，不展示单条请求明细。

**已有数据字段**（来自 `ProviderStat`）：
- `provider_id`、`upstream_model`
- `total_requests`、`success_count`、`failure_count`
- `total_duration_ms`、`total_retries`
- `total_prompt_tokens`、`total_completion_tokens`、`total_tokens`
- `last_request_at_ms`

**建议展示内容**：

1. **顶部总览卡片**（同概览，但在此页可常驻）
   - 总请求 / 成功率 / 平均延迟 / 累计重试 / 累计 Token

2. **按 Provider/Model 聚合表**
   - 列：提供商、上游模型、请求数、成功、失败、成功率、平均延迟、重试、Token、最近请求
   - 支持按 provider 折叠、按请求数排序
   - 当前 UI 已实现该表，可直接迁移并增强

3. **趋势图（可选增强）**
   - 后端当前只返回累计值，没有时序数据
   - 若需趋势图，需要后端补充按时间窗口聚合，或前端在内存中维护最近 N 个采样点
   - **建议先做表格，趋势图列为 P2**

4. **失败分析**
   - 失败率最高的 provider/model
   - 点击可跳转到请求日志并自动筛选该 provider

**设计要点**：
- 表头增加排序按钮
- 成功率低于 80% 标黄，低于 60% 标红
- 提供「清空统计」按钮（调用 `clear_request_logs`）

### 5.8 请求日志面板（从「日志统计」拆分）

该 Tab 专注于**单条请求明细**查询与排查。

**已有数据字段**（来自 `RequestLog`）：
- `request_id`、`local_model`、`provider_id`、`upstream_model`
- `endpoint`、`stream`、`started_at_ms`、`duration_ms`
- `status`、`success`、`error`、`retries`、`fell_back`
- `usage`（token 用量）

**建议展示内容**：

1. **筛选栏**
   - 按 provider 筛选
   - 按 local_model 筛选
   - 按状态筛选：全部 / 成功 / 失败 / 4xx / 5xx
   - 按是否流式筛选
   - 按是否降级筛选
   - 时间范围：最近 5 分钟 / 15 分钟 / 1 小时 / 今天 / 全部
   - 搜索框：按 request_id 或 local_model 搜索

2. **请求列表**
   - 默认显示最近 50/100/200/500 条
   - 列：时间、请求ID、本地模型、提供商、上游模型、端点、流式、状态、耗时、重试、降级、Token、错误
   - 当前 UI 已实现该表，可直接迁移

3. **请求详情抽屉**
   - 点击某行展开右侧/底部抽屉，显示完整信息：
     - 请求时间、耗时、请求ID
     - 本地模型 → 上游模型 的映射链路
     - 状态码与错误信息
     - Token 用量
     - 重试次数与是否经过降级
   - 可选：展示原始请求体/响应体的查看入口（需要后端补充命令）

4. **导出功能**
   - 导出当前筛选结果为 CSV / JSON
   - 由于数据仅存内存，导出由前端聚合当前列表生成文件

5. **日志开关提示**
   - 当 `enable_logging = false` 时，显示提示：
     > 请求日志记录未启用，当前仅展示聚合统计。前往「设置」开启后可查看详细日志。

**设计要点**：
- 失败行使用红色高亮
- 错误列支持 hover 显示完整错误信息
- 请求 ID 使用等宽字体并支持一键复制
- 自动刷新开关保留（当前 5s 轮询）

---

## 6. 后端命令速查（供前端调用）

| 命令 | 用途 | 所在文件 |
|------|------|---------|
| `get_config` | 加载配置 | commands.rs |
| `save_config` | 保存配置 + 存储 API Key | commands.rs |
| `delete_provider` | 删除 provider 及其密钥 | commands.rs |
| `start_server` / `stop_server` / `get_server_status` | 本地服务控制 | commands.rs |
| `fetch_provider_models` | 拉取上游模型列表 | commands.rs |
| `test_provider_target` | 测试 provider+model 连通性 | commands.rs |
| `check_provider_health` / `check_all_providers_health` / `get_health_status` | 健康检查 | commands.rs |
| `test_model_connection` / `test_model_config` | 模型映射连通测试 | commands.rs |
| `get_request_logs` / `get_request_stats` / `clear_request_logs` | 日志与统计 | commands.rs |

---

## 7. 关键类型定义（前后端共用）

详见：
- 前端类型：`src/types.ts`
- 后端类型：`src-tauri/src/config.rs`

需要 UI 重点关注的类型：

- `ProviderConfig`：provider 全部字段
- `ModelMapping`：映射全部字段，含 `cb_config`、`retry`
- `ModelTarget`：含 `override_params`
- `CircuitBreakerConfig` / `RetryConfig` / `CacheConfig`：高级配置
- `RequestLog` / `ProviderStat`：日志与统计字段

---

## 8. 总结：UI 重构优先级

| 优先级 | 事项 | 理由 |
|--------|------|------|
| P0 | 新增「概览」Tab 作为默认首页 | 用户进入即可看到服务状态和核心指标 |
| P0 | 拆分「日志统计」为「监控统计」+「请求日志」两个独立 Tab | 统计看聚合，日志查明细，职责分离 |
| P0 | 补齐概览/监控/日志所需的数据展示与跳转 | 后端命令已就绪，主要是前端展示 |
| P0 | 补齐添加供应商表单中的自动映射提示与交互细节 | 当前功能已实现，但用户认知路径不完整 |
| P0 | 补齐熔断器配置（全局 + 单映射） | 后端已完整实现，是核心可靠性能力 |
| P1 | 增加 `target.override_params` 编辑 | 高频需求：模型级 temperature/max_tokens 覆盖 |
| P1 | 增加缓存状态与清空操作 | 提升可观测性 |
| P1 | 增加熔断状态可视化 | 帮助用户理解为什么请求被路由到别处 |
| P2 | 日志筛选/搜索/导出 | 提升排查效率 |
| P2 | 重试策略精细化（状态码、Retry-After） | 后端已支持，但普通用户可保持默认 |
| P2 | 远期演进为三列布局 | 项目约定，但当前先以 Tab 框架补全功能 |

---

*文档生成时间：2026-07-23*
*对应后端 commit：以当前工作区 `src-tauri/src` 为准*
