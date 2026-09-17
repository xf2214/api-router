import { describe, it, expect, vi, afterEach } from 'vitest';
import { writeTextWithFallback } from '../../src/utils/clipboard';
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });
describe('writeTextWithFallback', () => {
  it('clipboard 可用时返回 true', async () => {
    vi.stubGlobal('navigator', { clipboard: { writeText: vi.fn(async () => {}) } });
    await expect(writeTextWithFallback('http://127.0.0.1:6123/v1')).resolves.toBe(true);
  });
  it('clipboard 失败时降级 execCommand 成功返回 true', async () => {
    vi.stubGlobal('navigator', { clipboard: { writeText: vi.fn(async () => { throw new Error('deny'); }) } });
    document.execCommand = vi.fn(() => true);
    await expect(writeTextWithFallback('x')).resolves.toBe(true);
  });
  it('全部失败返回 false', async () => {
    vi.stubGlobal('navigator', {});
    document.execCommand = vi.fn(() => false);
    await expect(writeTextWithFallback('x')).resolves.toBe(false);
  });
  it('clipboard undefined + execCommand true → true', async () => {
    vi.stubGlobal('navigator', {});
    document.execCommand = vi.fn(() => true);
    await expect(writeTextWithFallback('x')).resolves.toBe(true);
  });
  it('execCommand throws → false', async () => {
    vi.stubGlobal('navigator', { clipboard: { writeText: vi.fn(async () => { throw new Error('deny'); }) } });
    document.execCommand = vi.fn(() => { throw new Error('x'); });
    await expect(writeTextWithFallback('x')).resolves.toBe(false);
  });
});
