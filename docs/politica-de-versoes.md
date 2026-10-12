# Política de versões

> Written for: o Sthevan e quem for publicar uma release do Pacer. Vale para o repo `sthevan027/pacer`.

## Esquema

Versão no formato **SemVer**, com sufixo de pré-release antes da final:

| Tipo | Exemplo | Quando usar |
|---|---|---|
| Alpha | `2.2.0-alpha.1` | Build interno, pode quebrar. Só pra testar a ideia ou um fluxo novo |
| Beta | `2.2.0-beta.1` | Funcionalidades completas, aberto a testers. Correções entram como `beta.2`, `beta.3`… |
| RC | `2.2.0-rc.1` | Candidata a final. Só correção de bug, sem feature nova |
| Final | `2.2.0` | Versão pública. É a única que aparece no site e fica como *Latest* |

- **MAJOR.MINOR.PATCH:** `MAJOR` quebra compatibilidade, `MINOR` é funcionalidade nova, `PATCH` é correção.
- **Tag Git:** `v` + versão. Ex.: `v2.2.0-beta.1`.
- **Ordem:** `2.2.0-alpha.1` < `2.2.0-beta.1` < `2.2.0-rc.1` < `2.2.0`.

## Onde a versão mora

A versão precisa ser igual em **três arquivos**:

1. `package.json` → `version`
2. `src-tauri/tauri.conf.json` → `version` (define o nome do instalador)
3. `src-tauri/Cargo.toml` → `version`

O instalador sai como `Pacer_<versão>_x64-setup.exe`. O site só reconhece instaladores com esse padrão (`pickInstaller` em `site/src/lib/release.ts`), então não mude o nome.

## Regras de publicação no GitHub

- **Pré-release** (alpha, beta, rc): criar com a opção **Set as a pre-release** e **sem** marcar como *Latest*.
- **Final:** criar com **Set as the latest release**.
- **Nunca** publicar release em rascunho (*draft*) para o site enxergar. O site ignora `draft` e `prerelease`.
- **Notas de release:** o site extrai o resumo da seção `## Novidades`, com itens de lista (`- **Destaque** — texto`). Seções com "instala", "requisitos", "assinad", "sha256" ou ⚠ são ignoradas. Mantenha esse formato.

## Quem vê o quê

| Canal | Onde aparece | Como o usuário instala |
|---|---|---|
| Final | Site (botão de download e Novidades), GitHub como *Latest* | Baixa o instalador no site ou no GitHub |
| Beta / RC / alpha | Só no GitHub, na lista de pré-releases | Baixa manualmente em *Releases* |

O site **nunca** mostra pré-release: o botão de download pega a final mais recente e a lista de Novidades filtra `prerelease: true`. Os testes em `site/src/lib/release.test.ts` cobrem isso.

## Passo a passo para publicar

1. **Branch** de release, ex.: `release/2.2.0-beta.1`.
2. **Bump** da versão nos três arquivos acima. Commit: `chore(release): 2.2.0-beta.1`.
3. **Build:** `bun run tauri build`. O instalador sai em `src-tauri/target/release/bundle/nsis/`. A assinatura usa o certificado configurado em `tauri.conf.json`.
4. **Testar** o instalador gerado numa máquina limpa ou instalando por cima da versão anterior.
5. **Tag** no commit de release: `git tag v2.2.0-beta.1` e `git push origin v2.2.0-beta.1`.
6. **Criar a release** com o instalador anexado. Exemplo:
   ```bash
   gh release create v2.2.0-beta.1 \
     "src-tauri/target/release/bundle/nsis/Pacer_2.2.0-beta.1_x64-setup.exe" \
     --prerelease --title "Pacer 2.2.0-beta.1: <resumo>" --notes-file NOTAS.md
   ```
7. **Promover a final** (quando o beta estiver estável):
   - Repetir os passos com a versão sem sufixo (`2.2.0`), usando `--latest` em vez de `--prerelease`.
   - Atualizar `FALLBACK_INSTALLER_URL` em `site/src/lib/release.ts` para a nova versão. Commit junto da release.
   - Atualizar `site/src/content/pt.ts` (`news.fallback`) com a nova versão, se quiser que o fallback de Novidades acompanhe.

## Upgrade entre canais

O NSIS instala por cima da versão anterior, então ir de beta para final (ou de final para beta) é só rodar o instalador novo. Como o app **não tem auto-update**, o usuário precisa baixar e instalar manualmente.

## O que ainda não existe

- **Canal beta dentro do app.** Exigiria o plugin de updater do Tauri, um par de chaves de assinatura do updater e um endpoint por canal (ex.: um `latest.json` para beta e outro para final). Hoje não está implementado.
- **CI de release.** O `.github/` está vazio. Dá para automatizar os passos 3 a 6 com um workflow que dispara ao criar a tag.

## Histórico

- Tags `v1.x` são do Claude Glass, antes da recriação do Pacer em Tauri. Não são referência para o fluxo atual.
- A partir de `v2.0.0`, o fluxo é o descrito aqui (sem pré-releases até agora).
