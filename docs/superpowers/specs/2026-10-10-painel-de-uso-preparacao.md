# Pacer — Painel de uso (preparação)

Data: 2026-10-09 (preparação para a janela de execução de 2026-10-10)
Status: **rascunho / estudo** — nenhum código foi alterado por este documento.

## 1. Objetivo

Além do widget (que só mostra Sessão 5h, Semanal e a grade de 30 dias), criar uma
**tela à parte** — um painel — onde a pessoa vê com mais detalhe o consumo: tokens
por dia e por modelo, divisão entrada/saída/cache, quais sessões e projetos mais
gastaram, e uma lista das requisições recentes. Referência visual: a página "Uso"
do console da Anthropic (3 capturas, seção 3).

O widget continua sendo o produto principal e continua leve. O painel é secundário:
só gasta memória/CPU enquanto estiver aberto.

## 2. Agenda de amanhã

Evento no Google Calendar: **sáb 2026-10-10, 14:00–17:00 — "Pacer — finalizar a
Etapa B (outros provedores)"**. A descrição dele fala de escolher provedores
(OpenAI/Codex, OpenRouter, Copilot, Cursor, Gemini, z.ai...), spec/plano da etapa B e
release. **Não menciona o painel.**

> **Decisão pendente (do Sthevan):** a janela de amanhã é para a Etapa B, para o
> painel, ou o painel vira a prioridade e a Etapa B fica pro próximo bloco? Se o
> painel entrar, vale atualizar a descrição do evento. (No mesmo dia há outro evento
> às 19:00, Landing page/catálogo Virex — separado, não conflita.)

Ligação entre os dois: o painel deve nascer **independente de provedor** (seção 6.2),
senão a Etapa B obriga a refazê-lo.

## 3. O que as capturas de referência mostram

Fonte: `C:\Users\Sthev\OneDrive\Pictures\Screenshots\Captura de tela 2026-10-09 17340{1,4,8}.png`
(não copiadas pro repositório: são UI de terceiro, servem só de referência).

1. **"Consumo por tier"** — gráfico de área de tokens (entrada + saída) por dia;
   seletor de período **7 / 14 / 30 dias**; filtro de tier **Todos / Sonnet / Opus /
   Fable** (cada um com uma cor); abaixo, três colunas (uma por tier) com
   **Entrada, Saída, Valor (R$)**. Rodapé explicando medição: "tokens confirmados /
   estimados / sem origem identificada".
2. **"Registros recentes"** — tabela: Data (com rótulo "Tokens confirmados" ou
   "Medição não identificada"), Tier (bolinha colorida + nome), Entrada, Saída, Cache,
   Valor (R$); link "Ver as últimas 100 →".
3. **Página "Uso"** — título com selo "AO VIVO", abas **Visão geral / Registros**, mesmos
   filtros de período e tier, tabela "Requisições" (uma linha por requisição, da mais
   recente para a mais antiga) e botão **Exportar tudo (CSV)**.

Estética: tema escuro, cartões com borda sutil, rótulos em caixa-alta espaçada,
números alinhados à direita, paleta quente (tons de laranja por tier). Combina com o
visual atual do Pacer (cor de destaque configurável).

## 4. Referência → Pacer

| Elemento da referência | Equivalente no Pacer | Fonte de dado | Situação |
|---|---|---|---|
| Gráfico de área tokens/dia | Área/barras empilhadas por modelo | logs `.jsonl` | **Precisa mudar o parser** (hoje soma tudo num número por dia) |
| Período 7/14/30 | Mesmo seletor | logs | 7 e 14 triviais; 30 é o limite atual (`DAYS = 30` em `logs.rs`) |
| Filtro por tier | Filtro por modelo (Opus/Sonnet/Haiku...) | `message.model` | **Descartado hoje**, mas existe no log |
| Entrada / Saída / Cache | Idem | `usage.input_tokens`, `output_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens` | **Hoje somados num total só** (e o total infla com cache, ver 7.2) |
| Valor (R$) | "Equivalente a preço de API" estimado | tokens × tabela de preços | **Novo**; exige tabela de preços (ver 7.3) |
| Tabela de requisições | Mesma, por mensagem do assistente | `timestamp`, `message.id`+`requestId` | Viável; precisa guardar a linha, não só o total |
| "Medição não identificada" | Sem equivalente | — | Não se aplica: lemos o log local, não medimos pela conta |
| Selo "AO VIVO" | Indicador de atualização | scheduler já existe | Reaproveitar `fetchedAt`/`nextRefreshAt` |
| Exportar CSV | Exportar a tabela filtrada | — | Fácil (comando Rust grava em Downloads) |
| (extra, nosso) Ranking de **sessões** | Top sessões por tokens | `sessionId` | Novo, não existe na referência |
| (extra, nosso) Ranking de **projetos** | Top projetos por tokens | `cwd` / pasta em `projects/` | Novo; o Claude Code agrupa logs por projeto |
| (extra, nosso) Marcos de limite | Pontos "limite atingido" no gráfico | `quotaLimits` (só aparece quando bate o limite) | Opcional, investigar |

## 5. O que os logs já têm (verificado em 2026-10-09)

Em `~/.claude/projects`: 16 pastas de projeto, 155 arquivos `.jsonl`, ~506 MB, 80
sessões distintas. Cada linha de mensagem do assistente traz:

- `timestamp`, `sessionId`, `cwd`, `gitBranch`, `isSidechain` (subagentes), `requestId`
- `message.id`, `message.model` (ex.: `claude-opus-5-5`, `claude-sonnet-5`, `claude-sonnet-5-5`,
  `claude-sonnet-4-6`, `claude-haiku-4-5-...`, e `<synthetic>` que já é ignorado)
- `message.usage`: `input_tokens`, `output_tokens`, `cache_read_input_tokens`,
  `cache_creation_input_tokens`, `service_tier`, `speed`, entre outros
- `quotaLimits` (raro): `status`, `rateLimitType`, `resetsAt`, `isUsingOverage`...

Hoje `parse_line` (`src-tauri/src/providers/claude/logs.rs`) guarda só
`{ ts, tokens (soma de tudo), key }` e `scan()` agrega em `DayActivity { date, tokens,
messages }`. Ou seja: **os dados do painel já existem no disco; o trabalho é parar de
jogá-los fora e agregar de outro jeito.**

## 6. Arquitetura proposta (rascunho para decidir amanhã)

### 6.1 Janela

- Nova janela `panel`: normal (com barra de título ou decoração própria), redimensionável,
  na barra de tarefas enquanto aberta. Rota `index.html?view=panel`, no mesmo esquema do
  `?view=hover` (`App.tsx`).
- Abrir por: item "Abrir painel de uso" no menu da bandeja e um botão no cabeçalho do widget.
- **Atenção ao deadlock** (ver 7.1): criar a janela fora da thread principal.
- Lembrar de incluir `"panel"` em `capabilities/default.json` (lista `windows`).
- Ao fechar, destruir a janela (libera o WebView2) em vez de só esconder, já que é de uso
  esporádico — o oposto do `main`, que fica sempre vivo.

### 6.2 Dados

- Modelo **independente de provedor**: `UsageEvent { ts, provider, model, session, project,
  input, output, cache_read, cache_write }`. O Claude é o primeiro produtor; a Etapa B
  adiciona outros produtores sem mexer no painel.
- O painel **não** vai dentro do `Snapshot` (que é empurrado ao widget a cada refresh e
  ficaria pesado). Em vez disso, um comando `get_usage_report(range, model?)` calculado sob
  demanda quando o painel abre/filtra, devolvendo séries por dia×modelo, totais, top
  sessões, top projetos e as N últimas requisições.
- `LogScanner` passa a guardar `UsageEvent` por arquivo (hoje guarda `Entry` enxuta). Mantém o
  cache por mtime/tamanho e a deduplicação por `message.id:requestId`. O widget continua
  recebendo só o agregado diário de 30 dias, sem mudança de contrato.

### 6.3 Gráficos

Poucos tipos (área/barras empilhadas por dia, barras horizontais de ranking, tabela).
Recomendação inicial: **SVG feito à mão**, coerente com o resto do app (sem dependência
de gráfico hoje, `ActivityGrid` já é desenhado na unha). Biblioteca só se o escopo crescer.
Decidir amanhã.

### 6.4 Fatias (da menor pra maior)

1. **MVP:** janela + comando + gráfico por dia/modelo + filtros período/modelo + cartões
   Entrada/Saída/Cache por modelo.
2. Tabela de requisições recentes + exportar CSV.
3. Rankings de sessões e projetos.
4. Valor estimado (R$/US$) com tabela de preços editável.
5. Marcos de limite (`quotaLimits`), abas Visão geral/Registros, polimento.

## 7. Riscos e atenções

1. **Deadlock ao criar janela.** Investigado hoje: `WebviewWindowBuilder::build()` dentro de
   handler síncrono (evento da bandeja/menu) trava a thread principal no Windows e deixa o
   cursor de "carregando" infinito. O painel será aberto justamente por menu/clique, então
   **tem que usar o mesmo padrão da correção** (`fix/hover-deadlock`: criar em
   `std::thread::spawn`/comando assíncrono). Mergear essa correção **antes** do painel.
2. **Tokens com cache inflam o total.** `tokens` atual = entrada + saída + cache lido + cache
   criado. Nas capturas de referência, entrada e saída aparecem separadas do cache. O painel
   deve mostrar as quatro partes e **não** apresentar o total misturado como "consumo".
   Decidir: o gráfico principal usa entrada+saída (como a referência) e cache vira coluna à parte.
3. **Preço.** O valor em R$ depende de tabela por modelo (entrada/saída/cache) e de câmbio.
   Não inventar números: levantar os preços oficiais atuais na hora (skill `claude-api`) e
   deixar a tabela em arquivo/config editável, rotulando como "estimativa a preço de API"
   (quem usa assinatura não paga isso — a própria referência diz "quanto esse uso custaria a
   preço avulso").
4. **Janela de 30 dias.** `DAYS = 30` e o corte por mtime estão fixos; 7/14 saem de graça,
   mas qualquer coisa acima de 30 exige mexer nisso e no custo de leitura.
5. **Desempenho.** 506 MB de log: a leitura hoje é arquivo inteiro e só relê o que mudou. Ok
   para o widget; para o painel, medir o tempo da primeira abertura antes de otimizar.
6. **Subagentes (`isSidechain`).** Contam tokens reais; decidir se entram no total (hoje entram).
7. **Privacidade.** Ranking de projetos mostra caminhos/nomes de pasta e sessões; fica só local,
   sem telemetria (README promete isso). CSV exportado contém esses nomes.
8. **Duas instâncias.** Sem guarda de instância única, autostart + abrir manualmente sobe dois
   Pacers (dois ícones, mesmo `EBWebView`). Não bloqueia o painel, mas fica pior com mais
   uma janela. Candidato a PR próprio (`tauri-plugin-single-instance`).

## 8. Perguntas em aberto (decidir amanhã)

1. Janela de amanhã: painel, Etapa B, ou os dois (e em que ordem)?
2. Tema: o painel segue a **cor de destaque** do usuário ou tem paleta fixa por modelo como a
   referência (uma cor por tier)? Sugestão: cores por modelo fixas + destaque nos controles.
3. Moeda do valor estimado: R$, US$ ou ambos? Câmbio fixo configurável ou buscado?
4. Entrada visível: só menu da bandeja, ou botão no widget também?
5. Painel vale também para os provedores da Etapa B desde já (filtro de provedor) ou só Claude no começo?
6. Gráfico: SVG próprio ou biblioteca?

## 9. Sugestão de roteiro para 3 h (se o painel for o foco)

| Tempo | Bloco |
|---|---|
| 0:00–0:15 | Fechar pendências: commitar/mergear `fix/hover-deadlock`, instalar build novo, confirmar que o cursor de carregando sumiu |
| 0:15–0:35 | Responder as perguntas da seção 8; escrever spec curta a partir deste documento |
| 0:35–1:15 | Backend: `UsageEvent`, parser rico, testes (TDD), comando `get_usage_report` |
| 1:15–2:15 | Janela `panel` + rota + gráfico por dia/modelo + filtros (MVP, fatia 1) |
| 2:15–2:45 | Tabela de requisições + CSV (fatia 2) |
| 2:45–3:00 | Prints para PR (regra do projeto: todo PR leva imagens), docs/README, PR |

Fatias 3–5 ficam para o bloco seguinte. Se a Etapa B tiver que entrar também, ela
começa depois do MVP do painel.
