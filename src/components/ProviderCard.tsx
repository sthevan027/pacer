import { ClaudeLogo, Icon } from "../icons/icons";
import { formatAgo } from "../lib/format";
import type { Notice, Snapshot } from "../lib/types";
import { UsageBar } from "./UsageBar";

const NOTICE_TEXT: Record<Exclude<Notice, "noCredentials">, string> = {
  tokenExpired: "O login do Claude Code expirou. Abra o Claude Code para renovar.",
  rateLimited: "Não consegui atualizar (limite de pedidos).",
  offline: "Sem conexão com a Anthropic.",
};

export function ProviderCard({ snap, now }: { snap: Snapshot; now: number }) {
  return (
    <section className={`sec${snap.stale ? " stale" : ""}`}>
      <div className="ctitle">
        <span className="l">
          <ClaudeLogo />
          {snap.name}
          {snap.plan && <span className="badge">{snap.plan}</span>}
        </span>
      </div>
      {snap.notice === "noCredentials" ? (
        <p className="muted note">
          Não achei o login do Claude Code neste PC. Rode <code>claude</code> no terminal e faça login — o Pacer pega
          sozinho. Enquanto isso, mostro só a atividade dos logs.
        </p>
      ) : (
        snap.windows.map((w) => <UsageBar key={w.id} w={w} now={now} />)
      )}
      {snap.notice && snap.notice !== "noCredentials" && (
        <div className="warnline">
          <Icon name="alert" size={13} />
          <span>
            {NOTICE_TEXT[snap.notice]}
            {snap.stale && snap.fetchedAt ? ` Último dado: ${formatAgo(snap.fetchedAt, now)}.` : ""}
          </span>
        </div>
      )}
    </section>
  );
}
