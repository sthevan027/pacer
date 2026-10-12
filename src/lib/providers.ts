import type { Config, Notice, Snapshot } from "./types";

export type ProviderId = "claude" | "openai";

export interface ProviderDef {
  id: ProviderId;
  name: string;
  /** Texto quando está lendo dados com sucesso. */
  connectedText: string;
  /** Texto curto quando falta a fonte local (login ou dados). */
  missingText: string;
  /** Parágrafo do card quando não há nenhum dado. */
  noDataNote: string;
  tokenExpiredText: string;
  offlineText: string;
  /** Aviso fixo do card para provedores que só leem dados locais. */
  sourceNote?: string;
}

export const PROVIDERS: readonly ProviderDef[] = [
  {
    id: "claude",
    name: "Claude",
    connectedText: "Conectado via Claude Code",
    missingText: "Sem login do Claude Code",
    noDataNote:
      "Não achei o login do Claude Code neste PC. Rode `claude` no terminal e faça login — o Pacer pega sozinho. Enquanto isso, mostro só a atividade dos logs.",
    tokenExpiredText: "O login do Claude Code expirou. Abra o Claude Code para renovar.",
    offlineText: "Sem conexão com a Anthropic.",
  },
  {
    id: "openai",
    name: "OpenAI",
    connectedText: "Lendo os dados locais do Codex",
    missingText: "Sem dados do Codex neste PC",
    noDataNote:
      "Não achei dados do Codex neste PC. Use o Codex neste computador e o Pacer lê sozinho os limites da última sessão.",
    tokenExpiredText: "Não consegui ler os dados do Codex.",
    offlineText: "Não consegui ler os dados do Codex.",
    sourceNote: "Mostra só o uso do Codex feito neste PC, na última sessão.",
  },
];

export function providerDef(id: string): ProviderDef | undefined {
  return PROVIDERS.find((p) => p.id === id);
}

export interface ProviderStatus {
  connected: boolean;
  text: string;
}

/** Estado de conexão mostrado nas Configurações. Desligado nunca finge estar conectado. */
export function providerStatus(def: ProviderDef, snap: Snapshot | undefined, enabled: boolean): ProviderStatus {
  if (!enabled) return { connected: false, text: "Desligado" };
  const missing = !snap || snap.notice === "noCredentials" || snap.notice === "tokenExpired";
  return missing ? { connected: false, text: def.missingText } : { connected: true, text: def.connectedText };
}

/** Texto do aviso de um card, sem citar a empresa de outro provedor. */
export function noticeText(def: ProviderDef | undefined, notice: Exclude<Notice, "noCredentials">): string {
  if (notice === "rateLimited") return "Não consegui atualizar (limite de pedidos).";
  if (notice === "tokenExpired") return def?.tokenExpiredText ?? "O login expirou.";
  return def?.offlineText ?? "Sem conexão.";
}

/** Snapshot da aba escolhida; sem escolha (ou se o provedor sumiu) vale o primeiro. */
export function pickActive(snapshots: Snapshot[], chosen: string | null): Snapshot | undefined {
  return snapshots.find((s) => s.provider === chosen) ?? snapshots[0];
}

/** Provedor com a janela mais cheia: a mesma regra do anel da bandeja. Empate fica com o primeiro. */
export function worstSnapshot(snapshots: Snapshot[]): Snapshot | undefined {
  const peak = (s: Snapshot) => Math.max(0, ...s.windows.map((w) => w.usedPct));
  return snapshots.reduce<Snapshot | undefined>((best, s) => (!best || peak(s) > peak(best) ? s : best), undefined);
}

/** Liga/desliga um provedor sem perder os outros (o `save_config` troca o objeto inteiro). */
export function withProviderEnabled(providers: Config["providers"], id: ProviderId, enabled: boolean): Config["providers"] {
  return { ...providers, [id]: { enabled } };
}
