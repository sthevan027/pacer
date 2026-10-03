import type { ReactNode } from "react";

interface Props {
  title: string;
  sub?: string;
  draggable: boolean;
  leading?: ReactNode;
  actions?: ReactNode;
}

export function Header({ title, sub, draggable, leading, actions }: Props) {
  const drag = draggable ? { "data-tauri-drag-region": "" } : {};
  return (
    <header className="top" {...drag}>
      {leading}
      <h4>
        {title}
        {sub && <span className="sub">{sub}</span>}
      </h4>
      {actions}
    </header>
  );
}
