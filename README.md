# Pacer

**Português** · [English](README.en.md)

Widget para Windows que mostra quanto do seu plano de IA você já usou — e
**para onde o ritmo atual te leva** antes da redefinição.

![Pacer](docs/screenshots/pacer-main.png)

## O que mostra

- **Sessão (5h) e semanal** do Claude, com a % real vinda da Anthropic
- **Previsão do ritmo**: a marca branca na barra mostra onde você vai estar
  na redefinição; se for estourar, aparece "Limite em 1d 7h"
- **Atividade dos últimos 30 dias** em grade estilo GitHub, a partir dos logs
  locais do Claude Code
- **Alertas** do Windows ao passar de cada limiar (padrão 80% e 95%) e quando
  a previsão indicar que o limite acaba antes da redefinição
- Ícone na bandeja que muda de cor conforme o uso

![Configurações](docs/screenshots/pacer-settings.png)

## Requisitos

- Windows 10/11
- [Claude Code](https://claude.com/claude-code) instalado e logado — é de onde
  vêm o login e os logs

## Instalação

Baixe o instalador em [Releases](https://github.com/sthevan027/Claude-Glass/releases)
e execute. O executável é assinado com um certificado interno (Virex); o
Windows pode mostrar "Windows protegeu o computador" → **Mais informações →
Executar assim mesmo**.

### Rodar do código

Requer Rust, Bun e o Visual Studio Build Tools (C++).

```bash
bun install
bun run tauri dev     # desenvolvimento
bun run tauri build   # instalador em src-tauri/target/release/bundle/nsis/
```

Testes: `cargo test --manifest-path src-tauri/Cargo.toml` e `bun run test`.

## Privacidade

- O Pacer **lê** `~/.claude/.credentials.json` (o login do Claude Code) e
  **nunca escreve nele nem renova o token** — quem renova é o próprio Claude Code.
- O token nunca chega na interface; fica só no processo Rust.
- Só conversa com `api.anthropic.com`. Sem telemetria.
- Configuração em `%APPDATA%\dev.sthevan.pacer\config.json`.

## Créditos

Estilo inspirado no [ai-usagebar](https://github.com/akitaonrails/ai-usagebar)
de Fabio Akita (MIT), de onde vem também o símbolo do Claude usado no app.

## Licença

MIT
