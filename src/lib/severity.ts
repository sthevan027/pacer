import type { Pace, Severity } from "./types";

/** Mesma regra de src-tauri/src/snapshot.rs. */
export function severity(usedPct: number, pace: Pace | null): Severity {
  if (usedPct > 90 || (pace !== null && pace.projectedPct > 100)) return "critical";
  if (usedPct >= 75) return "warn";
  return "ok";
}
