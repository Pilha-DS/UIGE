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

## Tecnologia

A stack adotada (Rust, Slint, Tokio, SQLite, Serde + JSON) e o suporte a plataformas estão em [`technology.md`](technology.md).

A decisão correspondente é o [`ADR 0001`](../decisions/0001-stack-tecnologica.md). A arquitetura conceitual permanece independente de detalhes de crate; novas escolhas de tecnologia exigem ADR.
