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

## Interfaces

O UIGE é um produto de **interface gráfica**. O binário `uige` aceita um argumento posicional opcional que escolhe o que a interface abre:

| Invocação | Comportamento |
|---|---|
| `uige` | abre a interface principal |
| `uige <id>` | abre a interface já focada na ferramenta `<id>` |

O argumento apenas define o estado inicial; não existe modo headless e nenhuma ferramenta é executada sem a GUI. A ferramenta é identificada pelo `id` do Manifest. Argumentos iniciados por `-` são reservados para flags; um `id` desconhecido abre a interface principal e informa o usuário. Decisão e consequências: [`../decisions/0002-abertura-da-interface-por-comando.md`](../decisions/0002-abertura-da-interface-por-comando.md).

Sem servidor gráfico o produto não é utilizável; nesse caso a falha deve ser explícita, não um travamento.

## Camadas

### Core (Rust)

Concentra regras de domínio, validação de manifests, montagem de execução, casos de uso e acesso a dados. Não deve depender de widgets ou detalhes de layout da UI.

### UI (Slint)

Camada de apresentação. Consome o core; não deve duplicar regras de negócio nem montar comandos de shell por conta própria. É a única camada de apresentação do produto.

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

## Código da aplicação

O código vive em [`../../software/`](../../software/). Workspace Cargo com crates `uige-core`, `uige-ui` e binário `uige`. Detalhes de build em `software/README.md`.

## Fora do escopo atual

- escolha de crates secundárias sem necessidade concreta;
- suporte documentado a Windows ou macOS.