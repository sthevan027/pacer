import { describe, expect, it } from "vitest";
import { billableTokens, cacheWrite, EMPTY_TOTALS, metricValue, totalTokens, type Totals } from "./usage";

const t = (patch: Partial<Totals> = {}): Totals => ({ ...EMPTY_TOTALS, ...patch });

describe("somas", () => {
  it("junta as duas faixas de escrita de cache", () => {
    expect(cacheWrite(t({ cacheWrite5m: 400, cacheWrite1h: 600 }))).toBe(1000);
  });

  it("entrada + saída ignora o cache", () => {
    expect(billableTokens(t({ input: 2, output: 26, cacheRead: 171107, cacheWrite5m: 3186 }))).toBe(28);
  });

  it("o total é a mesma soma que o widget usa", () => {
    expect(totalTokens(t({ input: 2, output: 26, cacheRead: 171107, cacheWrite5m: 3186 }))).toBe(174321);
  });

  it("cache escrito de 1h entra no total", () => {
    expect(totalTokens(t({ cacheWrite1h: 50 }))).toBe(50);
  });
});

describe("metricValue", () => {
  const amostra = t({ input: 10, output: 5, cacheRead: 100, cacheWrite5m: 7, cacheWrite1h: 3 });

  it("cada métrica pega a sua parte", () => {
    expect(metricValue(amostra, "tokens")).toBe(15);
    expect(metricValue(amostra, "cacheRead")).toBe(100);
    expect(metricValue(amostra, "cacheWrite")).toBe(10);
    expect(metricValue(amostra, "total")).toBe(125);
  });

  it("métrica zerada não vira NaN", () => {
    expect(metricValue(EMPTY_TOTALS, "total")).toBe(0);
  });
});
