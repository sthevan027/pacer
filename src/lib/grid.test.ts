import { describe, expect, it } from "vitest";
import { buildGrid, levelFor } from "./grid";
import type { DayActivity } from "./types";

function last30(endIso: string, tokens: (i: number) => number): DayActivity[] {
  const [y, m, d] = endIso.split("-").map(Number);
  const end = Date.UTC(y, m - 1, d);
  return Array.from({ length: 30 }, (_, i) => {
    const date = new Date(end - (29 - i) * 86_400_000).toISOString().slice(0, 10);
    return { date, tokens: tokens(i), messages: i };
  });
}

describe("levelFor", () => {
  it("divide em 5 níveis relativos ao máximo", () => {
    expect(levelFor(0, 100)).toBe(0);
    expect(levelFor(1, 100)).toBe(1);
    expect(levelFor(25, 100)).toBe(1);
    expect(levelFor(26, 100)).toBe(2);
    expect(levelFor(75, 100)).toBe(3);
    expect(levelFor(100, 100)).toBe(4);
    expect(levelFor(5, 0)).toBe(0);
  });
});

describe("buildGrid", () => {
  it("monta colunas por semana com segunda na primeira linha", () => {
    // 30 dias até sáb 03/10/2026 → começa sex 04/09
    const weeks = buildGrid(last30("2026-10-03", (i) => i * 10));
    expect(weeks).toHaveLength(5);
    expect(weeks.every((w) => w.length === 7)).toBe(true);
    expect(weeks[0][3]).toBeNull(); // quinta antes do início
    expect(weeks[0][4]?.date).toBe("2026-09-04");
    expect(weeks[4][5]?.date).toBe("2026-10-03");
    expect(weeks[4][6]).toBeNull(); // domingo depois de hoje
    expect(weeks[4][5]?.level).toBe(4);
    expect(weeks[0][4]?.level).toBe(0);
  });

  it("lista vazia não quebra", () => {
    expect(buildGrid([])).toEqual([]);
  });
});
