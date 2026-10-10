import { describe, expect, it } from "vitest";
import { FALLBACK_INSTALLER_URL, fetchReleases, formatSize, installerUrl, parseReleases, pickInstaller } from "./release";
import { isWindows } from "./platform";

const asset = (name: string) => ({ name, size: 2_411_724, browser_download_url: `https://x/${name}` });
const rel = (tag: string, extra = {}) => ({
  tag_name: tag,
  name: tag,
  html_url: `https://x/${tag}`,
  published_at: "2026-10-09T12:00:00Z",
  draft: false,
  prerelease: false,
  assets: [asset(`Pacer_${tag.slice(1)}_x64-setup.exe`)],
  ...extra,
});

describe("release", () => {
  it("acha o instalador pelo padrão de nome, ignorando outros assets", () => {
    expect(pickInstaller([asset("Sthevan.Dev-Root-CA.cer"), asset("Pacer_2.1.0_x64-setup.exe")])?.name).toBe(
      "Pacer_2.1.0_x64-setup.exe",
    );
    expect(pickInstaller([asset("Sthevan.Dev-Root-CA.cer")])).toBeUndefined();
  });

  it("ignora rascunhos e pré-releases e não quebra sem asset", () => {
    const list = parseReleases([rel("v2.2.0", { draft: true }), rel("v2.1.1", { prerelease: true }), rel("v2.1.0", { assets: [] })]);
    expect(list.map((r) => r.version)).toEqual(["v2.1.0"]);
    expect(list[0].installer).toBeUndefined();
  });

  it("usa a entrada não-array como lista vazia", () => {
    expect(parseReleases({ message: "rate limited" })).toEqual([]);
  });

  it("propaga erro da API e usa o cache de sessão", async () => {
    await expect(fetchReleases((async () => new Response("", { status: 403 })) as typeof fetch)).rejects.toThrow("403");
    const store = new Map<string, string>();
    const storage = { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => void store.set(k, v) };
    let calls = 0;
    const ok = (async () => (calls++, new Response(JSON.stringify([rel("v2.1.0")])))) as typeof fetch;
    expect((await fetchReleases(ok, storage))[0].version).toBe("v2.1.0");
    await fetchReleases(ok, storage);
    expect(calls).toBe(1);
  });

  it("baixa direto o instalador da última release, ou o fallback — nunca a página da release", () => {
    const [latest] = parseReleases([rel("v2.2.0")]);
    expect(installerUrl(latest)).toBe("https://x/Pacer_2.2.0_x64-setup.exe");
    expect(installerUrl(undefined)).toBe(FALLBACK_INSTALLER_URL);
    expect(installerUrl(parseReleases([rel("v2.3.0", { assets: [] })])[0])).toBe(FALLBACK_INSTALLER_URL);
    expect(FALLBACK_INSTALLER_URL).toMatch(/\/releases\/download\/.+_x64-setup\.exe$/);
  });

  it("formata tamanho em MB pt-BR", () => {
    expect(formatSize(2_411_724)).toBe("2,3 MB");
  });
});

describe("platform", () => {
  it("detecta Windows", () => {
    expect(isWindows({ userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)" })).toBe(true);
    expect(isWindows({ userAgent: "Mozilla/5.0 (Macintosh)", userAgentData: { platform: "macOS" } })).toBe(false);
    expect(isWindows({ userAgent: "Mozilla/5.0 (Linux; Android 14)" })).toBe(false);
  });
});
