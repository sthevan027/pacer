import { describe, expect, it } from "vitest";
import { severity } from "./severity";

describe("severity", () => {
  it("segue os mesmos limiares do Rust", () => {
    expect(severity(74.9, null)).toBe("ok");
    expect(severity(75, null)).toBe("warn");
    expect(severity(90, null)).toBe("warn");
    expect(severity(90.1, null)).toBe("critical");
  });
  it("projeção acima de 100% é crítica", () => {
    expect(severity(30, { projectedPct: 101, limitAt: null })).toBe("critical");
    expect(severity(30, { projectedPct: 100, limitAt: null })).toBe("ok");
  });
});
