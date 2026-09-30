# Ordem recomendada de implementação

Guia para avançar **devagar**, uma etapa de cada vez. Não é compromisso de prazo.

Capacidades planejadas (o quê) estão em [`roadmap.md`](roadmap.md). Este arquivo define a **ordem** (quando fazer).

## Como usar

- Concluir uma fase antes de abrir a próxima, salvo bloqueio explícito.
- Preferir fatias verticais pequenas (core + UI mínima) a camadas inteiras sem uso.
- Ao criar componente de UI reutilizável, registrar o padrão em `docs/standards/ui.md` e `/patterns/ui/`.
- Mudanças relevantes → novo `changes/change_vN.md`.

Stack e plataformas: [`../architecture/technology.md`](../architecture/technology.md).

---

## Fase 0 — Fundação

Objetivo: app Linux sobe e o core conversa com a UI.

1. Scaffold Rust + Slint + Tokio (alvo Linux).
2. Layout de crates: `core` / `ui` (ou equivalente) + binário da app.
3. SQLite mínimo (schema inicial + migração).
4. “Hello”: janela Slint + chamada ao core.

**Pronto quando:** a app abre no Linux e o core responde a um comando trivial da UI.

---

## Fase 1 — Manifest + execução (fatia vertical)

Objetivo: carregar um manifest e executar um comando real.

5. Modelo Serde/JSON de Manifest (Command + Parameter).
6. Um manifest de exemplo (`echo`, `ls` ou similar).
7. Montar e executar processo via Tokio (argumentos estruturados).
8. Mostrar status, exit code e output na UI.

**Pronto quando:** o usuário escolhe uma ação do exemplo, executa e vê o resultado.

---

## Fase 2 — Base do produto

Objetivo: Workspace, ferramentas e execução rápida úteis no dia a dia.

9. Workspace padrão + Workdir global.
10. Biblioteca de ferramentas (listar / abrir).
11. Execução rápida (ação + parâmetros).
12. Profile simples (salvar e reabrir uma Execution).

**Pronto quando:** dá para organizar um Workspace, rodar ações e reutilizar um Profile.

---

## Fase 3 — Persistência e histórico

Objetivo: estado local confiável.

13. Persistir Workspaces, referências a Tools e Profiles no SQLite.
14. Versionamento de manifests.
15. Histórico básico de execuções.

**Pronto quando:** fechar e reabrir a app preserva o essencial; manifests têm histórico.

---

## Fase 4 — Evolução

Só depois da base estável. Alinha com a seção Evolução do roadmap.

16. Workflows multi-etapa. Modelo, motor, persistência (schema v4), histórico por etapa e UI de montagem/execução entregues.
17. Launchers `.desktop`.
18. ✅ Descoberta / importação de manifests.
19. Relações e validações avançadas entre parâmetros.

---

## Depois (não começar agora)

Itens da seção Futuro do roadmap e plataformas além de Linux:

- automações por agenda/condição;
- Workspace temporário ao abrir pasta;
- retry / fallback / condições em workflows;
- compartilhamento de outputs entre etapas;
- Windows / macOS (exigem ADR).

---

## Último passo — abertura por comando

`uige` abre a interface principal; `uige <id>` abre a interface já focada na ferramenta indicada (ex.: `uige git`). Não existe modo headless. Decisão: [`../decisions/0002-abertura-da-interface-por-comando.md`](../decisions/0002-abertura-da-interface-por-comando.md).

Como a GUI continua sendo a única camada de apresentação e a orquestração permanece onde está, este item **não tem pré-requisitos** de refatoração.

---

## Próximo passo atual

**Fase 4 em andamento.**

- Item 18 (descoberta / importação de manifests): entregue.
- Item 16 (workflows multi-etapa): fatias 1 (modelo + motor), 2 (persistência + histórico) e 3 (UI de montagem/execução) entregues. Falta o vínculo `Profile → Workflow`, que altera relação canônica e por isso não foi incluído junto da UI.
- Item 19 (validações avançadas entre parâmetros): pendente.

O item 17 (launchers `.desktop`) fica adiado: exige ambiente Linux para ser validado e, se for suportado fora de Linux, exige ADR.

Fases 0–3 entregues: scaffold, execução, Workspace/Profile, histórico e versionamento de manifests.
Fase 4: item 18 entregue; item 16 em andamento.
