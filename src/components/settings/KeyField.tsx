import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import type { ProviderDef } from "../../lib/providers";

interface Props {
  provider: ProviderDef;
  /** Já existe uma chave guardada no Windows para este provedor. */
  stored: boolean;
  /** Chamado depois de guardar, para ligar o provedor se ainda estiver desligado. */
  onSaved: () => void;
}

/**
 * Campo para colar a chave de API. O valor só passa daqui para o Rust (que o guarda no
 * Gerenciador de Credenciais do Windows); depois de salvar o campo é limpo e a interface
 * nunca mais recebe a chave de volta, só o aviso de que existe uma.
 */
export function KeyField({ provider, stored, onSaved }: Props) {
  const cfg = provider.apiKey;
  const [text, setText] = useState("");
  const [replacing, setReplacing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (!cfg) return null;

  const run = async (cmd: string, args: Record<string, unknown>, done?: () => void) => {
    setBusy(true);
    setError(null);
    try {
      await invoke<string[]>(cmd, args);
      setText("");
      setReplacing(false);
      done?.();
    } catch (e) {
      setError(typeof e === "string" ? e : "Não consegui guardar a chave.");
    } finally {
      setBusy(false);
    }
  };

  const save = () => run("set_api_key", { provider: provider.id, key: text }, onSaved);
  const showInput = !stored || replacing;

  return (
    <div className="keyf">
      {showInput ? (
        <form
          className="keyr"
          onSubmit={(e) => {
            e.preventDefault();
            if (text.trim() && !busy) save();
          }}
        >
          <input
            className="keyin"
            type="password"
            aria-label={`Chave de API do ${provider.name}`}
            placeholder={cfg.placeholder}
            autoComplete="off"
            spellCheck={false}
            value={text}
            onChange={(e) => setText(e.target.value)}
          />
          <button type="submit" className="keybtn pri" disabled={busy || !text.trim()}>
            Guardar
          </button>
          {replacing && (
            <button
              type="button"
              className="keybtn"
              onClick={() => {
                setReplacing(false);
                setText("");
                setError(null);
              }}
            >
              Cancelar
            </button>
          )}
        </form>
      ) : (
        <div className="keyr">
          <span className="keyok">Chave guardada no Windows</span>
          <button type="button" className="keybtn" disabled={busy} onClick={() => setReplacing(true)}>
            Trocar
          </button>
          <button type="button" className="keybtn" disabled={busy} onClick={() => run("clear_api_key", { provider: provider.id })}>
            Remover
          </button>
        </div>
      )}
      {error && (
        <span className="keyerr" role="alert">
          {error}
        </span>
      )}
      <span className="keyhint">
        Fica no Gerenciador de Credenciais do Windows, só neste PC.{" "}
        <a
          href={cfg.getUrl}
          onClick={(e) => {
            e.preventDefault();
            openUrl(cfg.getUrl);
          }}
        >
          {cfg.getLabel}
        </a>
      </span>
    </div>
  );
}
