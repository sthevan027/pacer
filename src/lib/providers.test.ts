import { describe, expect, it } from "vitest";
import { PROVIDERS, hoverWindows, pickActive, providerStatus, withProviderEnabled, worstSnapshot } from "./providers";
import type { Config, Snapshot } from "./types";

const claude = PROVIDERS.find((p) => p.id === "claude")!;
const openai = PROVIDERS.find((p) => p.id === "openai")!;
const copilot = PROVIDERS.find((p) => p.id === "copilot")!;

function snap(provider: string, notice: Snapshot["notice"] = null): Snapshot {
  return {
    provider, name: provider, plan: null, windows: [], activity: [], todayTokens: 0,
    fetchedAt: null, nextRefreshAt: null, stale: false, notice,
  };
}

describe("PROVIDERS", () => {
  it("lists claude first, then openai, copilot, openrouter and z.ai", () => {
    expect(PROVIDERS.map((p) => p.id)).toEqual(["claude", "openai", "copilot", "openrouter", "zai"]);
  });
});

describe("chave de API", () => {
  it("só OpenRouter e z.ai pedem chave colada (os mesmos ids que o Rust aceita)", () => {
    expect(PROVIDERS.filter((p) => p.apiKey).map((p) => p.id)).toEqual(["openrouter", "zai"]);
  });

  it("cada campo diz onde conseguir a chave e não promete nada que o Pacer não faz", () => {
    for (const p of PROVIDERS.filter((x) => x.apiKey)) {
      expect(p.apiKey!.placeholder.length).toBeGreaterThan(3);
      expect(p.apiKey!.getUrl).toMatch(/^https:\/\//);
      expect(p.missingText).toMatch(/chave|token/i);
    }
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

describe("copilot texts", () => {
  it("talks about the GitHub login, not about Claude or Codex", () => {
    expect(copilot.noDataNote).toContain("gh auth login");
    expect(copilot.noDataNote).not.toMatch(/Claude|Codex/);
    expect(copilot.tokenExpiredText).toContain("GitHub");
    expect(copilot.offlineText).toContain("GitHub");
    expect(copilot.sourceNote).toBeUndefined();
    expect(providerStatus(copilot, snap("copilot"), true).text).toBe("Conectado via GitHub CLI");
  });
});

describe("openrouter texts", () => {
  it("tells the user to paste the key (env var still works) and never mentions another provider", () => {
    const o = PROVIDERS.find((p) => p.id === "openrouter")!;
    expect(o.noDataNote).toContain("Configurações");
    expect(o.noDataNote).toContain("OPENROUTER_API_KEY");
    expect(o.noDataNote).not.toMatch(/Claude|Codex|GitHub/);
    expect(providerStatus(o, undefined, true).text).toBe("Cole a chave de API");
  });
});

describe("z.ai texts", () => {
  it("tells the user to paste the key or use the Claude Code settings, and to no other provider", () => {
    const z = PROVIDERS.find((p) => p.id === "zai")!;
    expect(z.noDataNote).toContain("Configurações");
    expect(z.noDataNote).toContain("settings.json");
    expect(z.noDataNote).not.toMatch(/Codex|GitHub|OpenRouter/);
    expect(providerStatus(z, snap("zai"), true).text).toBe("Conectado com a chave de API");
  });
});

describe("hoverWindows", () => {
  const w = (id: string) => ({ id, label: id, usedPct: 10, resetsAt: null, pace: null });

  it("keeps the session and weekly bars when the provider has them", () => {
    expect(hoverWindows([w("seven_day_opus"), w("seven_day"), w("five_hour")]).map((x) => x.id)).toEqual(["five_hour", "seven_day"]);
  });

  it("falls back to the first two windows for providers without those ids", () => {
    expect(hoverWindows([w("chat"), w("completions"), w("premium")]).map((x) => x.id)).toEqual(["chat", "completions"]);
    expect(hoverWindows([])).toEqual([]);
  });
});

describe("worstSnapshot", () => {
  const withUse = (provider: string, pct: number): Snapshot => ({
    ...snap(provider),
    windows: [{ id: "five_hour", label: "Sessão (5h)", usedPct: pct, resetsAt: null, pace: null }],
  });

  it("picks the provider with the highest window, like the tray ring", () => {
    expect(worstSnapshot([withUse("claude", 20), withUse("openai", 95)])?.provider).toBe("openai");
    expect(worstSnapshot([withUse("claude", 60), withUse("openai", 30)])?.provider).toBe("claude");
  });

  it("keeps the first on a tie and is undefined without snapshots", () => {
    expect(worstSnapshot([withUse("claude", 0), snap("openai")])?.provider).toBe("claude");
    expect(worstSnapshot([])).toBeUndefined();
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
  const providers: Config["providers"] = { claude: { enabled: true }, openai: { enabled: false }, copilot: { enabled: false }, openrouter: { enabled: false }, zai: { enabled: false } };

  it("changes only the chosen provider and keeps the others", () => {
    expect(withProviderEnabled(providers, "openai", true)).toEqual({ claude: { enabled: true }, openai: { enabled: true }, copilot: { enabled: false }, openrouter: { enabled: false }, zai: { enabled: false } });
    expect(withProviderEnabled(providers, "claude", false)).toEqual({ claude: { enabled: false }, openai: { enabled: false }, copilot: { enabled: false }, openrouter: { enabled: false }, zai: { enabled: false } });
  });

  it("does not mutate the original", () => {
    withProviderEnabled(providers, "openai", true);
    expect(providers.openai.enabled).toBe(false);
  });
});
