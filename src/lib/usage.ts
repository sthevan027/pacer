/**
 * Espelha `crate::usage` (src-tauri/src/usage/mod.rs): mesma forma, camelCase.
 * O widget não consome nada daqui — só o painel.
 */

export interface Totals {
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite5m: number;
  cacheWrite1h: number;
  messages: number;
}

export const EMPTY_TOTALS: Totals = {
  input: 0,
  output: 0,
  cacheRead: 0,
  cacheWrite5m: 0,
  cacheWrite1h: 0,
  messages: 0,
};

export interface ModelTotal {
  model: string;
  family: string;
  totals: Totals;
  /** null = modelo fora da tabela de preços. */
  costBrl: number | null;
}

export interface FamilySlice {
  family: string;
  totals: Totals;
}

export interface DaySlice {
  date: string; // "2026-10-10"
  totals: Totals;
  byFamily: FamilySlice[];
}

export interface RequestRow {
  ts: string;
  model: string;
  family: string;
  session: string;
  project: string;
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  costBrl: number | null;
}

export interface GroupRow {
  name: string;
  sub: string;
  totals: Totals;
  firstAt: string;
  lastAt: string;
}

export interface UsageReport {
  rangeDays: number;
  generatedAt: string;
  /** Famílias do período, sem aplicar o filtro — é com isso que os chips são montados. */
  availableFamilies: string[];
  filteredFamilies: string[];
  days: DaySlice[];
  byModel: ModelTotal[];
  totals: Totals;
  totalCostBrl: number;
  unpricedModels: string[];
  usdBrl: number;
  topSessions: GroupRow[];
  topProjects: GroupRow[];
  recent: RequestRow[];
  /** Quantas requisições passaram pelo filtro; `recent` é só a cauda. */
  requestCount: number;
}

export const RANGES = [7, 14, 30] as const;
export type Range = (typeof RANGES)[number];

/** Espelha `RECENT_LIMIT` em src-tauri/src/usage/mod.rs: quantas requisições a tabela recebe. */
export const RECENT_LIMIT = 200;

/** O log separa a escrita de cache em 5 min e 1 h; na tela somem numa coisa só. */
export function cacheWrite(t: Totals): number {
  return t.cacheWrite5m + t.cacheWrite1h;
}

/** Entrada + saída, sem cache — o consumo principal, como na referência do console. */
export function billableTokens(t: Totals): number {
  return t.input + t.output;
}

export function totalTokens(t: Totals): number {
  return t.input + t.output + t.cacheRead + cacheWrite(t);
}

export type Metric = "tokens" | "cacheRead" | "cacheWrite" | "total";

export const METRICS: { id: Metric; label: string }[] = [
  { id: "tokens", label: "Entrada + saída" },
  { id: "cacheRead", label: "Cache lido" },
  { id: "cacheWrite", label: "Cache criado" },
  { id: "total", label: "Total" },
];

export function metricLabel(m: Metric): string {
  return METRICS.find((x) => x.id === m)?.label ?? m;
}

/**
 * O que o gráfico plota. "Cache lido" existe porque na prática ele é ordens de grandeza maior
 * que entrada + saída: sem essa opção o gráfico ficaria achatado e não diria nada.
 */
export function metricValue(t: Totals, m: Metric): number {
  switch (m) {
    case "tokens":
      return billableTokens(t);
    case "cacheRead":
      return t.cacheRead;
    case "cacheWrite":
      return cacheWrite(t);
    case "total":
      return totalTokens(t);
  }
}
