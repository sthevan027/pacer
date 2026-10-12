# OpenAI/Codex — pesquisa do passo 0 (Tarefa 3)

Data: 2026-10-12 · Decisão do Sthevan: **ler só dados locais** (sem token, sem endpoint não documentado).

## O que foi verificado e onde

Fonte: repositório oficial `openai/codex` (código aberto), `codex-rs/protocol/src/protocol.rs` e
`codex-rs/rollout/src/recorder.rs`.

- **Onde fica:** `<CODEX_HOME>/sessions/AAAA/MM/DD/rollout-<data>-<id>.jsonl`. `CODEX_HOME` é a variável de
  ambiente; sem ela, `~/.codex`.
- **Formato da linha:** JSON por linha, com `timestamp` (UTC, ISO 8601) e o item achatado no mesmo objeto
  (`type` + `payload`).
- **Evento de uso:** o evento `token_count` traz `rate_limits` com:
  - `primary` e `secondary` (cada um: `used_percent` 0–100, `window_minutes`, `resets_at` em segundos Unix);
  - `plan_type`, `credits`, `limit_id`, `limit_name`.
- Não existe `resets_in_seconds`; só `resets_at`.

## O que o Pacer faz com isso

- Lê o último `token_count` com `rate_limits` do rollout mais recente. Nenhum token é lido, nenhuma rede é usada.
- `primary`/`secondary` viram janelas pelo `window_minutes` (300 → "Sessão (5h)", 10080 → "Semanal"); nada é
  assumido por posição.
- Janela cuja `resets_at` já passou conta como 0% (a janela reiniciou).

## Limites assumidos (dizer na interface)

1. **Só reflete o que o Codex fez neste computador.** Uso em outro aparelho ou no site não aparece.
2. **É o dado da última interação.** `fetchedAt` é a hora do evento, não a da leitura.
3. Esta máquina nunca rodou uma conversa do Codex (0 threads), então a validação aqui é por fixture
   anonimizada baseada no esquema oficial, não por arquivo real. Testar com uma sessão real antes do release.
4. O formato é interno do Codex e pode mudar: o leitor ignora linhas que não entende e, sem dado, mostra
   "Sem login" em vez de quebrar.

## Ideia registrada: gasto na API (fora desta tarefa)

Mostrar quanto o usuário já gastou na API (OpenAI, Anthropic, OpenRouter) exige **chave digitada** (as de
custo/uso das duas primeiras são de administrador da organização). Isso contraria a decisão 2 do spec
("só ler o que já existe") e pede guardar a chave no Gerenciador de Credenciais do Windows. Fica como Etapa C,
com spec próprio. Endpoints e permissões precisam ser verificados na documentação oficial antes.
