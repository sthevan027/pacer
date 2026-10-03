import type { ReactNode } from "react";

export function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="row">
      <div className="lbl">
        <b>{label}</b>
        {hint && <span>{hint}</span>}
      </div>
      {children}
    </div>
  );
}
