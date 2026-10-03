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
  it("a animação de entrada do cartão não segura a opacidade final (senão .stale nunca apaga)", () => {
    // fill-mode `both`/`forwards` mantém o `to { opacity: 1 }` e vence `.sec.stale { opacity: .7 }`
    expect(rule(".sec")).not.toMatch(/animation:[^;]*\b(both|forwards)\b/);
    expect(rule(".sec.stale")).toMatch(/opacity:\s*0\.7/);
  });
});
