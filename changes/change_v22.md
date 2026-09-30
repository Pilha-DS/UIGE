# Mudança

Workspaces de verdade e Ferramentas completas. O id fixo `"default"`, repetido em dez pontos, foi substituído pelo Workspace ativo, e a tela de Ferramentas ganhou busca, filtro, favoritos e curadoria de participação no Workspace.

# Arquivos

Core:

- `software/crates/core/src/workspace/mod.rs`
- `software/crates/core/src/store.rs`
- `software/crates/core/src/db.rs`
- `software/crates/core/src/lib.rs`
- `software/crates/core/migrations/005_workspaces.sql`

UI:

- `software/crates/ui/ui/screens/tools.slint`
- `software/crates/ui/ui/screens/settings.slint`
- `software/crates/ui/ui/app.slint`
- `software/crates/ui/ui/components/light-controls.slint` (`LightToggleButton`)
- `software/crates/ui/ui/components/create-workspace-dialog.slint`

App:

- `software/crates/app/src/main.rs`

Docs:

- `docs/architecture/domain-model.md`
- `docs/product/interface.md`
- `changes/change_v22.md`

# Motivo

Duas lacunas apontadas na entrega anterior:

1. o seletor de Workspace não existia porque só havia o `default` e o app usava esse id fixo em dez pontos;
2. Ferramentas não tinha busca, filtro nem favoritos, então a lista não escalava e não havia como dizer o que pertence a qual contexto.

O filtro Por Workspace / Todas depende das referências de ferramentas do Workspace, por isso os Workspaces vieram primeiro.

# Impacto

## Workspaces

Novo em `Store`: `list_workspaces`, `active_workspace`, `set_active_workspace`, `save_workspace` (criar e renomear), `delete_workspace`, `add_tool_to_workspace`, `remove_tool_from_workspace`, `set_tool_favorite`. A migração 005 adiciona `workspace_favorites`, o índice único de nome e a chave `active_workspace` em `app_meta`.

O Workspace ativo é lido do banco por `active_workspace_id(store)`. Optei por consultar em vez de manter o id em memória: uma cópia paralela do id seria mais um lugar para ficar desatualizado.

Decisões de comportamento:

- o Workspace **Padrão** é protegido — `delete_workspace` recusa, e a UI desabilita o botão em vez de oferecer algo que será recusado;
- um Workspace novo nasce com as ferramentas disponíveis. Sem isso, criar um Workspace mostraria uma lista vazia e o filtro pareceria quebrado. A curadoria é feita removendo depois;
- remover uma ferramenta do Workspace também a retira dos favoritos: favorito de ferramenta fora do contexto não significa nada;
- remover o Workspace ativo volta para o padrão;
- `active_workspace` órfão (apontando para Workspace inexistente) cai no padrão em vez de impedir o app de abrir.

## Unicidade de nome passou a considerar acentos

A checagem anterior usava `COLLATE NOCASE`, que **só dobra ASCII**. `Ação` e `AÇÃO` passavam como nomes diferentes — um furo real num produto em português. A comparação agora é feita em Rust com `to_lowercase` (helper `names_match`), usada por Workspace **e** por Profile. O índice único no banco continua existindo como rede de segurança, mas quem decide é o Rust, porque o índice tem a mesma limitação ASCII.

Isto **muda** o comportamento de Profile aceito anteriormente: nomes que antes passavam agora são recusados. É uma correção, não uma regressão.

## Ferramentas

- **Busca** por nome ou id, e alternador **Do Workspace / Todas**.
- **Favoritos**: botão no detalhe; a lista marca com `★`.
- **Participação no Workspace**: botão no detalhe adiciona/remove, com texto explícito quando a ferramenta está fora.

Consequência estrutural: o índice da lista deixou de coincidir com a posição na biblioteca. Antes `store.tools()[index]` era o mapeamento; com busca e filtro ativos, não é. Introduzi `VisibleTools` (`Rc<RefCell<Vec<String>>>`) como fonte única da ordem exibida, e `selected_tool_id(ui, visible)` para converter índice em id. Os diálogos de perfil e workflow continuam usando a lista completa da biblioteca, porque têm combos próprios e não filtrados.

Em `apply_selected_profile` a ferramenta do perfil pode estar escondida pela busca ou pelo filtro. Limpo os dois e uso o filtro "Todas" antes de selecionar, em vez de tentar selecionar algo invisível.

## UI

`LightToggleButton` (novo) para escolhas de dois estados mutuamente exclusivas, onde um ComboBox esconderia a alternativa. `CreateWorkspaceDialog` coleta apenas nome e workdir; a curadoria de ferramentas fica em Ferramentas, para o diálogo não virar formulário longo.

Configurações ganhou a seção Workspace (renomear, remover). O Workdir global continua **só** na barra de contexto — `docs/product/interface.md` foi ajustado, porque a versão anterior previa o campo nos dois lugares e isso criaria dois editores para o mesmo valor.

# Validação

- `cargo build` — compila.
- `cargo clippy --all-targets` — sem avisos.
- `cargo test` — 47 testes passam (43 no core, 2 + 2 nas integrações), com 11 testes novos de Workspace e 1 do caso de acento em Profile.
- Migração 005 verificada no teste de `init_database`, que agora confere a versão 5, a tabela `workspace_favorites` e o índice único.
- Execução do binário — abre e permanece estável; captura das telas Home, Ferramentas e Configurações.

Erros encontrados e corrigidos durante a implementação:

- a migração 005 existia mas não estava registrada no `db.rs` — os `if schema_version(...)` são explícitos, então o arquivo sozinho não basta. Nove testes falharam com `no such table: workspace_favorites`;
- `list_workspaces` não compilava: temporários na expressão final de um bloco são liberados depois das variáveis locais, e as linhas do `query_map` emprestam o `stmt`. Resolvido coletando antes do fim do bloco;
- `manifests-dir-text` voltou a ser `in` por descuido numa reescrita e quebrou o vínculo `<=>` vindo do `app.slint`;
- seis handlers de UI precisaram de `let visible = Rc::clone(&visible);` antes do `move`, senão o primeiro fechamento consumia o `Rc`.

**Não validado:** o resultado visual. As capturas foram feitas, mas a conferência de espaçamento, alinhamento e proporção das áreas continua sem julgamento confiável, e os cliques de navegação não foram confirmados quadro a quadro. O ajuste fino do layout segue em aberto.
