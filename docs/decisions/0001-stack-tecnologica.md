# ADR 0001 — Stack tecnológica

## Status

Aceito

## Contexto

O UIGE precisa de uma stack alinhada a local-first, execução segura de processos e separação entre motor e interface. A arquitetura conceitual já existia, mas a escolha de linguagem, UI, async, persistência e plataformas ainda estava em aberto.

## Decisão

Adotar a seguinte stack:

| Camada | Escolha |
|---|---|
| Core | Rust |
| UI | Slint |
| Async / processos | Tokio |
| Persistência | SQLite |
| Manifests / configuração | Serde + JSON |
| Design | inspirado no GNOME HIG |
| Plataformas | Linux como alvo inicial |

Detalhes operacionais ficam em `docs/architecture/technology.md`.

## Alternativas consideradas

- **Tauri + React/TypeScript:** UI web-based; maior superfície e stack híbrida menos alinhada a um core Rust único com UI nativa.
- **Electron + Node/TypeScript:** ecossistema amplo, porém mais pesado e menos adequado ao princípio local-first enxuto.
- **Flutter / Avalonia / Qt:** viáveis para desktop, mas afastam o core da execução de processos e do ecossistema Rust já escolhido para o motor.

## Consequências

- O domínio, a execução e a persistência ficam no core Rust.
- A UI Slint não deve concentrar regras de domínio.
- Windows e macOS ficam fora do escopo até nova decisão/ADR.
- Crates secundárias (ORM, logger, etc.) só entram quando houver necessidade concreta.
