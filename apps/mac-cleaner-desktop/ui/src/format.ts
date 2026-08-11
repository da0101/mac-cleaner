export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1_000 && unit < units.length - 1) {
    value /= 1_000;
    unit += 1;
  }
  return unit === 0 ? `${bytes} B` : `${value.toFixed(1)} ${units[unit]}`;
}

export function formatPercent(used: number, total: number): number {
  if (total <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((used / total) * 100)));
}

export type MemorySeverity = "normal" | "high" | "very-high";

export function availabilitySeverity(availableBytes: number, totalBytes: number): MemorySeverity {
  if (totalBytes <= 0) return "normal";
  const availableShare = availableBytes / totalBytes;
  if (availableShare < 0.10) return "very-high";
  if (availableShare < 0.20) return "high";
  return "normal";
}

export function processMemorySeverity(bytes: number, totalBytes: number): MemorySeverity {
  if (totalBytes <= 0) return "normal";
  const share = bytes / totalBytes;
  if (share >= 0.10) return "very-high";
  if (share >= 0.05) return "high";
  return "normal";
}

export function formatDate(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
