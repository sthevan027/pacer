import type { ReactNode } from "react";
import { Icon } from "../icons/icons";
import { formatAgo } from "../lib/format";
import { noticeText, providerDef } from "../lib/providers";
import type { Snapshot } from "../lib/types";
import { ProviderMark } from "./ProviderMark";
import { UsageBar } from "./UsageBar";

/** Trecho entre crases vira <code>. */
function withCode(text: string): ReactNode[] {
  return text.split("`").map((part, i) => (i % 2 ? <code key={i}>{part}</code> : part));
}

export function ProviderCard({ snap, now }: { snap: Snapshot; now: number }) {
  const def = providerDef(snap.provider);
  return (
    <section className={`sec${snap.stale ? " stale" : ""}`}>
      <div className="ctitle">
        <span className="l">
          <ProviderMark id={snap.provider} />
          {snap.name}
          {snap.plan && <span className="badge">{snap.plan}</span>}
        </span>
      </div>
      {snap.notice === "noCredentials" ? (
        <p className="muted note">{withCode(def?.noDataNote ?? "Sem dados para este provedor.")}</p>
      ) : (
        snap.windows.map((w) => <UsageBar key={w.id} w={w} now={now} />)
      )}
      {def?.sourceNote && snap.notice !== "noCredentials" && (
        <p className="muted note">
          {def.sourceNote}
          {snap.fetchedAt ? ` Último dado: ${formatAgo(snap.fetchedAt, now)}.` : ""}
        </p>
      )}
      {snap.notice && snap.notice !== "noCredentials" && (
        <div className="warnline">
          <Icon name="alert" size={13} />
          <span>
            {noticeText(def, snap.notice)}
            {snap.stale && snap.fetchedAt ? ` Último dado: ${formatAgo(snap.fetchedAt, now)}.` : ""}
          </span>
        </div>
      )}
    </section>
  );
}
