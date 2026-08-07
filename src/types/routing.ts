export interface ModelTarget {
  provider_id: string;
  model_name: string;
  weight: number;
  override_params?: Record<string, any>;
  tier: number;
}

export type RoutingStrategy = 'priority' | 'weighted' | 'round_robin';

export interface GroupMember {
  local_name: string;
  weight: number;
}

export interface ModelGroup {
  name: string;
  members: GroupMember[];
  strategy: RoutingStrategy;
  fallback_enabled: boolean;
}

export interface CircuitBreakerConfig {
  failure_threshold: number;
  failure_threshold_percentage: number | null;
  cooldown_interval_ms: number;
  failure_status_codes: number[] | null;
  minimum_requests: number | null;
}

export interface AccessPoint {
  provider_id: string;
  upstream_model_name: string;
  enabled: boolean;
  weight: number;
}

export interface ModelDefinition {
  id: string;
  display_name: string;
  context_length?: number;
  max_tokens?: number;
  auto_aggregate: boolean;
  access_points: AccessPoint[];
}

export interface ModelMapping {
  local_name: string;
  strategy: RoutingStrategy;
  fallback_enabled: boolean;
  max_retries: number;
  targets: ModelTarget[];
  group: string;
  model_id?: string;
  context_length?: number;
  max_tokens?: number;
  override_params?: Record<string, any>;
  cb_config?: CircuitBreakerConfig;
  retry?: import('./config').RetryConfig;
}

/** 路由目标（第三层叶子节点） */
export interface RouteTargetLeaf {
  key: string;                // 唯一键: `${provider_id}::${model_name}::tier${tier}`
  providerId: string;
  providerName: string;       // 从 config.providers 反查的显示名
  upstreamModel: string;
  weight: number;
  tier: number;               // 层级（默认 1，数字越小优先级越高）
  enabled: boolean;           // 供应商是否启用
}

/** 模型节点（第二层中间节点） */
export interface RouteModelNodeV2 {
  key: string;                // member.local_name
  localName: string;          // 本地模型名
  displayLabel: string;       // 展示标签（模型定义 display_name 或 localName）
  strategy: RoutingStrategy;  // 调度策略（从 ModelMapping 取）
  memberWeight: number;       // 分组内成员权重
  targets: RouteTargetLeaf[]; // 下层路由目标
  hasModelDefinition: boolean; // 是否引用了模型定义（用于编辑跳转提示）
  mappingModelId?: string;    // 若引用模型定义，则为定义 id
}

/** 分组节点（第一层根节点） */
export interface RouteGroupTree {
  key: string;                // group.name
  groupName: string;
  strategy: RoutingStrategy;  // 分组调度策略
  fallbackEnabled: boolean;   // 失败切换开关
  models: RouteModelNodeV2[]; // 下层模型节点
}
