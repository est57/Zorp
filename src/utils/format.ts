export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value < 10 ? 2 : 1)} ${units[unit]}`;
}

export function formatDuration(ms: number): string {
  return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(2)} s`;
}

export function statusTone(status: number | null | undefined): string {
  if (status == null) return "text-danger";
  if (status >= 500) return "text-danger";
  if (status >= 400) return "text-warning";
  if (status >= 200 && status < 300) return "text-success";
  return "text-[var(--info)]";
}

export function dayLabel(timestamp: number, now = Date.now()): string {
  const day = (t: number) => new Date(t).toDateString();
  if (day(timestamp) === day(now)) return "Hari ini";
  if (day(timestamp) === day(now - 86_400_000)) return "Kemarin";
  return new Date(timestamp).toLocaleDateString("id-ID", {
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

export const timeLabel = (timestamp: number) =>
  new Date(timestamp).toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" });
