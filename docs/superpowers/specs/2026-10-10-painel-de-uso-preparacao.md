# Pacer — Painel de uso

Data: 2026-10-09 (preparação) · executado em 2026-10-09/10
Status: **implementado** na branch `feat/painel-de-uso`.

> As seções 1–7 são o estudo original e continuam valendo como contexto. As
> **decisões tomadas** e o que de fato foi entregue estão nas seções 8 e 10;
> a seção 9 (roteiro) ficou como registro do plano.

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

## 8. Decisões tomadas

| # | Pergunta | Decisão | Contra a sugestão da spec? |
|---|---|---|---|
| 1 | Escopo da janela | **Só o painel**; a Etapa B fica para outro bloco | — |
| 2 | Cores | **Tudo com a cor de destaque** do usuário, sem paleta fixa por tier | sim (sugeria cores fixas) |
| 3 | Moeda | **R$ com câmbio fixo configurável** nas Configurações | sim (é a fatia 4, fora do MVP sugerido) |
| 4 | Entrada | Menu da bandeja **e** botão no cabeçalho do widget | sim (a spec deixava em aberto) |
| 5 | Provedores | Só Claude agora, mas o modelo de dados já é independente de provedor | — |
| 6 | Gráfico | **Biblioteca (Recharts)**, não SVG na mão | sim (sugeria SVG) |

Consequência da decisão 2, que precisou de uma saída própria: se todas as famílias usam a mesma
cor, a pilha do gráfico não distingue nada. Resolvido variando a **mistura com o branco**
(`color-mix`), de 100% a 45% do accent — os tons continuam todos sendo a cor de destaque, mas
cada tier tem o seu. Usa o mesmo `color-mix` que o `--accent-2` já usava.

Consequência da decisão 3: entrou `usdBrl` na `Config` (padrão 4,97 — referência do BCB em
06/10/2026 —, editável), e um campo que só grava ao sair do foco, senão o valor salvo reformata
o campo a cada tecla.

Consequência da decisão 6: o Recharts pesa ~367 kB. O painel entra por `import()` dinâmico, então
ele sai num chunk que só a janela do painel carrega — o widget continua com os mesmos 243 kB.

**Não implementado** (fica registrado como pendência): os marcos de limite (`quotaLimits`) da
fatia 5. As abas Visão geral/Registros entraram.

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

## 10. O que foi entregue, e o que a execução ensinou

Todas as fatias 1–4 saíram, mais as abas da fatia 5. Comunicação e comandos:

- `usage/` (novo): `UsageEvent`, `UsageStore`, `UsageReport`, `pricing.rs`.
- `panel.rs` (novo): janela `panel`, criada fora da thread principal (risco 7.1).
- `LogScanner` deixou de viver **dentro** do `ClaudeProvider`: o provedor vive dentro do laço do
  scheduler, então o scanner era inalcançável por um comando Tauri. Foi para o `Shared`, e o
  provedor guarda um `Arc<Mutex<UsageStore>>`. O contrato do widget não mudou.
- Comandos `get_usage_report` e `export_usage_csv`, ambos `async` +
  `spawn_blocking` — são chamados de dentro do WebView e não podem travar a thread principal.

Achados que valem lembrar:

1. **O parser contra os logs reais.** 7.939 eventos nos últimos 30 dias. Os totais conferiram
   exatamente com uma segunda implementação independente (escrita só para conferir), o que
   validou a janela de 30 dias, o fuso local, a deduplicação e a divisão do cache. Vale repetir
   esse truque em mudança de parser: os testes de unidade não pegam erro de recorte de janela.
2. **A escrita de cache de 1 h domina.** Na amostra, 45,4 M contra 3,8 M da de 5 min — 12× maior,
   e 1,6× mais cara por token. Somar as duas numa taxa só (como a spec simplificava) subestimaria
   o valor de forma relevante. O log já traz a divisão; usar é de graça.
3. **Deduplicar por (sessão, timestamp, modelo) é errado.** Foi o primeiro rascunho, e o teste
   pegou: duas requisições diferentes no mesmo segundo e no mesmo modelo viravam uma. A identidade
   é `message.id` + `requestId` — o que o código original já fazia.
4. **A janela abre no topo.** Na primeira abertura o painel aparecia rolado até o fim: enquanto o
   relatório não chega o conteúdo é curto, e o crescimento do gráfico depois deixa o contêiner
   rolado lá embaixo. Corrigido com um scroll pro topo só na primeira carga — o painel se
   atualiza sozinho a cada 10 s e não pode roubar o scroll de quem está lendo.

Sobre o valor em R$, para quem for mexer depois: a tabela de preços é um `const` em `pricing.rs`
com a **fonte e a data** no comentário do módulo, e o casamento é por **prefixo mais longo** —
`claude-opus-5-5` não pode cair no preço do `claude-opus-5`, e `claude-haiku-4-5-20251001`
(id com data) tem que cair no `claude-haiku-4-5`. Preço desatualizado se corrige ali.
