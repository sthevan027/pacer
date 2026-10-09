import type { Severity } from "./types";

const ORANGE = [0xd2, 0x99, 0x22];
const RED = [0xf8, 0x51, 0x49];

/** Só o uso decide (a previsão não): mesma regra de src-tauri/src/snapshot.rs. */
export function severity(usedPct: number): Severity {
  if (usedPct >= 90) return "critical";
  if (usedPct >= 70) return "warn";
  return "ok";
}

const hex = (rgb: number[]) => `#${rgb.map((c) => c.toString(16).padStart(2, "0")).join("")}`;

/**
 * Cor de destaque (configurável, ver `--accent` em styles.css) < 70%, laranja→vermelho de 70 a
 * 90%, vermelho ≥ 90% (aviso/crítico nunca mudam — igual ao ícone da bandeja, src-tauri/src/tray.rs).
 */
export function usageColor(usedPct: number): string {
  if (usedPct < 70) return "var(--accent)";
  if (usedPct >= 90) return hex(RED);
  const t = (usedPct - 70) / 20;
  return hex(ORANGE.map((a, i) => Math.round(a + (RED[i] - a) * t)));
}
