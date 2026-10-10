/** Ícone do app (app-icon.svg): anel de uso com ponteiro. O arco segue a cor de destaque da página. */
export function Logo({ size = 28 }: { size?: number }) {
  return (
    <svg className="logo" width={size} height={size} viewBox="0 0 1024 1024" aria-hidden="true">
      <rect width="1024" height="1024" rx="224" fill="#181c22" />
      <circle cx="512" cy="512" r="300" fill="none" stroke="#2d333b" strokeWidth="88" />
      <path d="M512 212 A300 300 0 1 1 252 662" fill="none" stroke="var(--accent)" strokeWidth="88" strokeLinecap="round" />
      <rect x="496" y="150" width="32" height="150" rx="16" fill="#e6edf3" />
    </svg>
  );
}
