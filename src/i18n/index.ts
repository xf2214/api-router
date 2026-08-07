import { createI18n } from 'vue-i18n';
import zhCN from './locales/zh-CN';
import enUS from './locales/en-US';

type LocaleKey = 'zh' | 'en';

const normalizeLocale = (l: string | null): LocaleKey => {
  if (!l) return 'zh';
  if (l === 'zh' || l.startsWith('zh')) return 'zh';
  if (l === 'en' || l.startsWith('en')) return 'en';
  return 'zh';
};

// Deep-clone messages to avoid sharing references between locale aliases
const cloneMessages = <T>(obj: T): T => JSON.parse(JSON.stringify(obj));

// Fix any stale/long locale keys (zh-CN, en-US) stored in localStorage before init
const STORAGE_KEY = 'api-router-locale';
const rawStored = (() => {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
})();
const finalLocale: LocaleKey = normalizeLocale(rawStored);
try {
  localStorage.setItem(STORAGE_KEY, finalLocale);
} catch {
  /* ignore */
}

const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: finalLocale,
  fallbackLocale: 'zh',
  missingWarn: true,
  fallbackWarn: false,
  messages: {
    'zh': cloneMessages(zhCN),
    'en': cloneMessages(enUS),
  },
});

export function setLocale(locale: string): void {
  const normalized: LocaleKey = normalizeLocale(locale);
  i18n.global.locale.value = normalized;
  try {
    localStorage.setItem(STORAGE_KEY, normalized);
  } catch {
    /* ignore */
  }
}

export function getLocale(): string {
  return i18n.global.locale.value;
}

export default i18n;