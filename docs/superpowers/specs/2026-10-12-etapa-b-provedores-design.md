# Pacer — Etapa B: outros provedores

Data: 2026-10-12 · Status: **rascunho para decisão** · Branch: `feat/etapa-b-provedores`

## 1. Objetivo

Hoje o Pacer só acompanha o Claude. A Etapa B adiciona outros provedores de IA
(OpenAI/Codex, OpenRouter, GitHub Copilot, Cursor, Gemini/Antigravity, z.ai/GLM),
um de cada vez, sem reescrever o que já existe.

**Sucesso:** o usuário ativa um provedor nas Configurações, ele aparece no widget
com o mesmo formato do Claude (janelas de uso, previsão, alertas, cor), e o painel
de uso passa a mostrar esse provedor nos filtros e tabelas.

**Fora de escopo:** cobrança/valor em R$ para provedores que não são de API
avulsa, sincronização na nuvem, autenticação por login web dentro do app.

## 2. Estado atual (base para a Etapa B)

O que já é genérico e pode ser reaproveitado sem mudança de contrato:

- **Trait `Provider`** (`src-tauri/src/providers/mod.rs`): `id()`, `tick_local()`
  (barato, a cada ~10 s), `fetch_remote()` (API, o scheduler decide quando) e
  `snapshot()`. Cada provedor é um módulo novo que implementa esse trait.
- **`Snapshot`** (`src-tauri/src/snapshot.rs`): `provider`, `name`, `plan`,
  `windows` (janelas de uso com `used_pct`, `resets_at` e `pace`), `activity`
  (30 dias), `today_tokens`, `fetched_at`, `next_refresh_at`, `stale` e `notice`.
  O widget já consome isso por provedor.
- **`UsageEvent`** (`src-tauri/src/usage/`, feito no painel): já tem o campo
  `provider`. O `UsageStore` e o `get_usage_report` não assumem Claude.
- **Configurações:** a seção Provedores já existe, com um toggle por provedor
  (`cfg.providers.claude.enabled`) e a linha "Adicionar provedor (em breve)".

O que é específico do Claude e **não** pode vazar para os outros:

- `providers/claude/creds.rs`: lê `~/.claude/.credentials.json`.
- `providers/claude/api.rs`: chamada à API do Claude.
- `providers/claude/logs.rs`: parser dos `.jsonl` do Claude Code.
- `usage/pricing.rs`: tabela de preços é do Claude. Cada provedor novo precisa
  de tabela própria ou de "sem valor estimado", nunca de um preço chutado.

## 3. Arquitetura proposta

### 3.1 Um módulo por provedor

```
src-tauri/src/providers/
  mod.rs        trait Provider + FetchResult (sem mudança de contrato)
  claude/       já existe
  openai/       Codex/OpenAI
  openrouter/
  copilot/
  cursor/
  gemini/
  zai/
```

Cada módulo tem a mesma forma: `creds` (onde achar a credencial local, se houver),
`api` (busca remota), opcionalmente `logs` (leitura local), e um `mod.rs` que
implementa o trait.

### 3.2 Credenciais

- Regra: o Pacer **só lê** credenciais que o próprio provedor já deixou no
  computador (como faz com o Claude Code). Não pede chave digitada na interface
  nesta etapa.
- Se um provedor só aceita chave de API digitada pelo usuário, isso vira um
  item separado de decisão (guardar a chave: no Windows Credential Manager, nunca
  em `config.json`). Fica fora da primeira leva.
- A credencial nunca chega na interface, igual ao Claude: fica só no processo Rust.

### 3.3 Registro e ativação

- O scheduler hoje conhece o Claude. Passa a iterar uma lista de provedores
  ativos em `config.providers`, cada um com `enabled`.
- Provedor desligado não faz chamada nenhuma (nem local nem remota).
- Provedor ligado sem credencial aparece com `notice: NoCredentials`, sem erro.

### 3.4 Interface

- Widget: uma aba ou seção por provedor ativo (a tela já tem abas). Os alertas
  de limite passam a valer por provedor.
- Configurações: a linha "Adicionar provedor (em breve)" vira a lista real, com
  toggle e o estado de conexão de cada um.
- Painel de uso: o filtro de tier vira filtro de provedor **e** modelo. Quem não
  tem `usd` por modelo mostra o valor como "sem tabela de preço".

### 3.5 Valor estimado

- Cada provedor declara se tem tabela de preço. Se não tiver, o painel mostra
  "sem valor estimado" em vez de R$ inventado.
- Para o Claude, a tabela continua em `pricing.rs`, com fonte e data no comentário.

## 4. Ordem de entrega

Um provedor por vez, cada um com seu próprio PR e teste manual. Ordem sugerida
(a confirmar, ver seção 6):

1. **OpenAI/Codex**: primeiro, por ser o mais próximo do modelo já implementado
   (uso por janela e por modelo).
2. **OpenRouter**: API com saldo e uso, bom segundo teste da arquitetura.
3. **GitHub Copilot**, **Cursor**, **Gemini/Antigravity**, **z.ai/GLM**: depois,
   conforme a API e a credencial local de cada um forem verificadas.

Para cada provedor, antes do código: verificar na documentação oficial **o que
existe de fato** (endpoint de uso, se é só local, se tem janela semanal). Não
assumir o formato do Claude.

## 5. Riscos

1. **Credencial local muda de formato** sem aviso. Mitigação: `NoCredentials` e
   `TokenExpired` como no Claude, sem derrubar o app.
2. **API de uso não existe ou é instável** para alguns provedores. Mitigação: o
   provedor pode ficar só com dado local, ou ser adiado.
3. **Peso de rede e de disco**: cada provedor novo soma chamadas. Mitigação:
   respeitar `refreshMinutes` e o cache por mtime já usado no Claude.
4. **Privacidade**: os logs de outros provedores podem conter nomes de projeto.
   Mesma regra do painel: fica só local, sem telemetria.
5. **Escopo crescendo**: seis provedores de uma vez tendem a atrasar tudo. Por
   isso uma leva por PR.

## 6. Decisões em aberto (do Sthevan)

1. **Ordem e corte:** quais provedores entram na 2.2.x, e quais ficam para depois?
2. **Credencial:** só ler o que o provedor já deixou no computador, ou aceitar
   chave digitada no Pacer também? (Muda o escopo da segurança.)
3. **Valor em R$:** só para provedores com tabela de preço pública, ou também
   tentar estimar os demais?
4. **Versão:** cada provedor sai numa versão (2.3.0, 2.4.0…), ou a Etapa B inteira
   sai junto numa 2.3.0?

## 7. Testes

- Unitários no Rust para o parser de cada provedor (com arquivos de exemplo
  anonimizados, nunca com credencial real).
- Teste de conferência: como no painel, comparar os totais com uma segunda
  implementação independente antes de confiar no número.
- Manual: ativar/desativar cada provedor, sem credencial, com credencial e com
  credencial vencida, e conferir o widget e as Configurações.
