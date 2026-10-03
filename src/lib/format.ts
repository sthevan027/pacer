const MONTHS = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
const oneDecimal = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 1 });

export function formatDuration(ms: number): string {
  const mins = Math.max(0, Math.floor(ms / 60_000));
  const d = Math.floor(mins / 1440);
  const h = Math.floor((mins % 1440) / 60);
  const m = mins % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return "menos de 1m";
}

export function formatTokens(n: number): string {
  if (n >= 1e9) return `${oneDecimal.format(n / 1e9)}B`;
  if (n >= 1e6) return `${oneDecimal.format(n / 1e6)}M`;
  if (n >= 1e3) return `${Math.round(n / 1e3)}k`;
  return String(n);
}

export function formatAgo(iso: string, now: number): string {
  const mins = Math.floor((now - Date.parse(iso)) / 60_000);
  if (mins < 1) return "agora";
  if (mins < 60) return `há ${mins} min`;
  if (mins < 1440) return `há ${Math.floor(mins / 60)} h`;
  return `há ${Math.floor(mins / 1440)} d`;
}

/** "2026-09-12" → "12 set" (parse manual, sem fuso). */
export function formatDay(isoDate: string): string {
  const [, m, d] = isoDate.split("-").map(Number);
  return `${d} ${MONTHS[m - 1]}`;
}
