import { useLayoutEffect, useRef, useState } from "react";

interface Props {
  value: number;
  options: readonly number[];
  format: (v: number) => string;
  onChange: (v: number) => void;
}

export function Segmented({ value, options, format, onChange }: Props) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const [pill, setPill] = useState({ left: 0, width: 0 });

  useLayoutEffect(() => {
    const el = refs.current[options.indexOf(value)];
    if (el) setPill({ left: el.offsetLeft, width: el.offsetWidth });
    // só reposiciona quando o valor muda; `options` é constante por uso
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value]);

  return (
    <div className="segc">
      <i className="pillbg" style={pill} />
      {options.map((o, i) => (
        <button
          key={o}
          type="button"
          ref={(el) => {
            refs.current[i] = el;
          }}
          className={o === value ? "on" : ""}
          onClick={() => onChange(o)}
        >
          {format(o)}
        </button>
      ))}
    </div>
  );
}
