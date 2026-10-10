import { useEffect, useState } from "react";

/**
 * Campo da cotação. Guarda o texto enquanto se digita (senão o valor salvo a cada tecla
 * reformata o campo e o cursor pula) e só grava quando o campo perde o foco ou recebe Enter.
 */
export function RateInput({ value, onCommit }: { value: number; onCommit: (v: number) => void }) {
  const [texto, setTexto] = useState(String(value));

  // valor salvo mudou por fora (outra janela, normalização): volta a refletir
  useEffect(() => setTexto(String(value)), [value]);

  const commitar = () => {
    const n = Number(texto.replace(",", "."));
    if (Number.isFinite(n) && n > 0) onCommit(n);
    else setTexto(String(value));
  };

  return (
    <input
      className="numin"
      type="text"
      inputMode="decimal"
      aria-label="Cotação do dólar"
      value={texto}
      onChange={(e) => setTexto(e.target.value)}
      onBlur={commitar}
      onKeyDown={(e) => {
        if (e.key === "Enter") e.currentTarget.blur();
      }}
    />
  );
}
