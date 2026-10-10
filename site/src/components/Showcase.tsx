import { pt } from "../content/pt";
import { UsageBar } from "./UsageWidget";

// Espelha ACCENT_PRESETS de src/lib/accent.ts do app.
export const ACCENTS = [
  { hex: "#1f6feb", name: "Azul" },
  { hex: "#8957e5", name: "Roxo" },
  { hex: "#2ea043", name: "Verde" },
  { hex: "#db61a2", name: "Rosa" },
  { hex: "#39c5cf", name: "Ciano" },
] as const;

function Heatmap() {
  const cells = Array.from({ length: 30 }, (_, i) => ((i * 37) % 11) / 10);
  return (
    <div className="heatmap" role="img" aria-label="Exemplo de grade de atividade dos últimos 30 dias">
      {cells.map((v, i) => (
        <div key={i} className="heat-cell" style={{ opacity: 0.12 + v * 0.88 }} />
      ))}
    </div>
  );
}

export function Showcase({ accent, onAccent }: { accent: string; onAccent: (hex: string) => void }) {
  const a = pt.accent;
  return (
    <section className="container showcase">
      <div className="card pad-lg">
        <h3 className="h-md">{pt.activity.title}</h3>
        <p className="muted small mb">{pt.activity.text}</p>
        <Heatmap />
      </div>
      <div className="card pad-lg">
        <h3 className="h-md">{a.title}</h3>
        <p className="muted small mb">{a.text}</p>
        <div className="swatches" role="radiogroup" aria-label={a.title}>
          {ACCENTS.map((s) => (
            <button
              key={s.hex}
              type="button"
              role="radio"
              aria-checked={accent === s.hex}
              aria-label={s.name}
              title={s.name}
              className={`swatch${accent === s.hex ? " swatch-on" : ""}`}
              style={{ background: s.hex }}
              onClick={() => onAccent(s.hex)}
            />
          ))}
        </div>
        <div className="stack mt">
          <UsageBar label={a.normal} pct={35} forecast={60} reset="2h" />
          <UsageBar label={a.warn} pct={82} forecast={95} reset="1d" tone="warn" />
          <UsageBar label={a.crit} pct={96} forecast={100} reset="4h" tone="crit" />
        </div>
      </div>
    </section>
  );
}
