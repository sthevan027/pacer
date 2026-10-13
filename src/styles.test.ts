/// <reference types="node" />
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./styles.css", import.meta.url), "utf8");

function rule(selector: string): string {
  const start = css.indexOf(`\n${selector} {`);
  expect(start, `regra ${selector} não encontrada`).toBeGreaterThan(-1);
  return css.slice(start, css.indexOf("}", start) + 1);
}

describe("estilos", () => {
  it("as abas de provedor dividem a largura e encolhem (5 provedores cabem no widget)", () => {
    expect(rule(".tab")).toMatch(/flex:\s*1 1 0/);
    expect(rule(".tab")).toMatch(/min-width:\s*0/);
  });

  it("o widget rola na vertical quando o conteúdo passa da altura da tela (Configurações com vários provedores)", () => {
    expect(rule("html, body")).toMatch(/overflow-y:\s*auto/);
    expect(rule("html, body")).toMatch(/overflow-x:\s*hidden/);
  });

  it("a animação de entrada do cartão não segura a opacidade final (senão .stale nunca apaga)", () => {
    // fill-mode `both`/`forwards` mantém o `to { opacity: 1 }` e vence `.sec.stale { opacity: .7 }`
    expect(rule(".sec")).not.toMatch(/animation:[^;]*\b(both|forwards)\b/);
    expect(rule(".sec.stale")).toMatch(/opacity:\s*0\.7/);
  });
});
