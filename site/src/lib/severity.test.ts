import { describe, expect, it } from "vitest";
import { usageColor } from "./severity";

describe("usageColor (igual ao app)", () => {
  it("usa a cor de destaque abaixo de 70%", () => {
    expect(usageColor(0)).toBe("var(--accent)");
    expect(usageColor(69.9)).toBe("var(--accent)");
  });

  it("faz degradê laranja→vermelho de 70 a 90%", () => {
    expect(usageColor(70)).toBe("#d29922"); // laranja
    expect(usageColor(80)).toBe("#e57536"); // meio do caminho
    expect(usageColor(82)).not.toBe(usageColor(70));
    expect(usageColor(82)).not.toBe(usageColor(90));
  });

  it("é vermelho a partir de 90%", () => {
    expect(usageColor(90)).toBe("#f85149");
    expect(usageColor(96)).toBe("#f85149");
  });
});
