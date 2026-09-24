# Tecnologia

Fonte canônica da stack adotada. A decisão e o histórico estão em [`../decisions/0001-stack-tecnologica.md`](../decisions/0001-stack-tecnologica.md).

## Stack

| Camada | Escolha | Papel |
|---|---|---|
| Core | Rust | Domínio, manifests, execução e persistência |
| UI | Slint | Apresentação e interação |
| Async / processos | Tokio | I/O assíncrono e orquestração de processos |
| Persistência | SQLite | Estado local do aplicativo |
| Manifests / configuração | Serde + JSON | Serialização e formatos versionáveis |
| Design | GNOME HIG (inspiração) | Clareza, densidade moderada, ações destrutivas explícitas |

## Camadas

### Core (Rust)

Concentra regras de domínio, validação de manifests, montagem de execução e acesso a dados. Não deve depender de widgets ou detalhes de layout da UI.

### UI (Slint)

Camada de apresentação. Consome o core; não deve duplicar regras de negócio nem montar comandos de shell por conta própria.

### Async e processos (Tokio)

Usado para I/O assíncrono e para orquestrar a execução de ferramentas externas com argumentos estruturados.

### Persistência (SQLite)

Armazena estado local: Workspaces, Profiles, histórico, preferências e metadados necessários ao uso cotidiano.

### Manifests e configuração (Serde + JSON)

Manifests e configurações serializáveis usam JSON via Serde. Mudanças de schema devem considerar versionamento/migração.

## Design

A interface deve se inspirar no [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/), sem copiá-lo como especificação completa. Padrões concretos de UI ficam em `docs/standards/ui.md` e `/patterns/ui/`.

## Plataformas

| Plataforma | Status |
|---|---|
| Linux | Alvo inicial |
| Windows | Fora do escopo inicial |
| macOS | Fora do escopo inicial |

No Linux, atalhos externos podem usar arquivos `.desktop`, alinhados ao modelo de Launcher do produto.

Suporte a outros sistemas operacionais exige decisão explícita (novo ADR ou atualização deste documento via ADR).

## Fora do escopo atual

- scaffold de aplicação ou `Cargo.toml`;
- escolha de crates secundárias sem necessidade concreta;
- suporte documentado a Windows ou macOS.
