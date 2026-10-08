import type { CSSProperties } from "react";
import { UsageBar } from "./UsageBar";
import { useNow, usePacer } from "../lib/hooks";

/** Janelinha que aparece ao passar o mouse na bandeja: Sessão (5h) e Semanal, visual. */
export function TrayHover() {
  const state = usePacer();
  const now = useNow();
  const windows = state?.snapshots[0]?.windows ?? [];
  const session = windows.find((w) => w.id === "five_hour");
  const week = windows.find((w) => w.id === "seven_day");

  if (!session && !week) return null;

  const style = state ? ({ "--accent": state.config.accentColor } as CSSProperties) : undefined;
  return (
    <div className="wg hover" style={style}>
      {session && <UsageBar w={session} now={now} />}
      {week && <UsageBar w={week} now={now} />}
    </div>
  );
}
