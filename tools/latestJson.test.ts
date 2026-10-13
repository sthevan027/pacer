import { describe, expect, it } from "vitest";
import { buildLatestJson, installerName, releaseTag } from "./latestJson";

const base = { notes: "notas", pubDate: new Date("2026-10-12T12:00:00Z"), signature: " sig \n", repo: "sthevan027/pacer" };

describe("buildLatestJson", () => {
  it("aponta o instalador para a tag v<versão> com o nome que o Tauri gera", () => {
    const json = buildLatestJson({ ...base, version: "v2.2.0" });
    expect(json.version).toBe("2.2.0");
    expect(json.platforms["windows-x86_64"].url).toBe(
      "https://github.com/sthevan027/pacer/releases/download/v2.2.0/Pacer_2.2.0_x64-setup.exe",
    );
    expect(json.platforms["windows-x86_64"].signature).toBe("sig");
    expect(json.pub_date).toBe("2026-10-12T12:00:00.000Z");
  });

  it("recusa versão fora de x.y.z e assinatura vazia", () => {
    expect(() => buildLatestJson({ ...base, version: "2.2", signature: "x" })).toThrow();
    expect(() => buildLatestJson({ ...base, version: "2.2.0", signature: "  " })).toThrow();
  });

  it("nomes auxiliares seguem o mesmo padrão", () => {
    expect(installerName("2.2.0-beta.1")).toBe("Pacer_2.2.0-beta.1_x64-setup.exe");
    expect(releaseTag("2.2.0")).toBe("v2.2.0");
  });
});
