import { describe, expect, it } from "vitest";
import { PROVIDERS, pickActive, providerStatus, withProviderEnabled } from "./providers";
import type { Config, Snapshot } from "./types";

const claude = PROVIDERS.find((p) => p.id === "claude")!;
const openai = PROVIDERS.find((p) => p.id === "openai")!;

function snap(provider: string, notice: Snapshot["notice"] = null): Snapshot {
  return {
    provider, name: provider, plan: null, windows: [], activity: [], todayTokens: 0,
    fetchedAt: null, nextRefreshAt: null, stale: false, notice,
  };
}

describe("PROVIDERS", () => {
  it("lists claude first and openai second", () => {
    expect(PROVIDERS.map((p) => p.id)).toEqual(["claude", "openai"]);
  });
});

describe("providerStatus", () => {
  it("is off when the toggle is off, whatever the snapshot says", () => {
    expect(providerStatus(claude, snap("claude"), false)).toEqual({ connected: false, text: "Desligado" });
  });

  it("is connected with the provider's own wording", () => {
    expect(providerStatus(claude, snap("claude"), true)).toEqual({ connected: true, text: "Conectado via Claude Code" });
    expect(providerStatus(openai, snap("openai"), true)).toEqual({ connected: true, text: "Lendo os dados locais do Codex" });
  });

  it("says what is missing when there is no snapshot yet or no credentials", () => {
    expect(providerStatus(openai, undefined, true)).toEqual({ connected: false, text: "Sem dados do Codex neste PC" });
    expect(providerStatus(openai, snap("openai", "noCredentials"), true).connected).toBe(false);
    expect(providerStatus(claude, snap("claude", "noCredentials"), true).text).toBe("Sem login do Claude Code");
  });

  it("treats an expired token as not connected", () => {
    expect(providerStatus(claude, snap("claude", "tokenExpired"), true).connected).toBe(false);
  });

  it("keeps connected while only rate limited or offline (data is still there)", () => {
    expect(providerStatus(claude, snap("claude", "rateLimited"), true).connected).toBe(true);
    expect(providerStatus(claude, snap("claude", "offline"), true).connected).toBe(true);
  });
});

describe("pickActive", () => {
  const two = [snap("claude"), snap("openai")];

  it("picks the requested provider", () => {
    expect(pickActive(two, "openai")?.provider).toBe("openai");
  });

  it("falls back to the first when nothing was chosen or the choice is gone", () => {
    expect(pickActive(two, null)?.provider).toBe("claude");
    expect(pickActive([snap("openai")], "claude")?.provider).toBe("openai");
  });

  it("is undefined when no provider is active", () => {
    expect(pickActive([], "claude")).toBeUndefined();
  });
});

describe("notice texts", () => {
  it("each provider explains its own missing source", () => {
    expect(claude.noDataNote).toContain("Claude Code");
    expect(openai.noDataNote).toContain("Codex");
    expect(openai.noDataNote).not.toContain("Claude");
  });

  it("only the local-only provider carries the source note", () => {
    expect(claude.sourceNote).toBeUndefined();
    expect(openai.sourceNote).toMatch(/neste PC/);
  });

  it("offline and expired texts never name another provider's company", () => {
    expect(openai.offlineText).not.toMatch(/Anthropic|Claude/);
    expect(claude.offlineText).toContain("Anthropic");
  });
});

describe("withProviderEnabled", () => {
  const providers: Config["providers"] = { claude: { enabled: true }, openai: { enabled: false } };

  it("changes only the chosen provider and keeps the others", () => {
    expect(withProviderEnabled(providers, "openai", true)).toEqual({ claude: { enabled: true }, openai: { enabled: true } });
    expect(withProviderEnabled(providers, "claude", false)).toEqual({ claude: { enabled: false }, openai: { enabled: false } });
  });

  it("does not mutate the original", () => {
    withProviderEnabled(providers, "openai", true);
    expect(providers.openai.enabled).toBe(false);
  });
});
