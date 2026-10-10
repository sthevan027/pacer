/** true quando o navegador parece ser Windows (o Pacer só roda lá). */
export function isWindows(nav: Pick<Navigator, "userAgent"> & { userAgentData?: { platform?: string } }): boolean {
  const platform = nav.userAgentData?.platform;
  if (platform) return /windows/i.test(platform);
  return /windows/i.test(nav.userAgent);
}
