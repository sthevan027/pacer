import { Icon } from "../icons/icons";
import { formatDuration } from "../lib/format";
import { severity } from "../lib/severity";
import type { UsageWindow } from "../lib/types";

export function UsageBar({ w, now }: { w: UsageWindow; now: number }) {
  const sev = severity(w.usedPct, w.pace);
  const resetIn = w.resetsAt ? formatDuration(Date.parse(w.resetsAt) - now) : null;
  const limitIn = w.pace?.limitAt ? formatDuration(Date.parse(w.pace.limitAt) - now) : null;
  const projected = w.pace?.projectedPct ?? null;

  let bottomRight = "";
  if (projected !== null) {
    bottomRight = projected > 100 ? (resetIn ? `Redefine em ${resetIn}` : "") : `~${Math.round(projected)}% na redefinição`;
  }

  return (
    <div className="m">
      <div className="mtop">
        <span>{w.label}</span>
        {sev === "critical" && limitIn ? (
          <span className="warn">
            <Icon name="trend" size={13} />
            Limite em {limitIn}
          </span>
        ) : (
          resetIn && <span className="muted light">Redefine em {resetIn}</span>
        )}
      </div>
      <div className="bar">
        <div className={`fill sev-${sev}`} style={{ width: `${w.usedPct}%` }} />
        {projected !== null && <div className="tick" style={{ left: `${Math.min(projected, 100)}%` }} />}
      </div>
      <div className="mbot">
        <span>{Math.round(w.usedPct)}% usado</span>
        <span>{bottomRight}</span>
      </div>
    </div>
  );
}
