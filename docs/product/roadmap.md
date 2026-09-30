# Roadmap

Este arquivo registra capacidades planejadas, não compromissos de prazo.

A **ordem** recomendada para implementar com calma está em [`implementation-order.md`](implementation-order.md).

## Base

- Workspaces;
- biblioteca de ferramentas;
- execução simples;
- manifests estruturados;
- perfis de execução;
- histórico/versionamento de manifests.

## Evolução

- workflows multi-tool;
- launchers externos;
- descoberta assistida de executáveis;
- importação/exportação de manifests;
- validações e relações avançadas entre parâmetros.

## Futuro

- automações por agenda/condição;
- Workspace temporário ao abrir uma pasta;
- retry/fallback/condições em workflows;
- compartilhamento de outputs entre etapas.

## Último passo

- **Abertura da interface por comando**: `uige` abre a interface principal; `uige <id>` abre a interface já focada na ferramenta indicada. Não é uma CLI: todo uso passa pela GUI. Decisão: [`../decisions/0002-abertura-da-interface-por-comando.md`](../decisions/0002-abertura-da-interface-por-comando.md).

Este item é pequeno (apenas o estado inicial da interface) e serve de base para os launchers externos, que podem apontar direto para uma ferramenta.
