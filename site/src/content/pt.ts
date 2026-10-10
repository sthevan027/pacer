export const pt = {
  nav: { home: "Início", features: "Recursos", about: "Sobre", github: "GitHub", download: "Download" },
  hero: {
    badge: "Widget para Windows",
    titleA: "Veja o fim do seu limite chegando — ",
    titleB: "antes dele chegar.",
    subtitle: "O Pacer mostra no Windows quanto do seu plano do Claude você já usou e prevê quando acaba nesse ritmo.",
    cta: "Baixar agora",
    ctaWindowsOnly: "O Pacer é só para Windows",
    ctaGithub: "Ver no GitHub",
    meta: "Windows 10/11 · Grátis · Código aberto",
    fallbackMeta: "instalador .exe",
    videoLabel: "Vídeo de 15 segundos mostrando o Pacer: o widget com as barras de Sessão e Semanal, a previsão de ritmo, a bandeja e a troca de cor de destaque, com narração em português",
    videoCaption: "● Assista ao Pacer em ação (0:15) · com som",
  },
  widget: {
    session: "Sessão (5h)",
    weekly: "Semanal",
    resets: "reinicia",
    warning: "Nesse ritmo, acaba em 1d 7h.",
    caption: "Exemplo com dados fictícios",
  },
  how: {
    title: "Como funciona",
    cards: [
      { title: "Previsão de ritmo", text: "Uma marca na barra mostra onde você vai estar na redefinição. Se o ritmo estoura o limite, o Pacer avisa quando.", img: "media/pacer-main.png", alt: "Janela do Pacer com barras de Sessão e Semanal e a previsão de ritmo" },
      { title: "Na bandeja, sem abrir nada", text: "O ícone muda de cor com o uso. Passe o mouse para ver Sessão e Semanal.", img: "media/pacer-hover.png", alt: "Popup do Pacer ao passar o mouse no ícone da bandeja" },
      { title: "Alertas", text: "Notificação do Windows em 80% e 95% (configurável) — e quando a previsão indicar estouro.", img: "media/pacer-settings.png", alt: "Configurações do Pacer com os limites de alerta" },
    ],
  },
  activity: {
    title: "Atividade de 30 dias",
    text: "Grade a partir dos logs locais do Claude Code, no estilo do GitHub.",
  },
  accent: {
    title: "Sua cor",
    text: "Troque o destaque — a página inteira segue. Aviso (laranja) e crítico (vermelho) continuam fixos, para o alerta nunca se perder.",
    normal: "Normal",
    warn: "Aviso",
    crit: "Crítico",
  },
  privacy: {
    title: "Privacidade, sem asterisco",
    items: [
      "Lê o login do Claude Code, nunca grava nem renova",
      "O token nunca chega à interface",
      "Só conversa com api.anthropic.com",
      "Sem telemetria",
      "Código aberto para conferir",
    ],
  },
  install: {
    title: "Instalação",
    subtitle: "Três passos, uns dois minutos.",
    steps: [
      { title: "Baixe o instalador", text: "Um único arquivo .exe, de cerca de 2,3 MB." },
      { title: "Rode o instalador", text: "O Windows pode mostrar um aviso antes de abrir — a explicação está logo ao lado." },
      { title: "Abra o Pacer", text: "Ele fica na bandeja do sistema e começa a acompanhar seu uso na hora." },
    ],
    warnTitle: "Sobre o aviso do Windows",
    warnText:
      "O instalador é assinado com um certificado próprio (CN=Sthevan.Dev), não de uma autoridade comercial. Por isso o Windows pode mostrar “O Windows protegeu o computador”. O app não foi bloqueado: o Windows só não reconhece o autor do certificado.",
    smartTitle: "O Windows protegeu o computador",
    smartText: "O Windows SmartScreen impediu a inicialização de um aplicativo não reconhecido.",
    smartMore: "Mais informações",
    smartRun: "Executar assim mesmo",
    smartCancel: "Não executar",
    requirements: "Requisitos",
    reqs: ["Windows 10/11", "Claude Code instalado e logado", "Conexão com api.anthropic.com"],
  },
  news: {
    title: "Novidades",
    subtitle: "As últimas releases, direto do GitHub.",
    all: "Ver todas as releases →",
    // usado só se a API do GitHub falhar
    fallback: [
      { version: "v2.1.0", title: "hover na bandeja e cor de destaque", date: "2026-10-09T12:00:00Z" },
      { version: "v2.0.1", title: "Pacer 2.0.1", date: "2026-10-04T12:00:00Z" },
      { version: "v2.0.0", title: "Pacer 2.0.0", date: "2026-10-03T12:00:00Z" },
    ],
  },
  footer: {
    made: "Feito por Sthevan · MIT",
    disclaimer: "Projeto independente. Não é afiliado nem endossado pela Anthropic. Claude é marca da Anthropic.",
  },
};
