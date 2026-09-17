import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import OverviewPanel from '../../src/components/OverviewPanel.vue';
import * as clipboard from '../../src/utils/clipboard';
import type { AppConfig, ServerStatus, RequestLog } from '../../src/types';

function makeI18n() {
  return createI18n({
    legacy: false,
    globalInjection: true,
    locale: 'zh',
    fallbackLocale: 'zh',
    missingWarn: false,
    fallbackWarn: false,
    messages: {
      zh: {
        app: { copied: 'app.copied', copyFailed: 'app.copyFailed', copy: 'copy' },
        overviewPanel: {
          serviceRunning: 'running',
          serviceStopped: 'stopped',
          successRate: 'successRate',
          failureCount: 'failureCount',
          totalTokens: 'totalTokens',
          noLogs: 'noLogs',
        },
      },
      en: {},
    },
  });
}

function makeConfig(): AppConfig {
  return {
    port: 6123,
    local_api_token: null,
    enable_logging: false,
    providers: [],
    models: [],
    model_definitions: [],
    groups: [],
    fallback: {
      default_retries: 2,
      timeout_seconds: 60,
      cb_config: {
        failure_threshold: 5,
        failure_threshold_percentage: null,
        cooldown_interval_ms: 60000,
        failure_status_codes: null,
        minimum_requests: 10,
      },
    },
    enable_auto_health_check: true,
    health_check_interval_seconds: 300,
    cache: { enabled: false, mode: 'simple', max_age_seconds: 3600, max_entries: 1000 },
    bind_address: '127.0.0.1',
    enable_cors: true,
    log_level: 'info',
    log_retention_days: 30,
    default_stream: true,
  };
}

function makeProps() {
  const serverStatus: ServerStatus = { running: true, port: 6123 };
  const config: AppConfig = makeConfig();
  const overallStats = { total: 10, success: 9, failure: 1, successRate: 90, totalTokens: 12345 };
  const recentLogs: RequestLog[] = [];
  return { serverStatus, config, overallStats, recentLogs };
}

const flushPromises = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

describe('OverviewPanel', () => {
  it('copy success → show-message success', async () => {
    const spy = vi.spyOn(clipboard, 'writeTextWithFallback').mockResolvedValue(true);
    const wrapper = mount(OverviewPanel, {
      props: makeProps() as never,
      global: { plugins: [makeI18n()] },
    });
    const btn = wrapper.find('button.btn--icon');
    expect(btn.exists()).toBe(true);
    await btn.trigger('click');
    await flushPromises();
    const emitted = wrapper.emitted('show-message') as unknown[][];
    expect(emitted).toBeTruthy();
    expect(emitted[emitted.length - 1][1]).toBe('success');
    expect(emitted[emitted.length - 1][0]).toBe('app.copied');
    spy.mockRestore();
  });

  it('copy failure → show-message error', async () => {
    const spy = vi.spyOn(clipboard, 'writeTextWithFallback').mockResolvedValue(false);
    const wrapper = mount(OverviewPanel, {
      props: makeProps() as never,
      global: { plugins: [makeI18n()] },
    });
    const btn = wrapper.find('button.btn--icon');
    await btn.trigger('click');
    await flushPromises();
    const emitted = wrapper.emitted('show-message') as unknown[][];
    expect(emitted).toBeTruthy();
    expect(emitted[emitted.length - 1][1]).toBe('error');
    expect(emitted[emitted.length - 1][0]).toBe('app.copyFailed');
    spy.mockRestore();
  });

  it('copy button exists with title attribute', () => {
    const wrapper = mount(OverviewPanel, {
      props: makeProps() as never,
      global: { plugins: [makeI18n()] },
    });
    const btn = wrapper.find('button.btn--icon');
    expect(btn.exists()).toBe(true);
    expect(btn.attributes('title')).toBeTruthy();
  });
});
