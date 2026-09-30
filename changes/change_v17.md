# Mudança

Registra que o UIGE também será uma ferramenta CLI, em binário único com a GUI. Nenhuma implementação nesta mudança: decisão, escopo e pré-requisito.

# Arquivos

- `docs/decisions/0002-cli-e-binario-unico.md` (novo)
- `docs/decisions/README.md`
- `docs/README.md`
- `README.md`
- `docs/product/vision.md`
- `docs/product/roadmap.md`
- `docs/product/implementation-order.md`
- `docs/domain/glossary.md`
- `docs/architecture/technology.md`
- `changes/change_v17.md`

# Motivo

O mantenedor definiu que o UIGE também será operável por linha de comando. Nenhum documento previa isso: `vision.md` e `README.md` descreviam o produto apenas como interface gráfica. A arquitetura atual já favorecia a extensão (o core não depende de Slint), mas a decisão precisava ser registrada antes de a fatia 3 de Workflows avançar.

# Impacto

Decisão registrada no ADR 0002:

- **um binário `uige`**: com subcomando opera headless; sem subcomando abre a GUI;
- a CLI opera os **conceitos do UIGE** (Workspace, Profile, Workflow, Manifest), não a sintaxe crua das ferramentas;
- contrato de saída: output completo em stdout, erros em stderr, exit code propagado;
- ambiente sem display deve falhar com mensagem clara, não travar.

Consequências de fronteira, já refletidas em `technology.md`:

- orquestração de casos de uso pertence ao **core**, compartilhada por GUI e CLI;
- formatação e recorte de output (ex.: "últimas 5 linhas" na GUI) ficam no **front** — a CLI precisa do output completo.

Pré-requisito registrado no roadmap e em `implementation-order.md`: hoje a orquestração de um caso de uso completo (resolver → executar → registrar histórico) está em `software/crates/app/src/main.rs`, junto das atualizações de UI. Antes de implementar a CLI, essa orquestração precisa ir para o core, senão a CLI duplicaria regra de domínio.

Escopo preservado: a CLI entra como **último passo**; a fatia 3 (UI de Workflows) continua sendo o próximo trabalho. Nenhum código foi alterado.

Trade-off aceito e documentado: o binário único carrega a stack da GUI mesmo em uso de terminal. Se o peso virar problema concreto, avaliar build headless com feature dedicada.
