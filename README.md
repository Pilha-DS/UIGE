# UIGE

**UIGE** é uma interface gráfica para ferramentas originalmente operadas por terminal.

O objetivo é transformar ferramentas como Git, Docker, Nmap, FFmpeg, Curl, Pacman, Systemctl, Maven e Kubectl em experiências gráficas consistentes, sem esconder do sistema a capacidade real dessas ferramentas.

> **Simples para executar, poderoso para configurar.**

> **Prova de conceito (POC) utilizável.** O UIGE está em desenvolvimento ativo: já dá para usar no dia a dia, mas **pode conter erros**, funcionalidades incompletas e mudanças incompatíveis entre versões. Não trate esta versão como a única via para operações destrutivas ou irreversíveis. Ver [Estado do projeto](#estado-do-projeto).

O binário aceita um argumento para abrir a interface já focada em uma ferramenta (`uige git`). Não há modo sem interface. Ver [`docs/decisions/0002-abertura-da-interface-por-comando.md`](docs/decisions/0002-abertura-da-interface-por-comando.md).

## Estado do projeto

Fases 0–3 concluídas; Fase 4 em andamento (descoberta/importação de manifests e Workflows multi-etapa entregues). Código em [`software/`](software/). A documentação em [`docs/`](docs/) continua sendo a fonte canônica de produto e arquitetura.

O projeto é uma **POC utilizável**, não um produto estável: as partes que existem funcionam e são usáveis, mas o conjunto ainda está incompleto e **pode conter erros**. O que a interface promete está descrito em [`docs/product/interface.md`](docs/product/interface.md), incluindo o que ainda está pendente; o que já foi validado está registrado em [`changes/`](changes/). Encare divergências como defeito a relatar, e não como comportamento garantido.

## Conceito central

O usuário trabalha principalmente com:

- Workspaces;
- Ferramentas;
- Ações;
- Perfis;
- Workdirs;
- Execuções.

Conceitos internos como `Manifest`, `ExecutionContext`, relações entre parâmetros e `WorkflowStep` devem permanecer detalhes técnicos sempre que possível.

## Tecnologias

| Camada | Escolha |
|---|---|
| Core | Rust |
| UI | Slint |
| Async / processos | Tokio |
| Persistência | SQLite |
| Manifests / configuração | Serde + JSON |
| Design | inspirado no GNOME HIG |
| Plataformas | Linux (alvo inicial) |

Detalhes e limites em [`docs/architecture/technology.md`](docs/architecture/technology.md). Decisões: [`0001-stack-tecnologica.md`](docs/decisions/0001-stack-tecnologica.md) e [`0002-abertura-da-interface-por-comando.md`](docs/decisions/0002-abertura-da-interface-por-comando.md).

## Código

```bash
cd software
cargo run -p uige
```

Requisitos e layout das crates: [`software/README.md`](software/README.md). Ordem de implementação: [`docs/product/implementation-order.md`](docs/product/implementation-order.md).

## Documentação

A documentação é separada por responsabilidade:

- [`docs/product/`](docs/product/) — o que o produto faz e como deve se comportar;
- [`docs/domain/`](docs/domain/) — termos e conceitos do domínio;
- [`docs/architecture/`](docs/architecture/) — como o sistema é estruturado internamente;
- [`docs/standards/`](docs/standards/) — regras e padrões obrigatórios do projeto;
- [`docs/ai/`](docs/ai/) — regras para agentes e IAs que alteram o projeto;
- [`docs/decisions/`](docs/decisions/) — decisões arquiteturais importantes (ADRs);
- [`patterns/`](patterns/) — exemplos canônicos e reutilizáveis;
- [`changes/`](changes/) — registro incremental de mudanças.

Comece por [`docs/README.md`](docs/README.md).

## Princípios

1. **Local-first**: tudo que for necessário para funcionamento normal deve estar disponível localmente depois da instalação/build prevista pelo projeto.
2. **Separação de responsabilidades**: produto, domínio, arquitetura, padrões e implementação não devem ser misturados no mesmo documento.
3. **Interface simples, motor poderoso**: complexidade interna não deve vazar desnecessariamente para a interface.
4. **Fonte única de verdade**: cada regra deve possuir um lugar canônico.
5. **Compatibilidade antes de conveniência**: mudanças de schema, manifests ou formatos persistidos devem considerar migração/versionamento.
6. **Segurança explícita**: ações destrutivas, privilegiadas ou arriscadas devem ser identificáveis antes da execução.
7. **Sem abstrações prematuras**: não criar camadas, serviços ou padrões sem uma necessidade concreta.

## Contribuição

Antes de alterar o projeto, leia:

- [`AGENTS.md`](AGENTS.md)
- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`docs/standards/`](docs/standards/)

Mudanças relevantes devem ser documentadas em `changes/` sem apagar registros anteriores.

## Licença

Veja [`LICENSE`](LICENSE).
