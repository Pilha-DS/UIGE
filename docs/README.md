# Documentação do UIGE

Esta pasta contém a documentação canônica do projeto. Cada assunto possui um local definido para evitar duplicação e mistura de responsabilidades.

## Onde cada informação deve ficar

| Tipo de informação | Local |
|---|---|
| Objetivo, comportamento e funcionalidades | `product/` |
| Termos e significados | `domain/` |
| Estrutura interna e relações técnicas | `architecture/` |
| Convenções obrigatórias | `standards/` |
| Regras para IA/agentes | `ai/` |
| Decisões arquiteturais e justificativas | `decisions/` |
| Documento antigo / referência histórica | `reference/` |
| Exemplos concretos e modelos canônicos | `/patterns/` |
| Histórico incremental de alterações | `/changes/` |

## Regra de separação

Um documento deve responder principalmente a **uma** destas perguntas:

- **Produto:** o que o usuário consegue fazer?
- **Domínio:** o que este termo significa?
- **Arquitetura:** como isso funciona internamente?
- **Padrão:** qual regra toda implementação deve seguir?
- **IA:** como um agente deve trabalhar neste repositório?
- **Decisão:** por que escolhemos esta abordagem?

Se um texto responde a várias perguntas, ele deve ser dividido e conectado por links.

## Ordem de leitura recomendada

1. `product/vision.md`
2. `product/features.md`
3. `domain/glossary.md`
4. `architecture/overview.md`
5. `standards/README.md`
6. `ai/README.md`

## Documento anterior

A especificação que antes estava inteira no `README.md` foi preservada em `reference/current-specification.md` para evitar perda de informação durante a reorganização.
