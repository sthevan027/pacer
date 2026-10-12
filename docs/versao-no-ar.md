# Versão no ar

> Written for: o Sthevan, pra saber em um lugar o que está publicado agora e o que falta. Data de referência: 2026-10-12.

## Versão final publicada

| Item | Valor |
|---|---|
| Versão | **v2.1.0** (marcada como *Latest* no GitHub) |
| Publicada em | 2026-10-09 |
| Instalador | `Pacer_2.1.0_x64-setup.exe` |
| Assinatura | certificado autoassinado `CN=Sthevan.Dev` |
| Release | https://github.com/sthevan027/pacer/releases/tag/v2.1.0 |

Versão nos três arquivos do app, todos iguais em `2.1.0`:
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

## Histórico de releases no GitHub

| Tag | Data | Observação |
|---|---|---|
| v2.1.0 | 2026-10-09 | Final, *Latest* |
| v2.0.1 | 2026-10-04 | Final |
| v2.0.0 | 2026-10-03 | Final |
| v1.2.1 | 2026-09-25 | Tags v1.x são anteriores à recriação em Tauri (Claude Glass) |
| v1.2.0 | 2026-09-09 | Claude Glass |
| v1.1.0 | 2026-08-10 | Claude Glass |

Pré-releases (alpha/beta/rc) publicadas: **nenhuma**.

## Site (landing)

- **Código:** pronto no `main`. Os commits `a2e3ee2` (configuração do Netlify) e `52f37e6` (correção do caminho de publicação) já estão no `origin/main`.
- **Deploy:** o primeiro deploy no Netlify falhou em 12/10 porque o caminho de publicação estava errado. A correção está no `main`, mas **não sei se já houve um novo deploy**. Confira no painel do Netlify. Enquanto isso, o site não está no ar.
- **Downloads:** o botão usa o instalador da release final mais recente. Se a API do GitHub falhar, usa o link fixo em `site/src/lib/release.ts` (`FALLBACK_INSTALLER_URL`), que aponta para `v2.1.0`.
- **Mudança pendente (não commitada):** `per_page` da API de releases foi de 4 para 10, pra que pré-releases não esvaziem a lista de finais. Os 13 testes do site passam.

## Lacunas conhecidas

- **Sem auto-update no app.** Não há plugin de updater do Tauri nem endpoint. O usuário precisa baixar o instalador manualmente.
- **Sem canal beta/alpha no app.** Pré-releases hoje só existem no GitHub, para download manual.
- **Sem CI de release.** O `.github/` do repo está vazio; o build e a publicação são manuais. O processo está em [politica-de-versoes.md](politica-de-versoes.md).
- **Fallback do site desatualizado a cada release.** Precisa ser atualizado junto com cada final.
- **Sem PR aberto no GitHub** agora (`gh pr list` vazio). A nota do vault sobre o painel de uso (PR #5) dizia que ele estava aberto, então vale confirmar se foi mergeado ou fechado.
