import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { UpdateRow } from "./UpdateRow";
import { initialUpdateState } from "../../lib/updateStatus";
import type { UpdateState } from "../../lib/updateStatus";

const noop = () => {};
const render = (state: UpdateState) =>
  renderToStaticMarkup(<UpdateRow update={{ state, checkNow: noop, install: noop }} />);

describe("UpdateRow", () => {
  it("oferece verificar antes de qualquer checagem", () => {
    const html = render(initialUpdateState);
    expect(html).toContain("Verificar agora");
    expect(html).not.toContain("Verificado às");
  });

  it("oferece atualizar quando há versão nova", () => {
    const html = render({ ...initialUpdateState, lastCheckedMs: new Date(2026, 9, 12, 17, 9).getTime(), availableVersion: "2.3.0" });
    expect(html).toContain("Versão 2.3.0 disponível");
    expect(html).toContain("Atualizar agora");
  });

  it("mostra o erro e desabilita o botão enquanto ocupado", () => {
    expect(render({ ...initialUpdateState, error: "sem internet" })).toContain("sem internet");
    expect(render({ ...initialUpdateState, checking: true })).toContain("disabled");
    expect(render({ ...initialUpdateState, installing: true })).toContain("disabled");
  });
});
