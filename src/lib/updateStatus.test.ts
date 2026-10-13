import { describe, expect, it } from "vitest";
import { formatCheckedAt, initialUpdateState, statusHint, updateStatusReducer } from "./updateStatus";

describe("updateStatusReducer", () => {
  it("marca checando e limpa o erro anterior no check_start", () => {
    const state = updateStatusReducer({ ...initialUpdateState, error: "antigo" }, { type: "check_start" });
    expect(state.checking).toBe(true);
    expect(state.error).toBeNull();
  });

  it("guarda horário e versão numa checagem bem-sucedida", () => {
    const state = updateStatusReducer(initialUpdateState, { type: "check_success", version: "2.3.0", nowMs: 1000 });
    expect(state).toMatchObject({ checking: false, lastCheckedMs: 1000, availableVersion: "2.3.0", error: null });
  });

  it("checagem sem atualização limpa uma versão vista antes", () => {
    const stale = { ...initialUpdateState, availableVersion: "2.2.0" };
    const state = updateStatusReducer(stale, { type: "check_success", version: null, nowMs: 2000 });
    expect(state.availableVersion).toBeNull();
  });

  it("falha silenciosa só para de checar, sem mexer no resto", () => {
    const before = { ...initialUpdateState, checking: true, lastCheckedMs: 500, availableVersion: "2.3.0" };
    const state = updateStatusReducer(before, { type: "check_error", silent: true, message: "offline", nowMs: 9999 });
    expect(state).toEqual({ ...before, checking: false });
  });

  it("falha manual registra horário e erro", () => {
    const state = updateStatusReducer(initialUpdateState, {
      type: "check_error",
      silent: false,
      message: "HTTP 500",
      nowMs: 3000,
    });
    expect(state).toMatchObject({ checking: false, lastCheckedMs: 3000, error: "HTTP 500" });
  });

  it("install_start limpa erro anterior", () => {
    const state = updateStatusReducer({ ...initialUpdateState, error: "antigo" }, { type: "install_start" });
    expect(state).toMatchObject({ installing: true, error: null });
  });

  it("install_error para de instalar e registra a mensagem", () => {
    const state = updateStatusReducer({ ...initialUpdateState, installing: true }, { type: "install_error", message: "falhou" });
    expect(state).toMatchObject({ installing: false, error: "falhou" });
  });

  it("install_done só para de instalar", () => {
    const state = updateStatusReducer({ ...initialUpdateState, installing: true }, { type: "install_done" });
    expect(state.installing).toBe(false);
  });
});

describe("formatCheckedAt", () => {
  it("escreve HH:MM local com zero à esquerda", () => {
    expect(formatCheckedAt(new Date(2026, 9, 9, 9, 5).getTime())).toBe("09:05");
  });
});

describe("statusHint", () => {
  it("fica indefinido antes de qualquer checagem terminar", () => {
    expect(statusHint(initialUpdateState)).toBeUndefined();
  });

  it("diz que está na versão mais recente quando não há versão nova", () => {
    const state = { ...initialUpdateState, lastCheckedMs: new Date(2026, 9, 9, 17, 9).getTime() };
    expect(statusHint(state)).toBe("Você está na versão mais recente. Verificado às 17:09.");
  });

  it("informa a versão disponível", () => {
    const state = { ...initialUpdateState, lastCheckedMs: new Date(2026, 9, 9, 17, 9).getTime(), availableVersion: "2.3.0" };
    expect(statusHint(state)).toBe("Versão 2.3.0 disponível. Verificado às 17:09.");
  });
});
