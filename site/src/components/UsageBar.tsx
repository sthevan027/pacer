import { pt } from "../content/pt";
import { usageColor } from "../lib/severity";

/** Barra de uso como no app: cor pelo uso (destaque → laranja→vermelho → vermelho) e marca da previsão. */
export function UsageBar({ label, pct, forecast, reset }: { label: string; pct: number; forecast: number; reset: string }) {
  return (
    <div className="bar">
      <div className="bar-head">
        <span className="muted">{label}</span>
        <span className="mono">{pct}% <span className="muted">· {pt.widget.resets} {reset}</span></span>
      </div>
      <div className="bar-track" role="img" aria-label={`${label}: ${pct}%, previsão ${forecast}%`}>
        <div className="bar-fill" style={{ width: `${pct}%`, backgroundColor: usageColor(pct) }} />
        <div className="bar-mark" style={{ left: `${Math.min(forecast, 99)}%` }} />
      </div>
    </div>
  );
}
