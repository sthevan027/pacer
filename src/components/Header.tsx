import type { ReactNode } from "react";

interface Props {
  title: string;
  sub?: string;
  leading?: ReactNode;
  actions?: ReactNode;
}

export function Header({ title, sub, leading, actions }: Props) {
  return (
    <header className="top">
      {leading}
      <h4>
        {title}
        {sub && <span className="sub">{sub}</span>}
      </h4>
      {actions}
    </header>
  );
}
