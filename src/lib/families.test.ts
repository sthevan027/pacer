import { describe, expect, it } from "vitest";
import { familyColor, familyLabel, rampPercent } from "./families";

describe("familyLabel", () => {
  it("traduz as famílias conhecidas", () => {
    expect(familyLabel("opus")).toBe("Opus");
    expect(familyLabel("sonnet")).toBe("Sonnet");
    expect(familyLabel("outro")).toBe("Outros");
  });

  it("deixa passar o que não conhece, em vez de sumir com a linha", () => {
    expect(familyLabel("gpt")).toBe("gpt");
  });
});

describe("rampPercent", () => {
  it("uma família só usa a cor de destaque pura", () => {
    expect(rampPercent(0, 1)).toBe(100);
  });

  it("distribui os tons entre 100% e 45%", () => {
    expect(rampPercent(0, 3)).toBe(100);
    expect(rampPercent(1, 3)).toBe(73);
    expect(rampPercent(2, 3)).toBe(45);
  });

  it("escurece de forma monotônica, sem repetir tom", () => {
    const tons = [0, 1, 2, 3, 4].map((i) => rampPercent(i, 5));
    for (let i = 1; i < tons.length; i++) expect(tons[i]).toBeLessThan(tons[i - 1]);
    expect(new Set(tons).size).toBe(tons.length);
  });

  it("nunca desce abaixo do piso, mesmo com muitas famílias", () => {
    expect(rampPercent(9, 10)).toBeGreaterThanOrEqual(45);
  });
});

describe("familyColor", () => {
  it("sai toda da cor de destaque", () => {
    expect(familyColor(0, 3)).toBe("color-mix(in srgb, var(--accent) 100%, white)");
    expect(familyColor(2, 3)).toContain("var(--accent)");
  });
});
