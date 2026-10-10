# Pacer — Landing page (spec)

Data: 2026-10-09
Status: **rascunho para aprovação** — nenhum código foi escrito; este documento só prepara a construção.
Relacionado: [`2026-10-10-painel-de-uso-preparacao.md`](2026-10-10-painel-de-uso-preparacao.md) (painel de uso), spec de auto-update do Focusbrew (seção 6).

## 1. Objetivo

Uma página de apresentação do Pacer, em **React**, que faça três coisas:

1. **Explicar em 5 segundos** o que o produto é: widget de Windows que mostra quanto do plano de
   IA (hoje, Claude) já foi usado e **para onde o ritmo atual leva** antes da redefinição.
2. **Mostrar o produto funcionando** com vídeo/GIF curtos (não só prints).
3. **Levar ao download** com um botão que sempre aponta para a versão mais recente.

Sucesso = alguém que nunca ouviu falar do Pacer entende, vê e instala sem sair da página
(ou vai ao GitHub por escolha, não por falta de caminho).

### Público

- Usuários do Claude Code no Windows que estouram limite de sessão/semana e querem previsão.
- Devs que chegam pelo GitHub, LinkedIn ou devlog e querem ver rápido se vale a pena.
- Secundário: quem avalia o Sthevan como dev (código aberto, MIT, Tauri + Rust + React).

### Fora de escopo (desta spec)

- Qualquer mudança no app. As pendências do app que a landing exige estão listadas na seção 10,
  mas são tarefas separadas.
- Cadastro, login, pagamento, backend próprio. É uma página estática.
- Landing/catálogo de apps industriais da Virex (evento separado na agenda, outro produto).
- Versão para macOS/Linux (o app é só Windows hoje).

## 2. Estado atual (verificado em 2026-10-09)

| Fato | Valor |
|---|---|
| Repositório | `github.com/sthevan027/pacer` (público, MIT), 0 estrelas |
| `homepageUrl` do repo | **vazio** (apontar para a landing quando existir) |
| Última release | `v2.1.0` (09/10/2026), asset único `Pacer_2.1.0_x64-setup.exe` (~2,3 MB) |
| Nome do instalador | **inclui a versão** (`Pacer_<versão>_x64-setup.exe`) |
| `.cer` da CA | README promete "publicado junto da release", mas **só a v2.0.1 o tem**; a v2.1.0 não |
| Downloads contados | 0 em todas as releases (`downloadCount` do GitHub) |
| Updater no app | **não existe** (sem `tauri-plugin-updater`, sem `latest.json`) |
| Assinatura | Authenticode autoassinado (`CN=Sthevan.Dev`) → SmartScreen avisa |
| Provedores | só Claude (outros provedores = Etapa B, não decidida) |
| Telas existentes p/ mídia | `docs/screenshots/`: main, settings, appearance, hover, accent-panel, accent-hover |
| Material de divulgação | `D:\Projetos\divulgacao\pacer-2.1.0\` (posts + 2 imagens), devlog no mypage |
| Modo demonstração no app | **não existe** (só fixtures de teste em Rust) |
| Ferramentas de mídia | `ffmpeg 9.0.2` instalado; sem gifski/ImageMagick/ScreenToGif |

Requisitos reais do produto (para a página não prometer demais): Windows 10/11 e Claude Code
instalado e logado. Lê `~/.claude/.credentials.json` sem nunca escrever nem renovar; só conversa
com `api.anthropic.com`; sem telemetria.

## 3. Decisões técnicas (recomendação + alternativa)

| Tema | Recomendação | Alternativa | Por quê |
|---|---|---|---|
| Framework | **Vite + React 19 + TypeScript**, gerenciado com **Bun** | Next.js 15 (como o `mypage`) | Página única estática; Vite é o que o próprio app usa; sem SSR a gerenciar. Next só compensaria se virar um site com várias páginas/blog |
| Onde mora | Pasta **`site/`** dentro do repo `pacer` | Repo separado `pacer-site` | Reaproveita prints, ícone e presets de cor; um PR pode atualizar app + página. Cuidado: `site/` tem `package.json` e `bun.lock` próprios, fora do build do Tauri |
| Hospedagem | **Vercel** (raiz = `site/`), como o `mypage` | GitHub Pages | Já é o fluxo conhecido; preview por PR |
| Estilo | **CSS próprio com variáveis** espelhando as do app (`--accent`, `--text`, `--muted`…) | Tailwind | O app usa CSS puro; a página herda a cara do produto sem dependência nova |
| Dados de versão | **API de releases do GitHub** no cliente + link de fallback fixo | Build-time fetch | Ver seção 5 |
| Idioma | **PT-BR primeiro**, textos num arquivo de strings pronto para EN | PT + EN desde o dia 1 | O repo já tem `README.en.md`; EN fica como fatia posterior |
| Domínio | **A decidir** (subdomínio de `sthevan.dev`, `*.vercel.app` ou domínio próprio) | — | Pergunta aberta |

Identidade visual: **a do produto** (tema escuro, azul `#1f6feb`, texto `#e6edf3`, apoio `#8b949e`),
não a do portfólio (que é sinalização de canteiro de obras, amarelo/preto — outro universo). Rodapé
pode linkar o portfólio do Sthevan.

## 4. Estrutura da página

Regra: **só afirmar o que é verdade hoje.** Funcionalidade futura aparece como "em breve" ou não aparece.

1. **Hero** — título, subtítulo, **botão de download** (seção 5), "Windows 10/11 · grátis · código
   aberto", e ao lado/abaixo o **vídeo principal** (seção 7).
   Rascunho de copy (a afinar com a skill `copywriting`/`humanizer` na construção):
   - Título: *"Veja o fim do seu limite chegando — antes dele chegar."*
   - Subtítulo: *"O Pacer mostra no Windows quanto do seu plano do Claude você já usou e prevê quando acaba nesse ritmo."*
2. **Como funciona (3 cartões)** — cada um com um loop curto:
   - **Previsão de ritmo**: marca na barra mostra onde você vai estar na redefinição; aviso
     "Nesse ritmo, acaba em 1d 7h".
   - **Na bandeja, sem abrir nada**: ícone muda de cor conforme o uso; passar o mouse mostra Sessão
     e Semanal em popup.
   - **Alertas**: notificação do Windows em 80% e 95% (configurável) e quando a previsão indicar estouro.
3. **Atividade de 30 dias** — a grade estilo GitHub a partir dos logs locais do Claude Code.
4. **Sua cor** — mini-demo **interativa**: 5 swatches (Azul, Roxo, Verde, Rosa, Ciano) que trocam a
   cor de destaque de um widget de mentira na página (e do botão). Mostra a feature de 2.1.0 e é
   verdadeira: aviso laranja / crítico vermelho continuam fixos — a demo deve mostrar isso também.
5. **Painel de uso** — *só entra quando o painel existir.* Até lá, ou some, ou vira uma linha em
   "Em breve" (decisão da pessoa responsável pelo painel; ver doc de preparação).
6. **Privacidade** — o ponto de confiança, em lista curta: lê o login do Claude Code mas nunca grava
   nem renova; token nunca chega à interface; só fala com `api.anthropic.com`; sem telemetria;
   código aberto para conferir.
7. **Instalação** — 3 passos + o aviso honesto do SmartScreen (seção 5.3) + requisitos.
8. **Novidades** — últimas 3 releases (título + data + link), vindas da mesma chamada da API.
9. **Rodapé** — GitHub, licença MIT, devlog, "feito por Sthevan" e o aviso de independência abaixo.

**Aviso de marca (obrigatório no rodapé):** *"Projeto independente. Não é afiliado nem endossado pela
Anthropic. Claude é marca da Anthropic."* Não usar o logo/símbolo do Claude como identidade da página
(o app usa um símbolo vindo do projeto ai-usagebar, MIT — isso é decisão do app, não precedente para
o site). Citar "Claude" como nome descritivo é ok.

## 5. Botão de download

### 5.1 Comportamento

- Texto: **"Baixar para Windows"** + linha secundária *"v2.1.0 · 2,3 MB · instalador .exe"* (versão e
  tamanho vêm da API).
- Clique inicia o download direto do instalador (`browser_download_url` do asset `*_x64-setup.exe`).
- Em navegador não-Windows (checar `navigator.userAgent`/`userAgentData`): trocar para
  *"O Pacer é só para Windows"* e oferecer "Ver no GitHub" — não esconder, não falhar mudo.
- Estados: carregando (mostra o botão com link de fallback já funcional), pronto, erro da API
  (continua funcionando via fallback).
- Acessível por teclado, com `rel="noopener"`; sem pop-up, sem formulário de e-mail.

### 5.2 De onde vem a versão (e por que não é um link fixo)

O instalador se chama `Pacer_2.1.0_x64-setup.exe` — o número da versão está no nome. Por isso
`https://github.com/sthevan027/pacer/releases/latest/download/<nome-fixo>.exe` **não funciona** sem
mudar o processo de release. Opções:

| Opção | Como | Prós | Contras |
|---|---|---|---|
| **A (recomendada)** | No cliente, `GET https://api.github.com/repos/sthevan027/pacer/releases/latest`; achar o asset `*_x64-setup.exe`; cachear em `sessionStorage` | Sempre atual, sem redeploy, traz notas/data/tamanho/`download_count` | Limite de 60 req/h por IP sem token (suficiente com cache); depende de JS |
| B | Buscar no build da Vercel + redeploy via webhook a cada release | Funciona sem JS no cliente | Exige webhook e passo extra no release |
| C | Passar a publicar também um asset de nome fixo (`Pacer-setup.exe`) | Link estático trivial | Duplica upload; contraria o fluxo atual |

**Fallback obrigatório em qualquer opção:** o `href` inicial do botão é
`https://github.com/sthevan027/pacer/releases/latest` (página da release), então a página funciona
sem JS e se a API falhar.

### 5.3 SmartScreen e certificado — comunicar com honestidade

O instalador é assinado com certificado **autoassinado** (`CN=Sthevan.Dev`); o Windows pode mostrar
"Windows protegeu o computador". A seção de Instalação deve dizer isso de forma direta:
*Mais informações → Executar assim mesmo*, e explicar que é por o certificado não ser de uma
autoridade comercial. Opcionalmente, instruções para confiar na CA (`Sthevan.Dev-Root-CA.cer`).

> **Pendência real (não é da landing, mas ela depende):** a v2.1.0 foi publicada **sem** o
> `Sthevan.Dev-Root-CA.cer`, embora o README diga que ele vai junto. Ou o release volta a anexá-lo
> (a v2.0.1 anexou), ou o README/landing deixam de prometer. Sugestão: anexar em toda release e a
> landing linkar `releases/latest` para ele.

Extra opcional: publicar o **SHA-256** do instalador nas notas da release e exibi-lo na página.

## 6. Atualizações — relação com o trabalho no Focusbrew

**Situação:** o Focusbrew está desenhando auto-update na branch `feat/auto-update` (spec em
`docs/superpowers/specs/2026-10-09-auto-update-design.md` do repo `focusbrew`): `tauri-plugin-updater` +
`tauri-plugin-process`, chave minisign própria (separada do certificado do Windows), endpoint fixo
`…/releases/latest/download/latest.json`, checagem silenciosa ao abrir, e área "Atualizações" nas
configurações com "Verificar agora" / "Atualizar agora". A própria spec deixa o **Pacer como o
próximo projeto**, "mesma mecânica depois de validada" no Focusbrew.

**O que isso muda para a landing:**

1. **A landing não depende do updater.** Ela funciona só com a API de releases (seção 5). O updater
   resolve "quem já tem o app se mantém atualizado"; a landing resolve "quem ainda não tem, instala".
2. **Quando o Pacer ganhar o updater**, cada release passa a incluir, além do `.exe`, o artefato do
   updater e o `latest.json`. A landing continua achando o instalador pelo mesmo critério
   (`*_x64-setup.exe`), então **a regra de nome do instalador deve ser preservada**.
3. **`latest.json` como fonte alternativa** de versão/notas/data (uma URL estável só). Só vale
   adotar se o acesso por navegador funcionar — o endpoint de download do GitHub redireciona, e
   **não verifiquei se há CORS**; testar antes de planejar em cima disso. Por enquanto: API de releases.
4. **Texto da landing** poderá afirmar "atualiza sozinho" apenas depois que o Pacer tiver o updater
   em produção. Hoje não afirmar.
5. **Custo no processo de release:** a chave privada do updater fica fora do repo (variável de
   ambiente) e o `latest.json` precisa subir com cada release. Vale decidir se esse mesmo release
   passa a anexar também o `.cer` (seção 5.3), já que o passo de upload vai ser revisto de qualquer forma.

## 7. Mídia: vídeo, GIFs e imagens

### 7.1 Inventário

| Peça | Uso | Formato | Meta |
|---|---|---|---|
| **Vídeo hero** (8–12 s, loop) | Topo da página | MP4 H.264 + WebM VP9, mudo, `autoplay loop muted playsinline`, com `poster` | ≤ ~1,5 MB cada |
| 3 loops de feature (3–6 s) | Cartões da seção 2 | MP4/WebM mudos | ≤ ~600 KB cada |
| Loop do painel de uso | Seção 5, quando existir | MP4/WebM | idem |
| **GIFs** (versões dos loops) | README do GitHub, posts, devlog (onde vídeo não toca sozinho) | GIF | ≤ ~3 MB cada |
| Prints estáticos | Fallback sem JS, `prefers-reduced-motion`, OG | PNG/WebP | Já existem em `docs/screenshots/` |
| Imagem de compartilhamento (OG) | Link no LinkedIn/X/WhatsApp | PNG 1200×630 | 1 |
| Ícone/favicon | Aba do navegador | SVG (de `app-icon.svg`) + PNG | — |

**Na página, preferir vídeo mudo em loop a GIF** (muito menor, melhor qualidade). GIF só onde
vídeo não é possível.

### 7.2 Roteiro (storyboard) do vídeo hero

1. Widget no canto superior direito com barras de **Sessão** e **Semanal** em estado "alerta"
   (laranja) e a mensagem *"Nesse ritmo, acaba em 1d 7h"* — o gancho.
2. Cursor vai à bandeja; **popup de hover** aparece com as duas barras.
3. Clique no ícone: o widget vem para a frente.
4. Configurações → **Aparência**: alterna 2–3 cores de destaque; painel, popup e ícone mudam juntos.
5. Fecha no widget estável + logo e botão (a página já tem o botão; o vídeo termina limpo para o loop).

Cada loop de feature recorta um desses momentos.

### 7.3 Como produzir

**Dados de demonstração são necessários.** Gravar com a conta real mostra o seu uso verdadeiro
(porcentagens aleatórias, que raramente chegam ao estado de alerta) e, no painel de uso futuro,
nomes de projetos e sessões. O app **não tem modo demo** → ver pendência 10.1.

Caminhos de gravação (decidir):

| Caminho | Prós | Contras |
|---|---|---|
| **A. Captura real + `ffmpeg`** (já instalado; `gdigrab` recorta uma região da tela) | Fiel ao produto, scriptável e repetível, sem ferramenta nova | Precisa do modo demo e de um roteiro cronometrado; conferir escala de DPI do monitor |
| B. **Remotion** (skill disponível) — animação em código reproduzindo a UI | Visual polido, trocável (cores, textos), não depende de gravar | Reimplementa a UI (risco de divergir do app real); mais trabalho |
| C. Vídeo gerado por IA (Higgsfield etc.) | Rápido | **Não recomendado**: UI inventada engana sobre o produto |

Recomendação: **A** para tudo que mostra o app; **B** só se quiser um intro/outro estilizado.

Receita de ponto de partida (validar antes de usar, ajustar região/escala):

```bash
# gravar uma região (x,y,largura,altura) em 30 fps
ffmpeg -f gdigrab -framerate 30 -offset_x 1500 -offset_y 0 -video_size 420x640 -i desktop \
  -c:v libx264 -crf 18 -pix_fmt yuv420p bruto.mp4
# MP4 final leve + WebM
ffmpeg -i bruto.mp4 -an -vf "scale=720:-2" -crf 26 -movflags +faststart hero.mp4
ffmpeg -i bruto.mp4 -an -vf "scale=720:-2" -c:v libvpx-vp9 -crf 34 -b:v 0 hero.webm
# GIF com paleta própria (sem gifski)
ffmpeg -i hero.mp4 -vf "fps=15,scale=480:-1:flags=lanczos,split[a][b];[a]palettegen[p];[b][p]paletteuse" hero.gif
```

Pontos de atenção: o widget fica **no fundo da área de trabalho** (`alwaysOnBottom`) — gravar com a
área limpa ou com um papel de parede neutro; ocultar a barra de tarefas/ícones pessoais; resolução
de captura com margem para escalar para baixo.

### 7.4 Reuso

Os mesmos arquivos alimentam o README (GIF), os posts de divulgação em
`D:\Projetos\divulgacao\<assunto>\` e o devlog do `mypage`. Guardar os brutos e o script de
conversão para refazer a mídia a cada release.

## 8. Design e experiência

- **Tokens:** copiar do app (`src/styles.css`): `--accent #1f6feb`, `--text #e6edf3`, `--muted
  #8b949e`, `--dim #6e7681`, `--ok #3fb950`, `--warn #d29922`, `--crit #f85149`; fundo escuro
  `#181c22` (o mesmo do ícone). Os 5 presets de cor vêm de `src/lib/accent.ts`.
- **Tipografia:** o app usa Segoe UI Variable; na web usar uma pilha de sistema ou uma fonte
  carregada de forma leve. Definir na construção com a skill de design.
- **Responsivo:** mobile primeiro (16 px de margem lateral, sem rolagem horizontal). No celular o
  botão vira "Só para Windows" + opção de copiar/mandar o link para o PC.
- **Movimento:** respeitar `prefers-reduced-motion` (mostra o `poster`/print, sem autoplay).
- **Acessibilidade:** contraste AA no tema escuro, foco visível, vídeos sem áudio com `aria-label`
  descritivo, ordem de leitura coerente.
- **Tom:** direto, técnico quando precisa, sem enrolação (a voz do Sthevan); evitar jargão de marketing.

## 9. SEO, medição e desempenho

- `<title>`/descrição, Open Graph e Twitter card, `canonical`, `sitemap.xml`, `robots.txt`, dados
  estruturados `SoftwareApplication` (sistema operacional: Windows; preço: 0; licença MIT).
- **Medição:** o app promete "sem telemetria" — a página pode ter análise leve e sem cookies
  (decisão pendente) ou nenhuma. Sem custo e sem rastreio: ler o `download_count` dos assets na
  própria API de releases (hoje todos 0).
- **Metas de desempenho (a conferir com Lighthouse):** LCP < 2,5 s em 4G, JS inicial pequeno
  (sem biblioteca de animação pesada; a demo de cores é CSS + estado), vídeos carregados sem
  bloquear o texto (`poster` + `preload="metadata"`).

## 10. Pendências no app/release que a landing exige (tarefas separadas)

1. **Modo demonstração** (só desenvolvimento, ex.: variável de ambiente `PACER_DEMO=1`) que injeta
   snapshots/atividade/sessões fictícios — necessário para gravar sem dados pessoais e para o painel.
2. **Anexar `Sthevan.Dev-Root-CA.cer` em toda release** (ou corrigir README/landing).
3. **Preencher `homepageUrl`** do repositório com a URL da landing.
4. **Manter o padrão de nome** `Pacer_<versão>_x64-setup.exe` (a landing procura por ele).
5. (Opcional) Publicar SHA-256 nas notas da release.
6. (Depois do updater) Anexar `latest.json` e o artefato do updater (ver seção 6).

## 11. Estrutura proposta do projeto

```
pacer/
└── site/
    ├── index.html
    ├── package.json            # React 19, Vite, TypeScript (bun)
    ├── src/
    │   ├── main.tsx
    │   ├── App.tsx
    │   ├── styles.css          # tokens espelhando o app
    │   ├── content/pt.ts       # todos os textos (pronto para en.ts)
    │   ├── lib/release.ts      # busca/cache da última release + escolha do asset
    │   ├── lib/platform.ts     # detecção de Windows
    │   └── components/         # Hero, DownloadButton, Features, AccentDemo, Privacy, Install, Changelog, Footer
    └── public/
        ├── media/              # hero.mp4/.webm, loops, gifs, posters
        ├── og.png, favicon.svg
        └── robots.txt, sitemap.xml
```

Testes: unitários para `release.ts` (escolha do asset, cache, erro de API, sem asset) e
`platform.ts`; verificação manual do botão em Windows/Android/macOS; Lighthouse; checagem sem JS.

## 12. Plano em fatias

| Fatia | Entrega | Depende de |
|---|---|---|
| **L1** | `site/` com Vite+React, tokens, Hero com **botão de download funcionando** (API + fallback), rodapé com aviso de marca | — |
| **L2** | Seções Como funciona / Privacidade / Instalação (+SmartScreen) / Novidades, com **prints** | L1 |
| **L3** | Demo interativa de cor de destaque | L2 |
| **L4** | Vídeo hero e loops reais (e GIFs para o README) | pendência 10.1 (modo demo) |
| **L5** | SEO/OG, Lighthouse, deploy na Vercel, `homepageUrl`, devlog/posts de lançamento | L1–L4 |
| **L6** | Versão EN; seção do painel de uso; "atualiza sozinho" após o updater | painel / updater |

L1–L3 não dependem de mídia nem de nenhuma mudança no app: dá para ter a página no ar com prints e
o botão funcionando antes dos vídeos.

## 13. Perguntas em aberto

1. **Domínio/URL** da landing (subdomínio de `sthevan.dev`, `*.vercel.app` ou domínio próprio)?
2. `site/` dentro do repo `pacer` ou repo separado?
3. **PT-BR apenas no começo** ou PT + EN já na primeira versão?
4. **Análise de acesso:** nenhuma, ou uma ferramenta leve sem cookies?
5. **Vídeo:** captura real com `ffmpeg` (recomendado), Remotion, ou os dois?
6. A landing menciona **outros provedores** ("em breve") ou fica só Claude até a Etapa B?
7. Assinar a página como **Sthevan**, **Virex**, ou os dois?
8. O `.cer`: anexar à release (recomendado) ou parar de mencioná-lo?
9. Adotar a opção **A** (API de releases) para o botão, ou prefere B/C?

## 14. Critérios de aceite

- O botão baixa o instalador **da versão mais recente** sem edição manual a cada release, e continua
  funcional (leva à página da release) com a API fora do ar e com JavaScript desligado.
- Nada na página afirma funcionalidade que o app ainda não tem; o aviso de independência da Anthropic
  está no rodapé.
- O SmartScreen/certificado é explicado antes do download.
- Mobile 360 px sem rolagem horizontal; `prefers-reduced-motion` respeitado.
- Mídia leve (hero ≤ ~1,5 MB) e mostrando dados fictícios, nunca dados pessoais.
- Lighthouse ≥ 90 em Desempenho, Acessibilidade e SEO (meta a confirmar).
