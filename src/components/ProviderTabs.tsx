import { ClaudeLogo } from "../icons/icons";
import type { Snapshot } from "../lib/types";

export function ProviderTabs({ snapshots, active }: { snapshots: Snapshot[]; active: string }) {
  return (
    <div className="tabs">
      {snapshots.map((s) => {
        const max = Math.max(0, ...s.windows.map((w) => w.usedPct));
        return (
          <span key={s.provider} className={`tab${s.provider === active ? " on" : ""}`}>
            <ClaudeLogo size={13} />
            {s.windows.length ? `${Math.round(max)}%` : "—"}
          </span>
        );
      })}
    </div>
  );
}
