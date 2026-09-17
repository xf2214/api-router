<template>
  <section class="page">
    <header class="ph">
      <div>
        <h1 class="ph__title">{{ $t('settings.title') }}</h1>
        <p class="ph__desc">{{ $t('settings.desc') }}</p>
      </div>
      <div class="ph__actions">
        <label style="display: inline-flex; align-items: center; gap: 6px; font-size: 12px; cursor: pointer; user-select: none;">
          <input v-model="includeKeys" type="checkbox" />
          {{ $t('settings.includeKeys') }}
        </label>
        <button class="btn btn--ghost btn--sm" @click="resetSettings">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>
          </svg>
          {{ $t('app.resetDefaults') }}
        </button>
        <input ref="importFile" type="file" accept="application/json,.json" class="hidden" @change="onImportFile" />
        <button class="btn btn--ghost btn--sm" :disabled="importing" @click="importFile?.click()">
          <svg v-if="!importing" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="15" height="15" class="spin-anim" style="transform-origin: center;">
            <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
          </svg>
          {{ $t('settings.importConfig') }}
        </button>
        <button class="btn btn--ghost btn--sm" :disabled="exporting" @click="exportSettings">
          <svg v-if="!exporting" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="15" height="15" class="spin-anim" style="transform-origin: center;">
            <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
          </svg>
          {{ $t('app.exportConfig') }}
        </button>
        <button class="btn btn--primary" :disabled="saving" @click="saveSettings">
          <svg v-if="!saving" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="16" height="16">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><path d="M17 21v-8H7v8"/><path d="M7 3v5h8"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16" class="spin-anim" style="transform-origin: center;">
            <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
          </svg>
          {{ $t('app.saveSettings') }}
        </button>
      </div>
    </header>

    <div v-if="importError" class="info-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
        <circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>
      </svg>
      <span>{{ importError }}</span>
    </div>

    <div style="display: flex; flex-direction: column; gap: 18px">
      <!-- Service -->
      <div class="card card--pad">
        <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
          <div class="row">
            <span class="op-70">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><rect x="3" y="4" width="18" height="8" rx="2"/><rect x="3" y="12" width="18" height="8" rx="2"/><line x1="7" y1="8" x2="7.01" y2="8"/><line x1="7" y1="16" x2="7.01" y2="16"/></svg>
            </span>
            <div>
              <div class="card__title">{{ $t('settings.service.title') }}</div>
              <div class="card__sub">{{ $t('settings.service.desc') }}</div>
            </div>
          </div>
          <div class="spacer" />
          <span class="badge" :class="serverStatus.running ? 'badge--ok' : 'badge--err'"><span class="dot" :class="serverStatus.running ? 'dot--ok' : 'dot--err'" />{{ serverStatus.running ? $t('settings.service.running') : $t('settings.service.stopped') }}</span>
        </div>

        <div class="grid-cols cols-2 mb-16">
          <div class="field">
            <span class="field__label">{{ $t('settings.service.port') }}</span>
            <input v-model.number="settings.port" class="input input--mono" inputmode="numeric">
            <span class="field__hint">{{ $t('settings.service.portHint') }}</span>
          </div>
          <div class="field">
            <span class="field__label">{{ $t('settings.service.bindAddress') }}</span>
            <input v-model="settings.bind_address" class="input input--mono" placeholder="127.0.0.1">
            <span class="field__hint">{{ $t('settings.service.bindAddressHint') }}</span>
          </div>
        </div>

        <div class="set-row">
          <div class="set-row__txt">
            <div class="set-row__t">{{ $t('settings.service.tokenAuth') }}</div>
            <div class="set-row__d">{{ $t('settings.service.tokenAuthDesc') }}</div>
          </div>
          <label class="switch">
            <input v-model="tokenAuthEnabled" type="checkbox">
            <span class="switch__track"><span class="switch__thumb" /></span>
          </label>
        </div>

        <div v-if="tokenAuthEnabled" class="field mt-12">
          <span class="field__label">{{ $t('settings.service.apiToken') }}</span>
          <div class="row">
            <input
              v-model="settings.local_api_token"
              :type="tokenVisible ? 'text' : 'password'"
              class="input input--mono grow"
              placeholder="sk-..."
            >
            <button
              type="button"
              class="btn btn--ghost btn--icon"
              :title="tokenVisible ? $t('settings.service.hideToken') : $t('settings.service.showToken')"
              @click="tokenVisible = !tokenVisible"
            >
              <svg
                v-if="tokenVisible"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
                <circle cx="12" cy="12" r="3" />
                <line x1="4" y1="4" x2="20" y2="20" />
              </svg>
              <svg
                v-else
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>
            <button type="button" class="btn btn--ghost btn--icon" :title="$t('app.copy')" @click="copyToken">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            </button>
          </div>
        </div>

        <div class="set-row">
          <div class="set-row__txt">
            <div class="set-row__t">{{ $t('settings.service.enableCors') }}</div>
            <div class="set-row__d">{{ $t('settings.service.corsDesc') }}</div>
          </div>
          <label class="switch">
            <input v-model="settings.enable_cors" type="checkbox">
            <span class="switch__track"><span class="switch__thumb" /></span>
          </label>
        </div>

        <div class="row mt-12" style="gap: 8px; flex-wrap: wrap;">
          <span class="muted" style="font-size: 12px">{{ $t('settings.service.endpoint') }}</span>
          <span class="endpoint" style="display: inline-flex; align-items: center; gap: 4px;">
            <span class="endpoint__url">{{ serverUrl }}/v1</span>
            <button type="button" class="btn btn--ghost btn--icon" :title="$t('app.copy')" @click="copyEndpoint">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            </button>
          </span>
        </div>
      </div>

      <!-- Routing defaults -->
      <div class="card card--pad">
        <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
          <div class="row">
            <span class="op-70">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="6" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M8.2 7.2 16 16"/><path d="M7.5 8 11 15.5"/><path d="M15.8 7.5 13 15.5"/></svg>
            </span>
            <div>
              <div class="card__title">{{ $t('settings.routing.title') }}</div>
              <div class="card__sub">{{ $t('settings.routing.desc') }}</div>
            </div>
          </div>
        </div>

        <div class="grid-cols cols-3 mb-16">
          <div class="field">
            <span class="field__label">{{ $t('settings.routing.defaultRetries') }}</span>
            <input v-model.number="settings.fallback.default_retries" class="input input--mono" inputmode="numeric">
            <span class="field__hint">{{ $t('settings.routing.retriesHint') }}</span>
          </div>
          <div class="field">
            <span class="field__label">{{ $t('settings.routing.defaultTimeout') }}</span>
            <input v-model.number="settings.fallback.timeout_seconds" class="input input--mono" inputmode="numeric">
            <span class="field__hint">{{ $t('settings.routing.timeoutHint') }}</span>
          </div>
          <div class="field">
            <span class="field__label">{{ $t('settings.routing.autoFallback') }}</span>
            <div class="row" style="margin-top: 6px">
              <label class="switch">
                <input v-model="settings.fallback_enabled" type="checkbox">
                <span class="switch__track"><span class="switch__thumb" /></span>
              </label>
              <span class="muted" style="font-size: 12px">{{ $t('settings.routing.fallbackHint') }}</span>
            </div>
          </div>
        </div>

        <div class="set-row">
          <div class="set-row__txt">
            <div class="set-row__t">{{ $t('settings.routing.defaultStream') }}</div>
            <div class="set-row__d">{{ $t('settings.routing.defaultStreamDesc') }}</div>
          </div>
          <label class="switch">
            <input v-model="settings.default_stream" type="checkbox">
            <span class="switch__track"><span class="switch__thumb" /></span>
          </label>
        </div>
      </div>

      <!-- Cache -->
      <div class="card card--pad">
        <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
          <div class="row">
            <span class="op-70">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"/></svg>
            </span>
            <div>
              <div class="card__title">{{ $t('settings.cache.title') }}</div>
              <div class="card__sub">{{ $t('settings.cache.desc') }}</div>
            </div>
          </div>
          <div class="spacer" />
          <label class="switch">
            <input v-model="settings.cache.enabled" type="checkbox">
            <span class="switch__track"><span class="switch__thumb" /></span>
          </label>
        </div>

        <div class="grid-cols cols-3 mb-16">
          <div class="field">
            <span class="field__label">{{ $t('settings.cache.mode') }}</span>
            <select v-model="settings.cache.mode" class="select">
              <option value="simple">{{ $t('settings.cache.modeSimple') }}</option>
              <option value="TTL">{{ $t('settings.cache.modeTTL') }}</option>
              <option value="LRU">{{ $t('settings.cache.modeLRU') }}</option>
            </select>
            <span class="field__hint">{{ $t('settings.cache.modeHint') }}</span>
          </div>
          <div class="field">
            <span class="field__label">{{ $t('settings.cache.ttl') }}</span>
            <input v-model.number="settings.cache.max_age_seconds" class="input input--mono" inputmode="numeric">
            <span class="field__hint">{{ $t('settings.cache.ttlHint') }}</span>
          </div>
          <div class="field">
            <span class="field__label">{{ $t('settings.cache.maxEntries') }}</span>
            <input v-model.number="settings.cache.max_entries" class="input input--mono" inputmode="numeric">
            <span class="field__hint">{{ $t('settings.cache.maxEntriesHint') }}</span>
          </div>
        </div>

        <div class="cache-card">
          <div class="cache-stat"><span class="cache-stat__v">{{ cacheStats.entries }}</span><span class="cache-stat__l">{{ $t('settings.cache.entries') }}</span></div>
          <div class="cache-divider" />
          <div class="cache-stat"><span class="cache-stat__v">{{ cacheStats.hit_rate }}%</span><span class="cache-stat__l">{{ $t('settings.cache.hitRate') }}</span></div>
          <div class="cache-divider" />
          <div class="cache-stat"><span class="cache-stat__v">{{ cacheStats.last_cleanup }}</span><span class="cache-stat__l">{{ $t('settings.cache.lastCleanup') }}</span></div>
          <div class="spacer" />
          <button class="btn btn--danger btn--sm" disabled :title="$t('settings.cache.clear')" @click="clearCache">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
              <path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
            {{ $t('settings.cache.clear') }}
          </button>
        </div>
      </div>

      <!-- Circuit breaker -->
      <div class="card card--pad">
        <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
          <div class="row">
            <span class="op-70">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/></svg>
            </span>
            <div>
              <div class="card__title">{{ $t('settings.circuitBreaker.title') }}</div>
              <div class="card__sub">{{ $t('settings.circuitBreaker.desc') }}</div>
            </div>
          </div>
        </div>

        <div class="cb-panel">
          <div class="cb-panel__row">
            <div class="field">
              <span class="field__label">{{ $t('settings.circuitBreaker.failureThreshold') }}</span>
              <input v-model.number="settings.fallback.cb_config.failure_threshold" class="input input--mono" inputmode="numeric">
            </div>
            <div class="field">
              <span class="field__label">{{ $t('settings.circuitBreaker.failurePercentage') }}</span>
              <input v-model.number="cbFailurePercentage" class="input input--mono" inputmode="numeric">
            </div>
          </div>
          <div class="cb-panel__row">
            <div class="field">
              <span class="field__label">{{ $t('settings.circuitBreaker.minRequests') }}</span>
              <input v-model.number="settings.fallback.cb_config.minimum_requests" class="input input--mono" inputmode="numeric">
            </div>
            <div class="field">
              <span class="field__label">{{ $t('settings.circuitBreaker.cooldown') }}</span>
              <input v-model.number="settings.fallback.cb_config.cooldown_interval_ms" class="input input--mono" inputmode="numeric">
            </div>
          </div>
        </div>

        <div class="field mt-16">
          <span class="field__label">{{ $t('settings.circuitBreaker.statusCodes') }}</span>
          <div class="tag-input">
            <span v-for="(code, i) in cbFailureCodes" :key="i" class="retry-code-chip">
              {{ code }}
              <span class="chip__x" @click="removeCbCode(i)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="11" height="11"><path d="M18 6 6 18M6 6l12 12"/></svg>
              </span>
            </span>
            <input v-model="cbCodeInput" class="tag-input__input" :placeholder="$t('settings.circuitBreaker.codePlaceholder')" @keydown.enter.prevent="addCbCode">
          </div>
          <span class="field__hint">{{ $t('settings.circuitBreaker.statusCodesHint') }}</span>
        </div>
      </div>

      <!-- Health check + Logs -->
      <div class="grid-cols cols-2">
        <div class="card card--pad">
          <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
            <div class="row">
              <span class="op-70">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
              </span>
              <div>
                <div class="card__title">{{ $t('settings.healthCheck.title') }}</div>
                <div class="card__sub">{{ $t('settings.healthCheck.desc') }}</div>
              </div>
            </div>
          </div>
          <div class="set-row">
            <div class="set-row__txt">
              <div class="set-row__t">{{ $t('settings.healthCheck.auto') }}</div>
              <div class="set-row__d">{{ $t('settings.healthCheck.autoDesc') }}</div>
            </div>
            <label class="switch">
              <input v-model="settings.enable_auto_health_check" type="checkbox">
              <span class="switch__track"><span class="switch__thumb" /></span>
            </label>
          </div>
          <div class="field mt-12">
            <span class="field__label">{{ $t('settings.healthCheck.interval') }}</span>
            <input v-model.number="settings.health_check_interval_seconds" class="input input--mono" inputmode="numeric">
            <span class="field__hint">{{ $t('settings.healthCheck.intervalHint') }}</span>
          </div>
        </div>

        <div class="card card--pad">
          <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
            <div class="row">
              <span class="op-70">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M9 13h6"/><path d="M9 17h4"/></svg>
              </span>
              <div>
                <div class="card__title">{{ $t('settings.logs.title') }}</div>
                <div class="card__sub">{{ $t('settings.logs.desc') }}</div>
              </div>
            </div>
          </div>
          <div class="set-row">
            <div class="set-row__txt">
              <div class="set-row__t">{{ $t('settings.logs.detailed') }}</div>
              <div class="set-row__d">{{ $t('settings.logs.detailedDesc') }}</div>
            </div>
            <label class="switch">
              <input v-model="settings.enable_logging" type="checkbox">
              <span class="switch__track"><span class="switch__thumb" /></span>
            </label>
          </div>
          <div class="grid-cols cols-2">
            <div class="field">
              <span class="field__label">{{ $t('settings.logs.level') }}</span>
              <select v-model="settings.log_level" class="select">
                <option value="trace">trace</option>
                <option value="debug">debug</option>
                <option value="info">info</option>
                <option value="warn">warn</option>
                <option value="error">error</option>
              </select>
            </div>
            <div class="field">
              <span class="field__label">{{ $t('settings.logs.retention') }}</span>
              <select v-model="settings.log_retention" class="select">
                <option value="7d">{{ $t('settings.logs.retention7d') }}</option>
                <option value="14d">{{ $t('settings.logs.retention14d') }}</option>
                <option value="30d">{{ $t('settings.logs.retention30d') }}</option>
                <option value="90d">{{ $t('settings.logs.retention90d') }}</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <!-- Language -->
      <div class="card card--pad">
        <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
          <div class="row">
            <span class="op-70">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
            </span>
            <div>
              <div class="card__title">{{ $t('settings.language.title') }}</div>
              <div class="card__sub">{{ $t('settings.language.desc') }}</div>
            </div>
          </div>
        </div>
        <div class="row" style="gap: 12px;">
          <label class="btn btn--ghost" :class="{ 'btn--primary': currentLocale === 'zh' }" style="cursor: pointer; padding: 6px 16px; border-radius: 8px;">
            <input type="radio" name="locale" value="zh" :checked="currentLocale === 'zh'" @change="switchLocale('zh')" style="display: none;">
            {{ $t('settings.language.zhCN') }}
          </label>
          <label class="btn btn--ghost" :class="{ 'btn--primary': currentLocale === 'en' }" style="cursor: pointer; padding: 6px 16px; border-radius: 8px;">
            <input type="radio" name="locale" value="en" :checked="currentLocale === 'en'" @change="switchLocale('en')" style="display: none;">
            {{ $t('settings.language.enUS') }}
          </label>
        </div>
      </div>

      <!-- About -->
      <div class="card card--pad">
        <div class="card__head" style="padding: 0 0 16px; margin-bottom: 6px; border-bottom: 1px solid var(--border)">
          <div class="row">
            <span class="op-70">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
            </span>
            <div>
              <div class="card__title">{{ $t('settings.about.title') }}</div>
              <div class="card__sub">{{ $t('settings.about.desc') }}</div>
            </div>
          </div>
        </div>

        <div class="row" style="gap: 12px; flex-wrap: wrap">
          <span>
            <svg viewBox="0 0 24 24" fill="none" stroke="var(--primary)" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="26" height="26">
              <rect x="3" y="3" width="18" height="18" rx="6"/><circle cx="8" cy="12" r="1.3"/><path d="M9.3 12h6"/><path d="M15 9l3 3-3 3"/>
            </svg>
          </span>
          <span class="card__title">API Router</span>
          <span class="tag">v0.1.0</span>
          <span class="badge badge--info">{{ $t('settings.about.tag') }}</span>
          <div class="spacer" />
          <span class="muted">{{ $t('settings.about.localFirst') }}</span>
        </div>

        <div class="row mt-12" style="gap: 8px; flex-wrap: wrap">
          <span class="eyebrow">{{ $t('settings.about.architecture') }}</span><span class="muted" style="font-size: 12px">{{ $t('settings.about.techStack') }}</span>
          <span class="muted">·</span>
          <span class="eyebrow">{{ $t('settings.about.platform') }}</span><span class="muted" style="font-size: 12px">{{ $t('settings.about.platforms') }}</span>
        </div>

        <hr class="divider">

        <div class="row" style="gap: 10px; flex-wrap: wrap">
          <button class="btn btn--ghost btn--sm" @click="emit('show-message', t('settings.about.upToDate'), 'success')">{{ $t('settings.about.checkUpdate') }}</button>
          <button class="btn btn--ghost btn--sm" @click="emit('show-message', t('settings.about.openingGithub'), 'info')">{{ $t('settings.about.viewSource') }}</button>
          <div class="spacer" />
          <span class="muted" style="font-size: 11.5px">{{ $t('settings.about.copyright') }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { exportConfig, getConfig, importConfig } from '../services/tauri';
import type { AppConfig, ServerStatus, CircuitBreakerConfig, CacheConfig } from '../types';
import { setLocale, getLocale } from '../i18n';

const { t } = useI18n();

const currentLocale = ref(getLocale());

function switchLocale(locale: string): void {
  setLocale(locale);
  currentLocale.value = locale;
}

interface ExtendedSettings {
  port: number;
  local_api_token: string;
  enable_logging: boolean;
  fallback: { default_retries: number; timeout_seconds: number; cb_config: CircuitBreakerConfig };
  enable_auto_health_check: boolean;
  health_check_interval_seconds: number;
  cache: CacheConfig;
  bind_address: string;
  enable_cors: boolean;
  fallback_enabled: boolean;
  log_level: string;
  log_retention: string;
  default_stream: boolean;
}

interface Props {
  config: AppConfig;
  serverStatus: ServerStatus;
  serverUrl: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'save-settings': [config: AppConfig];
  'show-message': [text: string, type: 'success' | 'error' | 'warn' | 'info'];
}>();

const defaultCbConfig: CircuitBreakerConfig = {
  failure_threshold: 5,
  failure_threshold_percentage: null,
  cooldown_interval_ms: 60000,
  failure_status_codes: null,
  minimum_requests: 10,
};

const defaultCacheConfig: CacheConfig = {
  enabled: false,
  mode: 'simple',
  max_age_seconds: 3600,
  max_entries: 1000,
};

const RETENTION_OPTIONS: Record<string, number> = {
  '7d': 7,
  '14d': 14,
  '30d': 30,
  '90d': 90,
};

function retentionOptionToDays(option: string): number {
  return RETENTION_OPTIONS[option] ?? 30;
}

function retentionDaysToOption(days: number): string {
  for (const [opt, value] of Object.entries(RETENTION_OPTIONS)) {
    if (value === days) return opt;
  }
  return '30d';
}

function defaultSettings(): ExtendedSettings {
  return {
    port: 6123,
    local_api_token: '',
    enable_logging: false,
    fallback: { default_retries: 2, timeout_seconds: 60, cb_config: { ...defaultCbConfig } },
    enable_auto_health_check: true,
    health_check_interval_seconds: 300,
    cache: { ...defaultCacheConfig },
    bind_address: '127.0.0.1',
    enable_cors: true,
    fallback_enabled: true,
    log_level: 'info',
    log_retention: '30d',
    default_stream: true,
  };
}

const settings = reactive<ExtendedSettings>(defaultSettings());
const tokenAuthEnabled = ref(false);
const tokenVisible = ref(false);
const cbFailureCodes = ref<number[]>([500, 502, 503, 504]);
const cbFailurePercentage = ref(50);
const cbCodeInput = ref('');
const cacheStats = ref({ entries: 0, hit_rate: 0, last_cleanup: '—' });
const importFile = ref<HTMLInputElement | null>(null);
const includeKeys = ref(true);
const saving = ref(false);
const exporting = ref(false);
const importing = ref(false);
const importError = ref<string | null>(null);

function syncFromConfig(): void {
  const c = props.config;
  settings.port = c.port ?? 6123;
  settings.local_api_token = c.local_api_token ?? '';
  settings.enable_logging = c.enable_logging ?? false;
  settings.fallback = {
    default_retries: c.fallback?.default_retries ?? 2,
    timeout_seconds: c.fallback?.timeout_seconds ?? 60,
    cb_config: c.fallback?.cb_config
      ? { ...defaultCbConfig, ...c.fallback.cb_config }
      : { ...defaultCbConfig },
  };
  settings.enable_auto_health_check = c.enable_auto_health_check ?? true;
  settings.health_check_interval_seconds = c.health_check_interval_seconds ?? 300;
  settings.cache = c.cache ? { ...defaultCacheConfig, ...c.cache } : { ...defaultCacheConfig };
  settings.bind_address = c.bind_address ?? '127.0.0.1';
  settings.enable_cors = c.enable_cors ?? true;
  settings.log_level = c.log_level ?? 'info';
  settings.log_retention = retentionDaysToOption(c.log_retention_days ?? 30);
  settings.default_stream = c.default_stream ?? true;

  tokenAuthEnabled.value = !!c.local_api_token;
  cbFailureCodes.value = Array.isArray(settings.fallback.cb_config.failure_status_codes)
    ? [...settings.fallback.cb_config.failure_status_codes]
    : [500, 502, 503, 504];
  cbFailurePercentage.value = settings.fallback.cb_config.failure_threshold_percentage ?? 50;
}

watch(() => props.config, syncFromConfig, { immediate: true });

function buildConfig(): AppConfig {
  return {
    ...props.config,
    port: settings.port,
    local_api_token: tokenAuthEnabled.value ? settings.local_api_token || null : null,
    enable_logging: settings.enable_logging,
    fallback: {
      default_retries: settings.fallback.default_retries,
      timeout_seconds: settings.fallback.timeout_seconds,
      cb_config: {
        ...settings.fallback.cb_config,
        failure_threshold_percentage: cbFailurePercentage.value,
        failure_status_codes: cbFailureCodes.value.length > 0 ? [...cbFailureCodes.value] : null,
      },
    },
    enable_auto_health_check: settings.enable_auto_health_check,
    health_check_interval_seconds: settings.health_check_interval_seconds,
    cache: { ...settings.cache },
    bind_address: settings.bind_address || '127.0.0.1',
    enable_cors: settings.enable_cors,
    log_level: settings.log_level,
    log_retention_days: retentionOptionToDays(settings.log_retention),
    default_stream: settings.default_stream,
  };
}

function saveSettings(): void {
  saving.value = true;
  try {
    emit('save-settings', buildConfig());
  } finally {
    saving.value = false;
  }
}

function resetSettings(): void {
  if (!confirm(t('settings.resetConfirm'))) return;
  Object.assign(settings, defaultSettings());
  tokenAuthEnabled.value = false;
  cbFailureCodes.value = [500, 502, 503, 504];
  cbFailurePercentage.value = 50;
  cacheStats.value = { entries: 0, hit_rate: 0, last_cleanup: '—' };
}

async function exportSettings(): Promise<void> {
  exporting.value = true;
  try {
    try {
      const json = await exportConfig(includeKeys.value);
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      const now = new Date();
      const ts = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}-${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`;
      a.download = `api-router-config-${ts}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
      emit('show-message', t('settings.configExported'), 'success');
    } catch (error) {
      emit('show-message', t('settings.exportFailed', { error: String(error) }), 'error');
    }
  } finally {
    exporting.value = false;
  }
}

async function onImportFile(e: Event): Promise<void> {
  importing.value = true;
  try {
    const f = (e.target as HTMLInputElement).files?.[0];
    if (!f) return;
    try {
      const text: string = await f.text();
      const summary = await importConfig(text);
      const fresh: AppConfig = await getConfig();
      emit('save-settings', fresh);
      emit('show-message', `${t('routing.saved')} ${summary.providers}/${summary.models}`, 'success');
      importError.value = null;
    } catch (err) {
      emit('show-message', String(err), 'error');
      importError.value = String(err).slice(0, 120);
    } finally {
      (e.target as HTMLInputElement).value = '';
    }
  } finally {
    importing.value = false;
  }
}

function addCbCode(): void {
  const raw = cbCodeInput.value.trim();
  if (!raw) return;
  const codes = raw.split(/[,\s]+/).map((s) => parseInt(s, 10)).filter((n) => !isNaN(n) && n > 0);
  for (const c of codes) {
    if (!cbFailureCodes.value.includes(c)) cbFailureCodes.value.push(c);
  }
  cbCodeInput.value = '';
}

function removeCbCode(index: number): void {
  cbFailureCodes.value.splice(index, 1);
}

function clearCache(): void {
  emit('show-message', t('settings.cache.clear'), 'warn');
}

function copyEndpoint(): void {
  navigator.clipboard?.writeText(`${props.serverUrl}/v1`).catch(() => {});
  emit('show-message', t('app.copied'), 'success');
}

async function copyToken(): Promise<void> {
  try {
    await navigator.clipboard?.writeText(settings.local_api_token);
    emit('show-message', t('app.copied'), 'success');
  } catch {
    emit('show-message', t('app.copyFailed'), 'error');
  }
}
</script>
