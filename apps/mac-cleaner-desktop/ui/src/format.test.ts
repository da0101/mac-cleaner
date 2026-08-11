import { describe, expect, it } from "vitest";
import { availabilitySeverity, formatBytes, formatPercent, processMemorySeverity } from "./format";

describe("formatBytes", () => {
  it("keeps byte values exact and scales larger values", () => {
    expect(formatBytes(999)).toBe("999 B");
    expect(formatBytes(1_500)).toBe("1.5 KB");
  });
});

describe("formatPercent", () => {
  it("bounds values for accessible progress bars", () => {
    expect(formatPercent(25, 100)).toBe(25);
    expect(formatPercent(2, 0)).toBe(0);
    expect(formatPercent(200, 100)).toBe(100);
  });
});

describe("processMemorySeverity", () => {
  it("warns relative to installed memory instead of alarming on ordinary usage", () => {
    const total = 20_000_000_000;
    expect(processMemorySeverity(999_000_000, total)).toBe("normal");
    expect(processMemorySeverity(1_000_000_000, total)).toBe("high");
    expect(processMemorySeverity(2_000_000_000, total)).toBe("very-high");
  });
});

describe("availabilitySeverity", () => {
  it("uses the same healthy, low, and critical bands for RAM and disk", () => {
    const total = 100_000_000_000;
    expect(availabilitySeverity(20_000_000_000, total)).toBe("normal");
    expect(availabilitySeverity(19_999_999_999, total)).toBe("high");
    expect(availabilitySeverity(10_000_000_000, total)).toBe("high");
    expect(availabilitySeverity(9_999_999_999, total)).toBe("very-high");
    expect(availabilitySeverity(0, 0)).toBe("normal");
  });
});
