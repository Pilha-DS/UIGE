# UIGE

**UIGE** é uma interface gráfica para ferramentas originalmente operadas por terminal.

O objetivo é transformar ferramentas como Git, Docker, Nmap, FFmpeg, Curl, Pacman, Systemctl, Maven e Kubectl em experiências gráficas consistentes, sem esconder do sistema a capacidade real dessas ferramentas.

> **Simples para configurar, poderoso para executar.**

## Estado do projeto

O projeto está em fase de especificação e definição de arquitetura. A documentação é a fonte principal de verdade enquanto a implementação é construída.

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

Detalhes e limites em [`docs/architecture/technology.md`](docs/architecture/technology.md). Decisão: [`docs/decisions/0001-stack-tecnologica.md`](docs/decisions/0001-stack-tecnologica.md).

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
