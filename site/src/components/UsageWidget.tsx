import { pt } from "../content/pt";

type Tone = "ok" | "warn" | "crit";

export function UsageBar({ label, pct, forecast, reset, tone = "ok" }: { label: string; pct: number; forecast: number; reset: string; tone?: Tone }) {
  return (
    <div className="bar">
      <div className="bar-head">
        <span className="muted">{label}</span>
        <span className="mono">{pct}% <span className="muted">· {pt.widget.resets} {reset}</span></span>
      </div>
      <div className="bar-track" role="img" aria-label={`${label}: ${pct}%, previsão ${forecast}%`}>
        <div className={`bar-fill tone-${tone}`} style={{ width: `${pct}%` }} />
        <div className="bar-mark" style={{ left: `${Math.min(forecast, 99)}%` }} />
      </div>
    </div>
  );
}

export function UsageWidget() {
  const w = pt.widget;
  return (
    <div className="widget">
      <div className="widget-head">
        <span className="widget-title"><span className="dot" /> Pacer</span>
        <span className="mono tiny muted">v2.1.0</span>
      </div>
      <div className="stack">
        <UsageBar label={w.session} pct={42} forecast={71} reset="em 2h 14m" />
        <UsageBar label={w.weekly} pct={81} forecast={100} reset="em 2d 9h" tone="warn" />
      </div>
      <div className="callout">{w.warning}</div>
    </div>
  );
}
