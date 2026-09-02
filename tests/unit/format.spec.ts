import { describe, it, expect } from "vitest";
import {
  formatDuration,
  formatTokens,
  formatTokensCompact,
  formatTime,
  strategyLabel,
  strategyBadgeClass,
  sortedTargets,
  statusCategory,
  statusBadgeClass,
  successRateClass,
  successBadgeClass,
  statSuccessRate,
  statAvgLatency,
  providerColor,
  providerGlyph,
} from "../../src/utils/format";

// vitest smoke tests for src/utils/format.ts — only assert behaviours that are actually true

describe("formatDuration", () => {
  it("formats <1000 as ms", () => {
    expect(formatDuration(0)).toBe("0ms");
    expect(formatDuration(999)).toBe("999ms");
    expect(formatDuration(1)).toBe("1ms");
  });
  it("formats >=1000 as seconds with 2 decimals", () => {
    expect(formatDuration(1000)).toBe("1.00s");
    expect(formatDuration(1500)).toBe("1.50s");
    expect(formatDuration(12345)).toBe("12.35s");
  });
});

describe("formatTokens", () => {
  it("returns 0 for undefined/null/0", () => {
    expect(formatTokens(undefined)).toBe("0");
    expect(formatTokens(null)).toBe("0");
    expect(formatTokens(0)).toBe("0");
  });
  it("formats thousands as k with 1 decimal", () => {
    expect(formatTokens(1000)).toBe("1.0k");
    expect(formatTokens(1500)).toBe("1.5k");
    expect(formatTokens(999999)).toBe("1000.0k"); // 999999/1000 = 999.999 -> 1000.0k
  });
  it("formats millions as M with 2 decimals", () => {
    expect(formatTokens(1_000_000)).toBe("1.00M");
    expect(formatTokens(2_500_000)).toBe("2.50M");
  });
  it("returns locale string for small numbers", () => {
    expect(formatTokens(42)).toBe("42");
    expect(formatTokens(999)).toBe("999");
  });
});

describe("formatTokensCompact", () => {
  it("mirrors formatTokens but uses String for small values", () => {
    expect(formatTokensCompact(undefined)).toBe("0");
    expect(formatTokensCompact(0)).toBe("0");
    expect(formatTokensCompact(1_000)).toBe("1.0k");
    expect(formatTokensCompact(1_000_000)).toBe("1.00M");
    expect(formatTokensCompact(7)).toBe("7");
  });
});

describe("formatTime", () => {
  it("returns '-' for falsy ms", () => {
    expect(formatTime(0)).toBe("-");
  });
  it("returns a time string for truthy ms", () => {
    const s = formatTime(Date.now());
    expect(typeof s).toBe("string");
    expect(s.length).toBeGreaterThan(0);
    expect(s).not.toBe("-");
  });
});

describe("strategyLabel / strategyBadgeClass", () => {
  it("maps strategy to Chinese label", () => {
    expect(strategyLabel("priority")).toBe("顺序优先");
    expect(strategyLabel("weighted")).toBe("按权重");
    expect(strategyLabel("round-robin")).toBe("轮询");
  });
  it("maps strategy to badge class", () => {
    expect(strategyBadgeClass("priority")).toBe("badge--mute");
    expect(strategyBadgeClass("weighted")).toBe("badge--info");
    expect(strategyBadgeClass("round-robin")).toBe("badge--info");
  });
});

describe("sortedTargets", () => {
  it("sorts by tier asc then weight desc without mutating input", () => {
    const input = [
      { providerId: "b", modelId: "m", tier: 2, weight: 1 },
      { providerId: "a", modelId: "m", tier: 1, weight: 5 },
      { providerId: "c", modelId: "m", tier: 1, weight: 10 },
    ] as any;
    const sorted = sortedTargets(input);
    expect(sorted.map((t) => t.providerId)).toEqual(["c", "a", "b"]);
    // original not mutated
    expect(input[0].providerId).toBe("b");
  });
  it("defaults tier 1 and weight 1 when missing", () => {
    const sorted = sortedTargets([
      { providerId: "x", modelId: "m" },
      { providerId: "y", modelId: "m", tier: 2 },
    ] as any);
    expect(sorted[0].providerId).toBe("x");
  });
});

describe("statusCategory / statusBadgeClass", () => {
  it("success true -> success", () => {
    expect(statusCategory({ success: true } as any)).toBe("success");
    expect(statusBadgeClass({ success: true } as any)).toBe("badge--ok");
  });
  it("success false + 5xx", () => {
    expect(statusCategory({ success: false, status: 500 } as any)).toBe("5xx");
    expect(statusBadgeClass({ success: false, status: 500 } as any)).toBe("badge--err");
  });
  it("success false + 4xx", () => {
    expect(statusCategory({ success: false, status: 404 } as any)).toBe("4xx");
    expect(statusBadgeClass({ success: false, status: 404 } as any)).toBe("badge--warn");
  });
  it("success false + no status -> other", () => {
    expect(statusCategory({ success: false } as any)).toBe("other");
    expect(statusBadgeClass({ success: false } as any)).toBe("badge--err");
  });
});

describe("successRateClass / successBadgeClass", () => {
  it("thresholds 95/80", () => {
    expect(successRateClass(95)).toBe("good");
    expect(successRateClass(94.9)).toBe("warn");
    expect(successRateClass(80)).toBe("warn");
    expect(successRateClass(79.9)).toBe("bad");
    expect(successBadgeClass(95)).toBe("badge--ok");
    expect(successBadgeClass(80)).toBe("badge--warn");
    expect(successBadgeClass(0)).toBe("badge--err");
  });
});

describe("statSuccessRate / statAvgLatency", () => {
  it("0 requests -> 0", () => {
    expect(statSuccessRate({ total_requests: 0, success_count: 0 } as any)).toBe(0);
    expect(statAvgLatency({ total_requests: 0, total_duration_ms: 0 } as any)).toBe(0);
  });
  it("computes rate and avg latency", () => {
    expect(statSuccessRate({ total_requests: 4, success_count: 3 } as any)).toBe(75);
    expect(statAvgLatency({ total_requests: 4, total_duration_ms: 400 } as any)).toBe(100);
    expect(statAvgLatency({ total_requests: 3, total_duration_ms: 10 } as any)).toBe(3); // rounded
  });
});

describe("providerColor", () => {
  it("deterministic and returns hex color", () => {
    const c1 = providerColor("openai");
    const c2 = providerColor("openai");
    expect(c1).toBe(c2);
    expect(c1).toMatch(/^#/);
  });
});

describe("providerGlyph", () => {
  it("returns id slice when provider not found", () => {
    expect(providerGlyph("openai", [])).toBe("OP");
  });
  it("returns initials for multi-word name", () => {
    const providers = [{ id: "p1", name: "Open AI" } as any];
    expect(providerGlyph("p1", providers)).toBe("OA");
  });
  it("returns 2-char upper for single word name", () => {
    const providers = [{ id: "p1", name: "Anthropic" } as any];
    expect(providerGlyph("p1", providers)).toBe("AN");
  });
  it("returns name itself if <=2 chars", () => {
    const providers = [{ id: "p1", name: "AI" } as any];
    expect(providerGlyph("p1", providers)).toBe("AI");
  });
});
