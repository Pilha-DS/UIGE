# Mudança

Criação de perfil passa a usar modal (padrão de UI), removendo o formulário inline da home.

# Arquivos

- `software/crates/ui/ui/components/modal.slint`
- `software/crates/ui/ui/components/create-profile-dialog.slint`
- `software/crates/ui/ui/app.slint`
- `software/crates/app/src/main.rs`
- `patterns/ui/modal.md`
- `patterns/ui/README.md`
- `docs/standards/ui.md`
- `changes/change_v10.md`

# Motivo

Manter a home focada em execução rápida e listagem; criação de perfil é uma tarefa pontual que cabe em modal.

# Impacto

Primeiro padrão concreto de Modal registrado. Fluxos seguintes de diálogo devem reutilizar `components/modal.slint`.
