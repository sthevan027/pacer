import { type PointerEvent as ReactPointerEvent, useEffect, useRef, useState } from "react";
import { Icon } from "../../icons/icons";
import { moveThreshold } from "../../lib/thresholds";

export function ThresholdBar({ values, onChange }: { values: number[]; onChange: (v: number[]) => void }) {
  const [local, setLocal] = useState(values);
  const bar = useRef<HTMLDivElement>(null);
  useEffect(() => setLocal(values), [values]);

  const startDrag = (i: number) => (e: ReactPointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    const el = e.currentTarget;
    el.setPointerCapture(e.pointerId);
    let current = local;
    const move = (ev: PointerEvent) => {
      const r = bar.current!.getBoundingClientRect();
      current = moveThreshold(current, i, ((ev.clientX - r.left) / r.width) * 100);
      setLocal(current);
    };
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      onChange([...current].sort((a, b) => a - b));
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
  };

  const first = local.length ? Math.min(...local) : 80;
  return (
    <div className="thr">
      <div className="tbar" ref={bar}>
        {local.map((v, i) => (
          <div key={i} className="h" style={{ left: `${v}%` }} onPointerDown={startDrag(i)}>
            <em>{v}%</em>
          </div>
        ))}
      </div>
      <div className="tcap">
        <span>0%</span>
        <span>aviso · perto do limite</span>
        <span>100%</span>
      </div>
      <div className="toast">
        <Icon name="alert" />
        <span>
          Sua sessão passou de <b>{first}</b>% — agora em {Math.min(100, first + 1)}%
        </span>
      </div>
    </div>
  );
}
