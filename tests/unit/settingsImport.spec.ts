import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../src/services/tauri/config', () => ({
  importConfig: vi.fn(),
  getConfig: vi.fn(),
  exportConfig: vi.fn(),
}));

import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import SettingsPage from '../../src/pages/SettingsPage.vue';
import { importConfig, getConfig } from '../../src/services/tauri/config';
import type { AppConfig, ServerStatus } from '../../src/types';

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
        app: { copied: 'copied', copyFailed: 'copyFailed', copy: 'copy' },
        routing: { saved: 'saved' },
        settings: {
          title: 'settings',
          desc: 'desc',
          includeKeys: 'includeKeys',
          resetConfirm: 'reset?',
          configExported: 'exported',
          exportFailed: 'exportFailed {error}',
          importConfig: 'import',
          service: {
            title: 'service',
            desc: 'desc',
            running: 'running',
            stopped: 'stopped',
            port: 'port',
            portHint: 'hint',
            bindAddress: 'bind',
            bindAddressHint: 'hint',
            tokenAuth: 'auth',
            tokenAuthDesc: 'desc',
            apiToken: 'token',
            showToken: 'show',
            hideToken: 'hide',
            enableCors: 'cors',
            corsDesc: 'desc',
            endpoint: 'endpoint',
          },
          routing: {
            title: 'routing',
            desc: 'desc',
            defaultRetries: 'retries',
            retriesHint: 'hint',
            defaultTimeout: 'timeout',
            timeoutHint: 'hint',
            autoFallback: 'fallback',
            fallbackHint: 'hint',
            defaultStream: 'stream',
            defaultStreamDesc: 'desc',
          },
          cache: {
            title: 'cache',
            desc: 'desc',
            mode: 'mode',
            modeSimple: 'simple',
            modeTTL: 'TTL',
            modeLRU: 'LRU',
            modeHint: 'hint',
            ttl: 'ttl',
            ttlHint: 'hint',
            maxEntries: 'max',
            maxEntriesHint: 'hint',
            entries: 'entries',
            hitRate: 'hitRate',
            lastCleanup: 'cleanup',
            clear: 'clear',
          },
          circuitBreaker: {
            title: 'cb',
            desc: 'desc',
            failureThreshold: 'thr',
            failurePercentage: 'pct',
            minRequests: 'min',
            cooldown: 'cd',
            statusCodes: 'codes',
            statusCodesHint: 'hint',
            codePlaceholder: 'placeholder',
          },
          healthCheck: {
            title: 'hc',
            desc: 'desc',
            auto: 'auto',
            autoDesc: 'desc',
            interval: 'interval',
            intervalHint: 'hint',
          },
          logs: {
            title: 'logs',
            desc: 'desc',
            detailed: 'detailed',
            detailedDesc: 'desc',
            level: 'level',
            retention: 'ret',
            retention7d: '7d',
            retention14d: '14d',
            retention30d: '30d',
            retention90d: '90d',
          },
          language: { title: 'lang', desc: 'desc', zhCN: 'zh', enUS: 'en' },
          about: {
            title: 'about',
            desc: 'desc',
            tag: 'tag',
            localFirst: 'local',
            architecture: 'arch',
            platform: 'platform',
            techStack: 'stack',
            platforms: 'platforms',
            checkUpdate: 'check',
            viewSource: 'source',
            upToDate: 'up',
            openingGithub: 'gh',
            copyright: 'cp',
          },
        },
      },
      en: {},
    },
  });
}

function makeConfig(overrides: Partial<AppConfig> = {}): AppConfig {
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
    ...overrides,
  };
}

function makeProps() {
  const config = makeConfig();
  const serverStatus: ServerStatus = { running: false, port: 6123 };
  const serverUrl = 'http://127.0.0.1:6123';
  return { config, serverStatus, serverUrl };
}

const flushPromises = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

describe('SettingsPage onImportFile', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('import success → emits save-settings with fresh config and show-message success with counts', async () => {
    const fresh = makeConfig({ port: 9999 });
    vi.mocked(importConfig).mockResolvedValue({ providers: 2, models: 5, backup_path: '/tmp/bak' });
    vi.mocked(getConfig).mockResolvedValue(fresh);

    const wrapper = mount(SettingsPage, {
      props: makeProps() as never,
      global: { plugins: [makeI18n()] },
    });

    const jsonStr = JSON.stringify({ port: 9999 });
    const file = new File([jsonStr], 'config.json', { type: 'application/json' });
    (file as unknown as { text: () => Promise<string> }).text = async () => jsonStr;
    const input = wrapper.find('input[type="file"]');
    expect(input.exists()).toBe(true);
    Object.defineProperty(input.element, 'files', { value: [file], writable: false, configurable: true });
    await input.trigger('change');
    await flushPromises();
    await flushPromises();

    const saveEmitted = wrapper.emitted('save-settings') as unknown[][];
    expect(saveEmitted).toBeTruthy();
    expect(saveEmitted[0][0]).toEqual(fresh);

    const msgEmitted = wrapper.emitted('show-message') as unknown[][];
    expect(msgEmitted).toBeTruthy();
    const successEntry = msgEmitted.find((e) => e[1] === 'success');
    expect(successEntry).toBeTruthy();
    expect(String(successEntry![0])).toContain('2');
    expect(String(successEntry![0])).toContain('5');
  });

  it('import failure → show-message error', async () => {
    vi.mocked(importConfig).mockRejectedValue(new Error('import fail'));

    const wrapper = mount(SettingsPage, {
      props: makeProps() as never,
      global: { plugins: [makeI18n()] },
    });

    const badStr = 'not json';
    const file = new File([badStr], 'bad.json', { type: 'application/json' });
    (file as unknown as { text: () => Promise<string> }).text = async () => badStr;
    const input = wrapper.find('input[type="file"]');
    Object.defineProperty(input.element, 'files', { value: [file], writable: false, configurable: true });
    await input.trigger('change');
    await flushPromises();
    await flushPromises();

    const msgEmitted = wrapper.emitted('show-message') as unknown[][];
    expect(msgEmitted).toBeTruthy();
    const errEntry = msgEmitted.find((e) => e[1] === 'error');
    expect(errEntry).toBeTruthy();
    expect(wrapper.emitted('save-settings')).toBeUndefined();
  });
});
