import { formatAgo, formatDuration } from "../lib/format";
import { providerDef } from "../lib/providers";
import type { Snapshot } from "../lib/types";

export function Footer({ snap, now }: { snap: Snapshot | undefined; now: number }) {
  if (!snap) return null;
  // Provedor que só lê arquivo local não "atualizou": o horário é o do último dado que ele deixou lá.
  const verb = providerDef(snap.provider)?.sourceNote ? "Último dado" : "Atualizado";
  const left = snap.fetchedAt ? `${verb} ${formatAgo(snap.fetchedAt, now)}` : "Só dados locais";
  const next = snap.nextRefreshAt ? formatDuration(Date.parse(snap.nextRefreshAt) - now) : null;
  const right = next ? (snap.stale ? `Tento de novo em ${next}` : `Próxima em ${next}`) : "";
  return (
    <div className="foot">
      <span>{left}</span>
      <span>{right}</span>
    </div>
  );
}
