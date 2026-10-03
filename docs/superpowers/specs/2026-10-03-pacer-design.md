# Pacer — especificação de design (etapa A)

- **Data:** 2026-10-03
- **Branch:** `pacer` (no repositório `Claude-Glass`, que será renomeado para `pacer`)
- **Versão alvo:** 2.0.0 (as tags v1.x já pertencem ao Claude Glass)
- **Referência de estilo:** [akitaonrails/ai-usagebar](https://github.com/akitaonrails/ai-usagebar)
- **Mockup aprovado:** visual v3 (tela principal + configurações), gerado no brainstorming

## 1. Objetivo

Recriar o Claude Glass do zero como **Pacer**: um widget flutuante para Windows
que mostra quanto dos planos de IA você já usou, **para onde o ritmo atual
leva** (projeção até a redefinição) e a atividade dos últimos 30 dias.

Público: uso pessoal diário do Sthevan + projeto público no GitHub/portfólio.
Por isso: README pt-BR/en com prints, release assinada; sem auto-update,
instalador MSI ou i18n por enquanto. Interface em pt-BR.

### Etapas

- **Etapa A (esta spec):** app completo, só com o provedor Claude, e a
  arquitetura pronta para vários provedores.
- **Etapa B (spec futura):** outros provedores, um por vez.

### Fora de escopo nesta etapa

- O pet de pixel art (fica no histórico do git, pode voltar depois).
- Histórico além de 30 dias (os logs do Claude Code só guardam 30 dias).
- Lista de tokens por modelo (a barra "Opus · semanal" da API cobre o essencial).
- Auto-update, MSI, tradução, outros provedores.
- Login OAuth próprio.

## 2. Stack

- **Tauri 2**: backend em Rust, janela via WebView2.
- **Frontend:** React + TypeScript + Vite.
- **Gerenciador JS:** Bun (`bun.lock`; o `package-lock.json` do Electron sai).
- **Plugins/crates:** `tauri-plugin-autostart`, `tauri-plugin-notification`,
  `window-vibrancy` (acrílico do Windows 11), `reqwest` (rustls), `serde`,
  `chrono`, `tokio`.
- **Testes:** `cargo test` (+ `insta` para snapshots), Vitest no frontend.
- **Pré-requisito do PC:** Visual Studio 2022 Build Tools com o workload C++
  (não está instalado desde a formatação de 02/10/2026).

## 3. Arquitetura

Princípio: **o Rust faz o trabalho, o React só desenha.** Token e arquivos
nunca chegam no WebView.

```
src-tauri/src/
  main.rs / lib.rs     setup do app, registro de comandos e plugins
  snapshot.rs          tipos comuns (Snapshot, UsageWindow, Pace, DayActivity)
  providers/
    mod.rs             trait Provider { id(), fetch() -> Snapshot }
    claude/
      creds.rs         lê ~/.claude/.credentials.json (somente leitura)
      api.rs           GET /api/oauth/usage → janelas
      logs.rs          lê ~/.claude/projects/**/*.jsonl → atividade 30 dias
  pacing.rs            projeção do ritmo até a redefinição
  scheduler.rs         ciclos de busca, backoff, cache do último Snapshot
  window.rs            posição, trava acima da taskbar, acrílico, esconder
  tray.rs              ícone colorido por severidade + menu
  alerts.rs            notificações por limiar e por previsão estourando
  config.rs            leitura/gravação segura da configuração
src/
  App.tsx              alterna tela principal ↔ configurações
  components/          Header, ProviderTabs, ProviderCard, UsageBar,
                       ActivityGrid, Footer, Settings/*, Toggle,
                       Segmented, ThresholdBar, StatusDot
  icons/               SVGs de traço (estilo Lucide) + logo do Claude
  lib/types.ts         espelho dos tipos do Rust
  lib/format.ts        "1d 7h", "4,2M", "há 3 min"
  lib/grid.ts          monta a grade 5×7 a partir dos 30 dias
```

### Tipo comum

```rust
struct Snapshot {
    provider: String,          // "claude"
    name: String,              // "Claude"
    plan: Option<String>,      // "Pro", "Max 5x"...
    windows: Vec<UsageWindow>,
    activity: Vec<DayActivity>, // 30 itens, mais antigo → hoje
    today_tokens: u64,
    fetched_at: DateTime<Utc>,
    stale: bool,
    notice: Option<Notice>,    // aviso a exibir (sem login, token vencido, 429...)
}
struct UsageWindow { id: String, label: String, used_pct: f64,
                     resets_at: Option<DateTime<Utc>>, pace: Option<Pace> }
struct Pace { projected_pct: f64, limit_at: Option<DateTime<Utc>> }
struct DayActivity { date: NaiveDate, tokens: u64, messages: u32 }
```

### Comunicação

- `invoke("get_state")` → último `Snapshot` de cada provedor + config.
- Evento `snapshot` (Rust → React) a cada mudança.
- Comandos: `save_config`, `refresh_now`, `hide_window`, `open_url`.

## 4. Fluxo de dados (Claude)

### Credenciais
- Lê `claudeAiOauth.{accessToken, expiresAt, subscriptionType, rateLimitTier}`.
- **Nunca renova o token nem escreve no arquivo**: o refresh token é rotativo,
  e renovar por fora poderia deslogar o Claude Code. Token vencido →
  aviso "abra o Claude Code para renovar".
- Plano exibido derivado de `subscriptionType`/`rateLimitTier`.

### API
- `GET https://api.anthropic.com/api/oauth/usage`, `Authorization: Bearer`,
  header `anthropic-beta: oauth-2025-04-20`.
- Campos usados: `five_hour`, `seven_day`, `seven_day_opus`,
  `seven_day_sonnet` → `{ utilization, resets_at }`. Ausentes são omitidos.
- Intervalo configurável 1/5/10 min (padrão 5), mínimo 60 s entre chamadas.

### Projeção (pacing)
Para uma janela de duração `L` que redefine em `R`, no instante `t`:
`decorrido = L − (R − t)`. Se `decorrido < 5% de L` → sem projeção.
`ritmo = usado / decorrido`; `projetado = usado × L / decorrido`.
Se `projetado > 100` → `limit_at = t + (100 − usado) / ritmo`.
Durações: sessão 5 h, semanais 7 dias.

### Logs
- Varre `~/.claude/projects/**/*.jsonl` (respeita `CLAUDE_CONFIG_DIR`)
  modificados nos últimos 31 dias.
- Considera entradas `type == "assistant"` com `message.usage`, ignora
  modelo `<synthetic>`, deduplica por `message.id:requestId`.
- Tokens = input + output + cache_read + cache_creation.
- Agrega por dia local: tokens e nº de mensagens; total de hoje.
- Cache por arquivo (mtime + tamanho); ciclo a cada ~10 s.

## 5. Interface (mockup v3)

Tema escuro, um único azul de destaque (`#1f6feb`), ícones de traço,
fonte Segoe UI Variable, sem emojis, sem brilhos/degradês decorativos.

### Tela principal (~300 px de largura)
1. Cabeçalho: "Pacer" + subtítulo, botões atualizar e configurações.
2. Abas de provedor com a maior % (só Claude na etapa A).
3. Card do provedor: logo, nome, selo do plano; uma barra por janela
   com "% usado", "Redefine em …" e marca branca da projeção.
   Cores: azul < 75%, laranja 75–90%, vermelho > 90% ou projeção > 100%
   (aí aparece "Limite em 1d 7h" com ícone de tendência).
4. Card "Atividade · 30 dias": grade estilo GitHub (colunas = semanas,
   linhas = dias seg→dom), 5 níveis de azul, tooltip
   "12 set · 4,2M tokens · 87 msgs"; "Hoje: X" e "30 dias: Y".
5. Rodapé: "Atualizado há X" · "Próxima em Y".

Animações: barras enchem (~0,9 s, ease-out) e a marca aparece depois;
quadradinhos surgem em onda. Respeitar `prefers-reduced-motion`.

### Configurações (abre no lugar do conteúdo)
Cartões com hover sutil:
- **Geral:** iniciar com o Windows, travar acima da barra, atualizar a cada
  (seletor deslizante 1m/5m/10m).
- **Alertas:** barra azul→vermelho com dois pontos arrastáveis (padrão
  80% e 95%) e prévia da notificação; "avisar se a previsão estourar".
- **Provedores:** card do Claude com bolinha verde pulsando quando
  conectado; "+ Adicionar provedor (em breve)".
- Rodapé: versão e link do GitHub.

Interruptores com efeito mola; seletor com pílula deslizante.

### Estados especiais
- **Sem credenciais:** card com instrução para logar no Claude Code; mostra
  só atividade/tokens.
- **Desatualizado/erro:** mantém o último dado com opacidade reduzida e aviso
  laranja com a idade do dado e a próxima tentativa.

### Janela e bandeja
- Sem borda, transparente + acrílico, sempre por cima, fora da taskbar.
- Ancorada no canto inferior direito acima da taskbar; arrastável pela área
  do cabeçalho; opção de travar; nunca fica sob a taskbar.
- Bandeja: ícone colorido pela maior severidade; clique esquerdo
  mostra/esconde; menu direito: Atualizar agora, Configurações, Sair.
- Fechar = esconder.

## 6. Configuração

`%APPDATA%\dev.sthevan.pacer\config.json` (identificador neutro, estável
mesmo se o nome mudar):

```json
{
  "startWithWindows": true,
  "lockPosition": false,
  "refreshMinutes": 5,
  "alerts": { "enabled": true, "thresholds": [80, 95], "pace": true },
  "providers": { "claude": { "enabled": true } }
}
```

Gravação atômica (arquivo temporário + rename). Arquivo inválido → padrões e
o original é renomeado para `config.json.bak`.

## 7. Erros

| Situação | Comportamento |
|---|---|
| Sem credenciais | Só logs; aviso "faça login no Claude Code" |
| 401 / token vencido | Mantém último dado; aviso "abra o Claude Code para renovar" |
| 429 | Backoff dobrando até 30 min; `stale = true`; aviso laranja |
| Rede/timeout | Igual ao 429, tenta no próximo ciclo |
| Linha de log inválida | Ignorada |
| Config corrompida | Padrões + `.bak` |

Alertas: um disparo por limiar enquanto o uso ficar acima; rearma quando cai.

## 8. Testes

- **Rust:** pacing (início de janela, 0%, 100%, projeção > 100%), parser de
  logs com fixtures (duplicadas, linha quebrada, virada de dia, synthetic),
  parsing da resposta da API (completa/mínima), backoff, config (padrões,
  corrompida, gravação atômica), credenciais (ausentes, vencidas).
- **Frontend (Vitest):** `format.ts`, `grid.ts`, cores por severidade.
- **Manual:** `bun run tauri build`; comparar com o mockup v3; testar sem
  credenciais e sem rede; verificar assinatura do `.exe` com `signtool`.

## 9. Repositório e entrega

- Branch `pacer`: remove o código Electron (fica no histórico/`main`),
  adiciona o projeto Tauri na raiz.
- Nome concentrado em `tauri.conf.json` (`productName`), título e README.
- README pt-BR + `README.en.md` com prints reais.
- Release **v2.0.0** com `.exe` assinado com o certificado Virex.
- Depois do merge: renomear o repositório no GitHub para `pacer`.
