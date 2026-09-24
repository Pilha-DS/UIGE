# Workflow obrigatório para IA

## Antes de alterar

1. Ler `README.md` e `AGENTS.md`.
2. Ler o mapa de leitura em `.cursor/rules/00-project.mdc` e abrir os arquivos do escopo.
3. Verificar padrões em `docs/standards/` e exemplos em `/patterns/`.
4. Verificar ADRs relevantes em `docs/decisions/`.
5. Definir exatamente o escopo da mudança.

### Atalho por tipo de tarefa

| Tarefa | Arquivos mínimos |
|---|---|
| Funcionalidade de produto | `docs/product/features.md`, `docs/product/flows.md` |
| Termo / conceito | `docs/domain/glossary.md` |
| Arquitetura / relações | `docs/architecture/overview.md` + modelo afetado |
| Stack / libs / OS | `docs/architecture/technology.md`, ADR `0001` |
| UI / componente | `docs/standards/ui.md`, `patterns/ui/` |
| Manifest | `docs/architecture/manifest-model.md`, `patterns/manifests/` |
| Execução | `docs/architecture/execution-model.md`, `docs/standards/security.md` |
| Código / crates | `docs/standards/code.md`, `docs/architecture/technology.md` |
| `.github/` | `.cursor/rules/40-github.mdc` |

## Durante

1. Alterar somente arquivos necessários.
2. Reutilizar padrões existentes.
3. Evitar duplicar lógica ou documentação.
4. Não criar tecnologia/abstração não pedida.
5. Manter nomes canônicos.

## Depois

1. Rodar testes/linters/checks aplicáveis.
2. Atualizar documentação afetada.
3. Criar novo registro em `/changes/` quando relevante.
4. Informar o que mudou, por quê e como foi validado.
