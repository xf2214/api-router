export interface ProviderHealth {
  provider_id: string;
  online: boolean;
  last_checked: number | null;
  latency_ms: number | null;
  error: string | null;
}

export interface ModelTestTargetResult {
  provider_id: string;
  model_name: string;
  online: boolean;
  latency_ms: number | null;
  error: string | null;
}

export interface ModelTestResult {
  local_name: string;
  targets: ModelTestTargetResult[];
}

export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface RequestLog {
  request_id: string;
  local_model: string;
  provider_id: string;
  upstream_model: string;
  endpoint: string;
  stream: boolean;
  started_at_ms: number;
  duration_ms: number;
  status: number | null;
  success: boolean;
  error: string | null;
  retries: number;
  fell_back: boolean;
  usage: TokenUsage | null;
}

export interface ProviderStat {
  provider_id: string;
  upstream_model: string;
  total_requests: number;
  success_count: number;
  failure_count: number;
  total_duration_ms: number;
  total_retries: number;
  total_prompt_tokens: number;
  total_completion_tokens: number;
  total_tokens: number;
  last_request_at_ms: number;
}
