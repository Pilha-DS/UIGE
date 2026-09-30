# Modelo de execução

## ExecutionContext

Pode carregar informações específicas da execução:

- workdir;
- environment;
- variables;
- permissions;
- metadata.

## Execution

Regra conceitual:

```text
Execution = Tool + Command + Parameters + ExecutionContext
```

Uma execução deve produzir/registrar ao menos:

- status;
- output;
- exit code;
- dados necessários para diagnóstico.

## Workflow

Workflow é uma sequência ordenada de `WorkflowStep`.

Cada etapa deve indicar:

- Tool;
- Command;
- Parameters;
- Workdir opcional;
- comportamento em caso de erro.

O Workdir da etapa sobrepõe o do contexto base; sem Workdir próprio, a etapa herda o contexto base.

Comportamento em caso de erro:

- `Stop` (padrão): a etapa que falhou fica `Failed` e as seguintes ficam `Skipped`;
- `Continue`: as etapas seguintes continuam sendo executadas.

Falha ao resolver a etapa (Tool ou Command inexistente) é falha da etapa, não invalida o Workflow inteiro.

## Estados

Estados iniciais recomendados:

- Pending;
- Running;
- Success;
- Failed;
- Skipped;
- Cancelled.

A execução é sequencial, na ordem declarada. `Pending`, `Running` e `Cancelled` existem para acompanhamento ao vivo, ainda não implementado.

## Histórico de Workflow

Uma execução de Workflow registra uma linha por etapa — inclusive as `Skipped`, para deixar claro o que não chegou a rodar. O status geral é `Failed` quando alguma etapa falha. O registro é do uso, não da definição: é associado ao Workspace ativo.
