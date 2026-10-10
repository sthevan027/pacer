export type Severity = "ok" | "warn" | "critical";
export type Notice = "noCredentials" | "tokenExpired" | "rateLimited" | "offline";

export interface Pace {
  projectedPct: number;
  limitAt: string | null;
}

export interface UsageWindow {
  id: string;
  label: string;
  usedPct: number;
  resetsAt: string | null;
  pace: Pace | null;
}

export interface DayActivity {
  date: string; // "2026-10-03"
  tokens: number;
  messages: number;
}

export interface Snapshot {
  provider: string;
  name: string;
  plan: string | null;
  windows: UsageWindow[];
  activity: DayActivity[];
  todayTokens: number;
  fetchedAt: string | null;
  nextRefreshAt: string | null;
  stale: boolean;
  notice: Notice | null;
}

export interface AlertsConfig {
  enabled: boolean;
  thresholds: number[];
  pace: boolean;
}

export interface Config {
  startWithWindows: boolean;
  refreshMinutes: number;
  accentColor: string;
  /** Cotação US$→R$ para estimar o consumo a preço de API. */
  usdBrl: number;
  alerts: AlertsConfig;
  providers: { claude: { enabled: boolean } };
}

export interface AppState {
  snapshots: Snapshot[];
  config: Config;
  version: string;
}
