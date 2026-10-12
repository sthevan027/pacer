import { familyLabel } from "../../lib/families";
import { providerDef } from "../../lib/providers";
import { METRICS, RANGES, type Metric, type Range } from "../../lib/usage";

interface Props {
  range: Range;
  onRange: (r: Range) => void;
  metric: Metric;
  onMetric: (m: Metric) => void;
  available: string[];
  selected: string[];
  onToggleFamily: (f: string) => void;
  availableProviders: string[];
  selectedProviders: string[];
  onToggleProvider: (p: string) => void;
  colorOfFamily: (f: string) => string;
  busy: boolean;
}

export function Filters({
  range, onRange, metric, onMetric, available, selected, onToggleFamily,
  availableProviders, selectedProviders, onToggleProvider, colorOfFamily, busy,
}: Props) {
  return (
    <div className={`pnfilters${busy ? " busy" : ""}`}>
      <div className="pnseg">
        {RANGES.map((r) => (
          <button key={r} className={r === range ? "on" : undefined} onClick={() => onRange(r)}>
            {r} dias
          </button>
        ))}
      </div>

      {availableProviders.length > 1 && (
        <div className="pntiers">
          {availableProviders.map((p) => {
            const on = selectedProviders.length === 0 || selectedProviders.includes(p);
            const name = providerDef(p)?.name ?? p;
            return (
              <button key={p} className={`pnchip${on ? " on" : ""}`} onClick={() => onToggleProvider(p)} title={on ? `Esconder ${name}` : `Mostrar ${name}`}>
                {name}
              </button>
            );
          })}
        </div>
      )}

      <div className="pntiers">
        {available.length === 0 ? (
          <span className="muted small">sem consumo no período</span>
        ) : (
          available.map((f) => {
            // clicar num tier tira ele da pilha; sem nenhum selecionado, todos aparecem
            const on = selected.length === 0 || selected.includes(f);
            return (
              <button
                key={f}
                className={`pnchip${on ? " on" : ""}`}
                onClick={() => onToggleFamily(f)}
                title={on ? `Esconder ${familyLabel(f)}` : `Mostrar ${familyLabel(f)}`}
              >
                <i style={{ background: colorOfFamily(f) }} />
                {familyLabel(f)}
              </button>
            );
          })
        )}
      </div>

      <div className="pnseg right">
        {METRICS.map((m) => (
          <button key={m.id} className={m.id === metric ? "on" : undefined} onClick={() => onMetric(m.id)}>
            {m.label}
          </button>
        ))}
      </div>
    </div>
  );
}
