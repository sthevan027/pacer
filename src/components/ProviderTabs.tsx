import type { Snapshot } from "../lib/types";
import { ProviderMark } from "./ProviderMark";

export function ProviderTabs({ snapshots, active, onSelect }: { snapshots: Snapshot[]; active: string; onSelect: (provider: string) => void }) {
  return (
    <div className="tabs">
      {snapshots.map((s) => {
        const max = Math.max(0, ...s.windows.map((w) => w.usedPct));
        return (
          <button key={s.provider} type="button" title={s.name} className={`tab${s.provider === active ? " on" : ""}`} onClick={() => onSelect(s.provider)}>
            <ProviderMark id={s.provider} size={13} />
            {s.windows.length ? `${Math.round(max)}%` : "—"}
          </button>
        );
      })}
    </div>
  );
}
