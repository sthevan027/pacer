# Etapa B — Outros provedores: plano de implementação

**Spec:** [2026-10-12-etapa-b-provedores-design.md](../specs/2026-10-12-etapa-b-provedores-design.md)
**Branch:** `feat/etapa-b-provedores` (a partir da `main`)

## Decisões assumidas (padrão do spec, ajustar se mudar)

As decisões da seção 6 do spec ainda não foram respondidas. Este plano assume:

1. **Ordem:** OpenAI/Codex primeiro; depois OpenRouter; Copilot, Cursor, Gemini/Antigravity e z.ai ficam para planos próprios.
2. **Credencial:** só ler o que o provedor já deixou no computador. Nada de chave digitada no Pacer nesta etapa.
3. **Valor em R$:** só onde há tabela pública. Sem tabela, o painel mostra "sem valor estimado".
4. **Versão:** cada provedor sai no próprio PR. A versão de release fica para o fim da Etapa B.

## Regras globais

- TDD no Rust e no TypeScript: teste antes da implementação, e ver falhar.
- Nenhum teste usa credencial real. Fixtures são anonimizadas e ficam em `src-tauri/tests/fixtures/`.
- Verificar a documentação oficial de cada provedor antes de codar o módulo (Tarefa 3, passo 0).
- Um PR por tarefa grande. PR leva imagens (regra do projeto).
- Commits com as linhas de atribuição do lembrete da sessão.

## Mapa de arquivos

- Modificar: `src-tauri/src/scheduler.rs` (de um provedor fixo para uma lista), `src-tauri/src/config.rs` (lista de provedores ativos), `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/providers/mod.rs`.
- Criar: `src-tauri/src/providers/openai/{mod.rs,creds.rs,api.rs}`.
- Modificar UI: `src/components/settings/SettingsView.tsx`, `src/components/ProviderTabs.tsx`, `src/components/ProviderCard.tsx`, `src/components/MainView.tsx`, `src/components/panel/*`.
- Modificar: `src-tauri/src/usage/pricing.rs` (suporte a provedor sem tabela).

---

## Tarefa 1: Scheduler com lista de provedores (sem mudar comportamento)

**Objetivo:** o `scheduler.rs` deixa de criar só `ClaudeProvider` e passa a manter uma lista `Vec<Box<dyn Provider>>`, com o Claude como único item. O comportamento visível não muda.

- [ ] **Passo 1 — teste:** em `scheduler.rs`, escrever um teste que monta o scheduler com dois provedores falsos (um ligado, um desligado) e confirma que só o ligado é consultado. Deve falhar.
- [ ] **Passo 2 — implementar:** trocar o `provider: Box<dyn Provider>` por uma lista construída a partir de `config.providers`. Provedor desligado nunca é chamado, nem `tick_local` nem `fetch_remote`.
- [ ] **Passo 3 — rodar:** `cargo test --manifest-path src-tauri/Cargo.toml`. Todos passam, incluindo os 102 existentes.
- [ ] **Passo 4 — commit.**

## Tarefa 2: Configuração por provedor

**Objetivo:** `config.providers` ganha entradas para cada provedor futuro, com `enabled`, mantendo compatibilidade com o `config.json` atual.

- [ ] **Passo 1 — teste:** ler um `config.json` antigo (só com `claude`) e confirmar que o provedor `openai` aparece desligado por padrão, sem erro.
- [ ] **Passo 2 — implementar:** `ProvidersConfig` ganha `openai: ProviderToggle` com `#[serde(default)]`.
- [ ] **Passo 3 — teste de regressão:** `config.rs` já tem testes de `enabled` (linhas ~90 e ~247). Devem continuar passando.
- [ ] **Passo 4 — commit.**

## Tarefa 3: Módulo OpenAI/Codex

**Objetivo:** o provedor OpenAI/Codex aparece no widget com janelas de uso, quando a credencial local existir.

- [ ] **Passo 0 — pesquisa (obrigatória):** na documentação oficial do OpenAI/Codex, confirmar:
  - onde o Codex guarda a credencial local e o formato (sem copiar o token para o repo);
  - se existe endpoint de uso/limite, ou se o uso só é visível em log local;
  - se há janela semanal ou só de sessão.
  Registrar o resultado numa nota curta em `docs/superpowers/specs/` antes de codar. Se não houver fonte oficial de uso, parar e reportar antes de continuar.
- [ ] **Passo 1 — teste de creds:** `creds.rs` acha o arquivo de credencial numa pasta temporária criada no teste, e recusa credencial vencida com `Notice::TokenExpired`.
- [ ] **Passo 2 — implementar `creds.rs`** para o formato confirmado no passo 0.
- [ ] **Passo 3 — teste de parser:** com fixture anonimizada, o parser devolve `UsageWindow` com `used_pct` correto.
- [ ] **Passo 4 — implementar `api.rs`/parser** e o `impl Provider for OpenAiProvider`.
- [ ] **Passo 5 — teste de conferência:** comparar os totais do parser com uma segunda conta feita à mão sobre a mesma fixture, como foi feito no painel.
- [ ] **Passo 6 — registrar no scheduler** (Tarefa 1) e na config (Tarefa 2).
- [ ] **Passo 7 — rodar** `cargo test` e `bun run test`.
- [ ] **Passo 8 — commit.**

## Tarefa 4: Configurações — lista de provedores

**Objetivo:** a linha "Adicionar provedor (em breve)" vira a lista real de provedores, com toggle e estado de conexão.

- [ ] **Passo 1 — teste:** `SettingsView` (ou um componente `ProviderList`) renderiza uma linha por provedor, com "Sem login" quando `notice === NoCredentials`. Deve falhar.
- [ ] **Passo 2 — implementar** a lista, reaproveitando o `Toggle` e o `Row` existentes. Visual igual ao do Claude na seção Provedores.
- [ ] **Passo 3 — rodar** `bun run test` e `bun run build`.
- [ ] **Passo 4 — commit.**

## Tarefa 5: Widget — um card por provedor ativo

**Objetivo:** o widget mostra um card por provedor ativo, reaproveitando `ProviderCard`. As abas (`ProviderTabs`) já existem.

- [ ] **Passo 1 — teste:** com dois snapshots (Claude e OpenAI), a aba mostra os dois; com um provedor desligado, ele não aparece.
- [ ] **Passo 2 — implementar.**
- [ ] **Passo 3 — alertas:** `alerts.rs` passa a avaliar limite por provedor. Teste: alerta de um provedor não dispara o outro.
- [ ] **Passo 4 — commit.**

## Tarefa 6: Painel de uso — filtro por provedor e valor sem tabela

**Objetivo:** o painel filtra por provedor além de modelo, e mostra "sem valor estimado" para provedor sem tabela.

- [ ] **Passo 1 — teste** em `usage/mod.rs`: o relatório separa os eventos por provedor, e um evento sem preço fica com valor `None`, sem entrar no total em R$.
- [ ] **Passo 2 — implementar** o filtro de provedor no comando `get_usage_report` e na UI do painel.
- [ ] **Passo 3 — teste de conferência** dos totais por provedor.
- [ ] **Passo 4 — commit.**

## Tarefa 7: Verificação e PR

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` e `bun run test` passam.
- [ ] Build de instalador de teste (sem artefato de updater, como na 2.2.0-beta.1).
- [ ] Teste manual: ativar, desativar, sem credencial, com credencial vencida, para cada provedor da leva.
- [ ] Capturas da tela de Configurações e do widget com o provedor novo, com dados reais ou fictícios conforme você decidir.
- [ ] PR para a `main` com as imagens na descrição.

---

## Próximos provedores

OpenRouter, Copilot, Cursor, Gemini/Antigravity e z.ai repetem as Tarefas 3 a 7, cada um em plano e PR próprios. A pesquisa do passo 0 vale para cada um: nenhum formato é assumido do Claude nem do OpenAI.
