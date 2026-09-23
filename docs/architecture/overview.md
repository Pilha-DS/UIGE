# Visão arquitetural

## Responsabilidades

```text
Manifest   → define
Tool       → disponibiliza
Workspace  → organiza
Profile    → preconfigura
Execution  → executa
Launcher   → facilita acesso
Automation → automatiza
```

## Relações principais

```text
Tool ──uses──> Manifest
Manifest ──contains──> Command[]
Command ──contains──> Parameter[]

Workspace ──references──> Tool[]
Workspace ──references──> Profile[]

Profile ──configures──> Execution | Workflow
Workflow ──contains──> WorkflowStep[]

Launcher ──references──> Profile
Automation ──references──> Profile
```

## Regra de referência

Workspaces, Profiles, Launchers e Automations devem referenciar objetos canônicos em vez de copiar suas definições.

## Dependência de tecnologia

A arquitetura conceitual não deve ser acoplada prematuramente a uma linguagem, framework de UI ou mecanismo de persistência. Escolhas desse tipo devem ser registradas em ADRs em `docs/decisions/`.
