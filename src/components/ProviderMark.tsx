import { ClaudeLogo, Icon } from "../icons/icons";

/** Logo do provedor; os que ainda não têm logo próprio usam o ícone genérico de conexão. */
export function ProviderMark({ id, size = 15 }: { id: string; size?: number }) {
  return id === "claude" ? <ClaudeLogo size={size} /> : <Icon name="plug" size={size - 1} />;
}
