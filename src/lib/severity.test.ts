import { describe, expect, it } from "vitest";
import { severity, usageColor } from "./severity";

describe("severity", () => {
  it("segue só o uso, com os mesmos limiares do Rust", () => {
    expect(severity(0)).toBe("ok");
    expect(severity(69.9)).toBe("ok");
    expect(severity(70)).toBe("warn");
    expect(severity(89.9)).toBe("warn");
    expect(severity(90)).toBe("critical");
    expect(severity(100)).toBe("critical");
  });
});

describe("usageColor", () => {
  it("cor de destaque até 70%, laranja→vermelho de 70 a 90%, vermelho a partir de 90%", () => {
    expect(usageColor(0)).toBe("var(--accent)");
    expect(usageColor(69.9)).toBe("var(--accent)");
    expect(usageColor(70)).toBe("#d29922");
    expect(usageColor(80)).toBe("#e57536");
    expect(usageColor(90)).toBe("#f85149");
    expect(usageColor(100)).toBe("#f85149");
  });
});
