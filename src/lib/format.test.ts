import { describe, expect, it } from "vitest";
import { formatAgo, formatBrl, formatDay, formatDuration, formatStamp, formatTokens, formatUsd } from "./format";

const MIN = 60_000;

describe("formatDuration", () => {
  it("formata dias, horas e minutos", () => {
    expect(formatDuration((31 * 60 + 5) * MIN)).toBe("1d 7h");
    expect(formatDuration(134 * MIN)).toBe("2h 14m");
    expect(formatDuration(14 * MIN)).toBe("14m");
    expect(formatDuration(20_000)).toBe("menos de 1m");
    expect(formatDuration(-5 * MIN)).toBe("menos de 1m");
  });
});

describe("formatTokens", () => {
  it("usa vírgula decimal e sufixos", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(999)).toBe("999");
    expect(formatTokens(850_000)).toBe("850k");
    expect(formatTokens(4_200_000)).toBe("4,2M");
    expect(formatTokens(96_000_000)).toBe("96M");
    expect(formatTokens(1_250_000_000)).toBe("1,3B");
  });
});

describe("formatAgo", () => {
  const now = Date.parse("2026-10-03T12:00:00Z");
  it("diz há quanto tempo", () => {
    expect(formatAgo("2026-10-03T11:59:40Z", now)).toBe("agora");
    expect(formatAgo("2026-10-03T11:57:00Z", now)).toBe("há 3 min");
    expect(formatAgo("2026-10-03T09:00:00Z", now)).toBe("há 3 h");
    expect(formatAgo("2026-10-01T12:00:00Z", now)).toBe("há 2 d");
  });
});

describe("formatDay", () => {
  it("formata dia e mês curto sem depender do fuso", () => {
    expect(formatDay("2026-09-12")).toBe("12 set");
    expect(formatDay("2026-01-01")).toBe("1 jan");
  });
});

describe("moeda", () => {
  // O Intl do pt-BR separa o "R$" com espaço NÃO separável (U+00A0), não com espaço comum.
  // Não troque por um espaço normal: o teste passa a falhar por um caractere invisível.
  const NBSP = " ";

  it("formata em real e em dólar", () => {
    expect(formatBrl(1234.5)).toBe(`R$${NBSP}1.234,50`);
    expect(formatUsd(4)).toBe("$4.00");
  });

  it("não perde valores pequenos, que são a maioria das linhas", () => {
    expect(formatBrl(0.03)).toBe(`R$${NBSP}0,03`);
    expect(formatBrl(0)).toBe(`R$${NBSP}0,00`);
  });
});

describe("formatStamp", () => {
  it("mostra dia, mês, hora e minuto", () => {
    const d = new Date(2026, 9, 10, 9, 4);
    expect(formatStamp(d.toISOString())).toBe("10/10 09:04");
  });
});
