# Mudança

Perfis passam a exigir nome único no Workspace: não é mais possível criar dois perfis com o mesmo nome.

# Arquivos

- `software/crates/core/src/store.rs`
- `docs/product/features.md`
- `changes/change_v12.md`

# Motivo

`Profile::new_id` gera o id a partir de nome + timestamp, então salvar duas vezes o mesmo nome criava duas entidades distintas e a lista de perfis ficava ambígua.

# Impacto

`Store::save_profile` agora valida o nome antes de gravar e retorna `StoreError::ProfileNameTaken` (ou `StoreError::EmptyProfileName` quando vazio). Atualizar o próprio perfil continua permitido. A UI existente já exibe o erro no modal de criação, que permanece aberto para correção.
