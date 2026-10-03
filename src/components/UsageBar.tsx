import { Icon } from "../icons/icons";
import { formatDuration } from "../lib/format";
import { severity, usageColor } from "../lib/severity";
import type { UsageWindow } from "../lib/types";

export function UsageBar({ w, now }: { w: UsageWindow; now: number }) {
  const sev = severity(w.usedPct);
  const resetIn = w.resetsAt ? formatDuration(Date.parse(w.resetsAt) - now) : null;
  const projected = w.pace?.projectedPct ?? null;
  // Previsão: só avisa quando o ritmo atual estoura antes da redefinição. A cor da barra
  // continua sendo do uso; o aviso fica laranja (vermelho só se o uso já passou de 90%).
  const runsOut = projected !== null && projected > 100 && w.pace?.limitAt ? Date.parse(w.pace.limitAt) - now : null;

  let topRight = null;
  if (w.usedPct >= 100) {
    topRight = <span className="warn crit">Limite atingido</span>;
  } else if (runsOut !== null) {
    topRight = (
      <span className={`warn${sev === "critical" ? " crit" : ""}`}>
        <Icon name="trend" size={13} />
        Nesse ritmo, acaba em {formatDuration(runsOut)}
      </span>
    );
  } else if (resetIn) {
    topRight = <span className="muted light">Redefine em {resetIn}</span>;
  }

  let bottomRight = "";
  if (projected !== null) {
    bottomRight = projected > 100 ? (resetIn ? `Redefine em ${resetIn}` : "") : `~${Math.round(projected)}% na redefinição`;
  }

  return (
    <div className="m">
      <div className="mtop">
        <span>{w.label}</span>
        {topRight}
      </div>
      <div className="bar">
        <div className="fill" style={{ width: `${w.usedPct}%`, backgroundColor: usageColor(w.usedPct) }} />
        {projected !== null && <div className="tick" style={{ left: `${Math.min(projected, 100)}%` }} />}
      </div>
      <div className="mbot">
        <span>{Math.round(w.usedPct)}% usado</span>
        <span>{bottomRight}</span>
      </div>
    </div>
  );
}
