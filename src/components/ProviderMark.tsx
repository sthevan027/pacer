import { ClaudeLogo, Icon } from "../icons/icons";
import { PROVIDER_LOGO_PATHS, providerColor } from "../icons/providerLogos";

/** Logo do provedor; sem logo próprio cai no ícone genérico de conexão. */
export function ProviderMark({ id, size = 15 }: { id: string; size?: number }) {
  if (id === "claude") return <ClaudeLogo size={size} />;
  const path = PROVIDER_LOGO_PATHS[id];
  if (!path) return <Icon name="plug" size={size - 1} />;
  return (
    <svg className="plogo-svg" style={{ color: providerColor(id) }} viewBox="0 0 24 24" width={size} height={size} fill="currentColor" fillRule="evenodd" aria-hidden="true">
      <path d={path} />
    </svg>
  );
}
