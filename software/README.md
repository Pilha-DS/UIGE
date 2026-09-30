# Software (código da aplicação)

Workspace Cargo do UIGE. Documentação do produto e arquitetura permanece em `/docs`.

> Este código é uma **POC utilizável**: funciona e é usável, mas está incompleto e **pode conter erros**. Ver [Estado do projeto](../README.md#estado-do-projeto).

## Layout

```text
software/
├── Cargo.toml          # workspace
└── crates/
    ├── core/           # uige-core — domínio, SQLite, execução
    ├── ui/             # uige-ui — apresentação Slint
    └── app/            # uige — binário que liga UI + core
```

## Requisitos

- Rust estável (edição 2021)
- Linux (alvo inicial; ver `docs/architecture/technology.md`)
- Dependências de build do Slint no sistema (ex.: libxcb, libpango — conforme documentação do Slint)

## Comandos

Na pasta `software/`:

```bash
cargo build
cargo run -p uige
cargo test -p uige-core
```

## Fase atual

Fase 4 (evolução): item 18 entregue (descoberta e importação de manifests) e item 16 entregue — modelo, motor, persistência (schema v4), histórico por etapa e a UI de montagem/execução de Workflows.

A pasta padrão de descoberta é `manifests/` dentro da raiz de dados do app — a mesma do banco (`~/.local/share/uige/` no Linux).

Próximo passo da Fase 4: validações avançadas entre parâmetros (ver `docs/product/implementation-order.md`).
