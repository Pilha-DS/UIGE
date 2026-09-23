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

## Estados

Estados iniciais recomendados:

- Pending;
- Running;
- Success;
- Failed;
- Skipped;
- Cancelled.
