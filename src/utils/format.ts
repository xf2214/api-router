import type {
  ProviderConfig,
  ModelMapping,
  ModelTarget,
  ProviderStat,
  ProviderHealth,
  RequestLog,
} from '../types';

export function formatTime(ms: number): string {
  if (!ms) return '-';
  const date = new Date(ms);
  return date.toLocaleTimeString();
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

export function formatTokens(n?: number | null): string {
  if (n === undefined || n === null || n === 0) return '0';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return n.toLocaleString();
}

export function formatTokensCompact(n?: number | null): string {
  if (n === undefined || n === null || n === 0) return '0';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

export function providerColor(id: string): string {
  const colors = ['#10a37f', '#d97757', '#4285f4', '#0078d4', '#4d6bfe', '#615ced', '#34c759', '#ff9500', '#5e5ce6', '#bf5af2'];
  let hash = 0;
  for (let i = 0; i < id.length; i++) hash = id.charCodeAt(i) + ((hash << 5) - hash);
  return colors[Math.abs(hash) % colors.length];
}

export function providerGlyph(id: string, providers: ProviderConfig[]): string {
  const p = providers.find((x) => x.id === id);
  if (p) {
    const name = p.name.trim();
    if (name.length <= 2) return name;
    const words = name.split(/\s+/);
    if (words.length > 1) return (words[0][0] + words[words.length - 1][0]).toUpperCase();
    return name.slice(0, 2).toUpperCase();
  }
  return id.slice(0, 2).toUpperCase();
}

export function providerName(id: string, providers: ProviderConfig[]): string {
  return providers.find((p) => p.id === id)?.name ?? id;
}

export function strategyLabel(s: ModelMapping['strategy']): string {
  if (s === 'priority') return '顺序优先';
  if (s === 'weighted') return '按权重';
  return '轮询';
}

export function strategyBadgeClass(s: ModelMapping['strategy']): string {
  if (s === 'priority') return 'badge--mute';
  return 'badge--info';
}

export function sortedTargets(targets: ModelTarget[]): ModelTarget[] {
  return [...targets].sort((a, b) => {
    if (a.tier !== b.tier) return (a.tier || 1) - (b.tier || 1);
    return (b.weight || 1) - (a.weight || 1);
  });
}

export function statusCategory(log: RequestLog): 'success' | '4xx' | '5xx' | 'other' {
  if (log.success) return 'success';
  const s = log.status ?? 0;
  if (s >= 500) return '5xx';
  if (s >= 400) return '4xx';
  return 'other';
}

export function statusBadgeClass(log: RequestLog): string {
  const cat = statusCategory(log);
  if (cat === 'success') return 'badge--ok';
  if (cat === '5xx') return 'badge--err';
  if (cat === '4xx') return 'badge--warn';
  return 'badge--err';
}

export function successRateClass(rate: number): string {
  if (rate >= 95) return 'good';
  if (rate >= 80) return 'warn';
  return 'bad';
}

export function successBadgeClass(rate: number): string {
  if (rate >= 95) return 'badge--ok';
  if (rate >= 80) return 'badge--warn';
  return 'badge--err';
}

export function statSuccessRate(s: ProviderStat): number {
  if (s.total_requests === 0) return 0;
  return (s.success_count / s.total_requests) * 100;
}

export function statAvgLatency(s: ProviderStat): number {
  if (s.total_requests === 0) return 0;
  return Math.round(s.total_duration_ms / s.total_requests);
}

export function healthClass(
  providerId: string,
  healthMap: Record<string, ProviderHealth>
): 'ok' | 'err' | 'idle' {
  const h = healthMap[providerId];
  if (!h) return 'idle';
  return h.online ? 'ok' : 'err';
}

export function healthText(providerId: string, healthMap: Record<string, ProviderHealth>): string {
  const cls = healthClass(providerId, healthMap);
  if (cls === 'idle') return '未检查';
  return cls === 'ok' ? '在线' : '离线';
}

export function healthLatency(providerId: string, healthMap: Record<string, ProviderHealth>): string {
  const ms = healthMap[providerId]?.latency_ms;
  return ms ? `${ms}ms` : '—';
}

export function healthTime(providerId: string, healthMap: Record<string, ProviderHealth>): string {
  const ts = healthMap[providerId]?.last_checked;
  if (!ts) return '—';
  const date = new Date(ts * 1000);
  return date.toLocaleTimeString();
}
