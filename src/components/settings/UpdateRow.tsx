import { statusHint } from "../../lib/updateStatus";
import type { UseUpdateStatus } from "../../lib/useUpdateStatus";
import { Row } from "./Row";

export function UpdateRow({ update }: { update: UseUpdateStatus }) {
  const { state, checkNow, install } = update;
  const busy = state.checking || state.installing;
  const label = state.installing
    ? "Atualizando…"
    : state.checking
      ? "Verificando…"
      : state.availableVersion
        ? "Atualizar agora"
        : "Verificar agora";
  const onClick = state.availableVersion ? install : checkNow;
  return (
    <>
      <Row label="Atualizações" hint={statusHint(state)}>
        <button className="updbtn" onClick={onClick} disabled={busy}>
          {label}
        </button>
      </Row>
      {state.error && <p className="upderr">{state.error}</p>}
    </>
  );
}
