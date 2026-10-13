import type { CSSProperties } from "react";
import { UsageBar } from "./UsageBar";
import { useNow, usePacer } from "../lib/hooks";
import { hoverWindows, worstSnapshot } from "../lib/providers";

/** Janelinha que aparece ao passar o mouse na bandeja: Sessão (5h) e Semanal (ou as duas primeiras janelas do provedor), visual.
 * Mostra o provedor mais cheio, o mesmo que pinta o anel do ícone. */
export function TrayHover() {
  const state = usePacer();
  const now = useNow();
  const snap = state ? worstSnapshot(state.snapshots) : undefined;
  const bars = hoverWindows(snap?.windows ?? []);

  if (!bars.length) return null;

  const style = state ? ({ "--accent": state.config.accentColor } as CSSProperties) : undefined;
  return (
    <div className="wg hover" style={style}>
      {state && state.snapshots.length > 1 && <div className="muted small">{snap?.name}</div>}
      {bars.map((w) => (
        <UsageBar key={w.id} w={w} now={now} />
      ))}
    </div>
  );
}
