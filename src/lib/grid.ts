import type { DayActivity } from "./types";

export type Level = 0 | 1 | 2 | 3 | 4;
export interface Cell extends DayActivity {
  level: Level;
}

export function levelFor(tokens: number, max: number): Level {
  if (tokens <= 0 || max <= 0) return 0;
  return Math.min(4, Math.max(1, Math.ceil((tokens / max) * 4))) as Level;
}

/** Segunda = 0 … domingo = 6, calculado em UTC para não depender do fuso. */
function weekday(isoDate: string): number {
  const [y, m, d] = isoDate.split("-").map(Number);
  return (new Date(Date.UTC(y, m - 1, d)).getUTCDay() + 6) % 7;
}

/** Colunas = semanas; cada coluna tem 7 posições (seg→dom); fora dos 30 dias = null. */
export function buildGrid(days: DayActivity[]): (Cell | null)[][] {
  if (days.length === 0) return [];
  const max = Math.max(...days.map((d) => d.tokens));
  const slots: (Cell | null)[] = Array(weekday(days[0].date)).fill(null);
  for (const d of days) slots.push({ ...d, level: levelFor(d.tokens, max) });
  while (slots.length % 7 !== 0) slots.push(null);
  const weeks: (Cell | null)[][] = [];
  for (let i = 0; i < slots.length; i += 7) weeks.push(slots.slice(i, i + 7));
  return weeks;
}
