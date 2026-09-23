# Glossário

## Tool
Representa uma ferramenta disponível no UIGE, como Git, Docker ou Nmap.

## Manifest
Definição estruturada que descreve como uma Tool funciona: executável, comandos, parâmetros, validações, relações, riscos e permissões.

## Command
Operação disponibilizada por uma Tool, por exemplo `clone`, `pull` ou `status`.

## Parameter
Entrada de um Command. Pode ser argumento posicional, flag, valor, arquivo, diretório, enumeração etc.

## Workspace
Ambiente de organização do usuário. Agrupa referências para Tools, Profiles, favoritos e preferências, além de possuir um Workdir global.

## Workdir
Diretório usado como contexto de execução.

## Execution
Execução de um único Command de uma Tool com parâmetros e contexto definidos.

## Workflow
Sequência ordenada de etapas de execução.

## WorkflowStep
Uma etapa dentro de um Workflow.

## Profile
Configuração reutilizável de uma Execution ou Workflow.

## Launcher
Forma externa ou rápida de abrir um Profile, por exemplo um atalho no desktop.

## Automation
Mecanismo futuro para disparar um Profile automaticamente.

## Termos visuais x internos

A interface deve preferir termos compreensíveis ao usuário. Exemplo:

| Interno | Visual |
|---|---|
| `ManifestBuilder` | Nova ferramenta / Editar ferramenta |
| `ExecutionProfile` | Perfil |
| `ToolView` | Nome da ferramenta |
| `DesktopLauncher` | Criar atalho |
| `WorkspaceContext` | Workspace / Workdir |
