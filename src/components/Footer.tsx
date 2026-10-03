import { formatAgo, formatDuration } from "../lib/format";
import type { Snapshot } from "../lib/types";

export function Footer({ snap, now }: { snap: Snapshot | undefined; now: number }) {
  if (!snap) return null;
  const left = snap.fetchedAt ? `Atualizado ${formatAgo(snap.fetchedAt, now)}` : "Só dados locais";
  const next = snap.nextRefreshAt ? formatDuration(Date.parse(snap.nextRefreshAt) - now) : null;
  const right = next ? (snap.stale ? `Tento de novo em ${next}` : `Próxima em ${next}`) : "";
  return (
    <div className="foot">
      <span>{left}</span>
      <span>{right}</span>
    </div>
  );
}
