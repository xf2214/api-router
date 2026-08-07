import type { CircuitBreakerConfig } from './routing';

export interface ModelInfo {
  context_length?: number;
  max_tokens?: number;
}

export interface ProviderConfig {
  id: string;
  name: string;
  base_url: string;
  timeout_seconds: number;
  qps_limit: number;
  concurrency_limit: number;
  tpm_limit: number;
  enabled: boolean;
  default_models: string[];
  extra_headers: Record<string, string>;
  disable_proxy: boolean;
  model_info?: Record<string, ModelInfo>;
  provider_type?: 'openai_compatible' | 'custom_template';
}

export interface RetryConfig {
  attempts: number;
  on_status_codes: number[] | null;
  use_retry_after_headers: boolean;
}

export interface CacheConfig {
  enabled: boolean;
  mode: 'simple' | 'TTL' | 'LRU';
  max_age_seconds: number;
  max_entries: number;
}

export interface FallbackConfig {
  default_retries: number;
  timeout_seconds: number;
  cb_config: CircuitBreakerConfig;
}

export interface AppConfig {
  port: number;
  local_api_token: string | null;
  enable_logging: boolean;
  providers: ProviderConfig[];
  models: import('./routing').ModelMapping[];
  model_definitions: import('./routing').ModelDefinition[];
  groups: import('./routing').ModelGroup[];
  fallback: FallbackConfig;
  enable_auto_health_check: boolean;
  health_check_interval_seconds: number;
  cache: CacheConfig;
  bind_address?: string;
  enable_cors?: boolean;
  log_level?: string;
  log_retention_days?: number;
  default_stream?: boolean;
}

export interface ServerStatus {
  running: boolean;
  port: number;
}

export interface ProviderWithKey extends ProviderConfig {
  api_key: string;
}
