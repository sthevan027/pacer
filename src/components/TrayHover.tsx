import type { CSSProperties } from "react";
import { UsageBar } from "./UsageBar";
import { useNow, usePacer } from "../lib/hooks";
import { worstSnapshot } from "../lib/providers";

/** Janelinha que aparece ao passar o mouse na bandeja: Sessão (5h) e Semanal, visual.
 * Mostra o provedor mais cheio, o mesmo que pinta o anel do ícone. */
export function TrayHover() {
  const state = usePacer();
  const now = useNow();
  const snap = state ? worstSnapshot(state.snapshots) : undefined;
  const windows = snap?.windows ?? [];
  const session = windows.find((w) => w.id === "five_hour");
  const week = windows.find((w) => w.id === "seven_day");

  if (!session && !week) return null;

  const style = state ? ({ "--accent": state.config.accentColor } as CSSProperties) : undefined;
  return (
    <div className="wg hover" style={style}>
      {state && state.snapshots.length > 1 && <div className="muted small">{snap?.name}</div>}
      {session && <UsageBar w={session} now={now} />}
      {week && <UsageBar w={week} now={now} />}
    </div>
  );
}
