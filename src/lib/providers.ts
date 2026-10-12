import type { Config, Snapshot } from "./types";

export type ProviderId = "claude" | "openai";

export interface ProviderDef {
  id: ProviderId;
  name: string;
  /** Texto quando está lendo dados com sucesso. */
  connectedText: string;
  /** Texto quando falta a fonte local (login ou dados). */
  missingText: string;
}

export const PROVIDERS: readonly ProviderDef[] = [
  { id: "claude", name: "Claude", connectedText: "Conectado via Claude Code", missingText: "Sem login do Claude Code" },
  { id: "openai", name: "OpenAI", connectedText: "Lendo os dados locais do Codex", missingText: "Sem dados do Codex neste PC" },
];

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

/** Liga/desliga um provedor sem perder os outros (o `save_config` troca o objeto inteiro). */
export function withProviderEnabled(providers: Config["providers"], id: ProviderId, enabled: boolean): Config["providers"] {
  return { ...providers, [id]: { enabled } };
}
