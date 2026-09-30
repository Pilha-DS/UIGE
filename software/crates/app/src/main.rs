//! UIGE application entry point.
//!
//! Wires the Slint UI to the core. Keeps domain logic out of the presentation layer.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use slint::winit_030::{winit, WinitWindowAccessor};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use uige_core::{
    discover_manifests, execute_command, execute_workflow, find_command, list_subdirectories,
    Command, ExecutionContext, ExecutionStatus, Manifest, ManifestCandidate, ParameterType,
    Profile, StepFailurePolicy, Store, Workflow, WorkflowError, WorkflowStep, Workspace,
};
use uige_ui::{AppWindow, PageInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = default_database_path()?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let default_workdir = find_git_root(&std::env::current_dir()?)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let store = Rc::new(RefCell::new(Store::open(&db_path, default_workdir)?));

    // Compartilhado: execução rápida e execução de workflow usam o mesmo runtime.
    let runtime = Rc::new(tokio::runtime::Runtime::new()?);

    // Candidatos da última descoberta, na mesma ordem exibida na UI.
    let discovered: Rc<RefCell<Vec<ManifestCandidate>>> = Rc::new(RefCell::new(Vec::new()));
    // Etapas em construção no diálogo de workflow, ainda não persistidas.
    let workflow_draft: Rc<RefCell<Vec<WorkflowStep>>> = Rc::new(RefCell::new(Vec::new()));
    // Ferramentas exibidas na tela de Ferramentas, na ordem da lista.
    let visible: VisibleTools = Rc::new(RefCell::new(Vec::new()));
    // Favoritas exibidas na Home, na ordem exibida. Índice próprio: a Home mostra
    // um subconjunto das ferramentas, então as posições não coincidem com `visible`.
    let favorites_visible: VisibleTools = Rc::new(RefCell::new(Vec::new()));
    // Pastas exibidas no diálogo de Workdir, na ordem da lista: o rótulo é o que
    // aparece, o caminho é o destino do clique.
    let workdir_entries: Rc<RefCell<Vec<(String, PathBuf)>>> = Rc::new(RefCell::new(Vec::new()));
    // Destino do diálogo de Workdir aberto. O diálogo é o mesmo para o Workdir
    // global e para o de uma ferramenta; quem sabe qual está em jogo é quem o
    // abriu, porque depois de aberto o modal não carrega essa informação.
    let workdir_target = Rc::new(RefCell::new(WorkdirTarget::Global));
    // Ferramenta em foco em cada superfície. São separadas de propósito: a tela
    // da ferramenta e o modal ▶ podem apontar para ferramentas diferentes, e o
    // modal ⋮ age sobre a ferramenta de onde foi aberto.
    let detail_tool_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let run_tool_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let actions_tool_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let ui = AppWindow::new()?;
    ui.set_manifests_dir_text(SharedString::from(
        default_manifests_dir().display().to_string(),
    ));
    bind_workspace(&ui, &store.borrow())?;
    refresh_tools(&ui, &store.borrow(), &visible);
    refresh_home_favorites(&ui, &store.borrow(), &favorites_visible);
    refresh_profiles(&ui, &store.borrow())?;
    refresh_workflows(&ui, &store.borrow(), None)?;
    refresh_history(&ui, &store.borrow())?;
    refresh_workflow_history(&ui, &store.borrow())?;

    let ui_weak = ui.as_weak();

    // ---- Paginação das listas longas.
    // A tela só pede a página; ela entra no estado publicado e o recorte do app
    // ajusta e devolve. Cada lista tem o seu, e uma página não mexe na outra.
    ui.on_tool_filter_changed({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Busca ou filtro novos: abrir no meio do resultado anterior não
                // diria nada sobre o resultado novo.
                let info = ui.get_tool_page_info();
                ui.set_tool_page_info(page_request(info, 0));
                refresh_tools(&ui, &store.borrow(), &visible);
            }
        }
    });

    ui.on_tool_page_requested({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let ui_weak = ui_weak.clone();
        move |page| {
            if let Some(ui) = ui_weak.upgrade() {
                let info = ui.get_tool_page_info();
                ui.set_tool_page_info(page_request(info, page));
                refresh_tools(&ui, &store.borrow(), &visible);
            }
        }
    });

    ui.on_profile_page_requested({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move |page| {
            if let Some(ui) = ui_weak.upgrade() {
                let info = ui.get_profile_page_info();
                ui.set_profile_page_info(page_request(info, page));
                if let Err(err) = refresh_profiles(&ui, &store.borrow()) {
                    ui.set_status_text(err.to_string().into());
                }
            }
        }
    });

    ui.on_history_page_requested({
        let store = Rc::clone(&store);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move |page| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = detail_tool_id.borrow().clone() else {
                return;
            };
            let info = ui.get_history_page_info();
            ui.set_history_page_info(page_request(info, page));
            if let Err(err) = refresh_tool_history(&ui, &store.borrow(), &tool_id) {
                ui.set_status_text(err.to_string().into());
            }
        }
    });

    ui.on_candidate_page_requested({
        let store = Rc::clone(&store);
        let discovered = Rc::clone(&discovered);
        let ui_weak = ui_weak.clone();
        move |page| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let info = ui.get_candidate_page_info();
            ui.set_candidate_page_info(page_request(info, page));
            refresh_discovered_candidates(&ui, &store.borrow(), &discovered.borrow());
        }
    });

    ui.on_workflow_page_requested({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move |page| {
            if let Some(ui) = ui_weak.upgrade() {
                let info = ui.get_workflow_page_info();
                ui.set_workflow_page_info(page_request(info, page));
                if let Err(err) = refresh_workflows(&ui, &store.borrow(), None) {
                    ui.set_status_text(err.to_string().into());
                }
            }
        }
    });

    ui.on_workflow_history_page_requested({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move |page| {
            if let Some(ui) = ui_weak.upgrade() {
                let info = ui.get_workflow_history_page_info();
                ui.set_workflow_history_page_info(page_request(info, page));
                if let Err(err) = refresh_workflow_history(&ui, &store.borrow()) {
                    ui.set_status_text(err.to_string().into());
                }
            }
        }
    });

    // ---- ▶ execução simples: modal próprio, com o resultado dentro dele.
    ui.on_open_run_dialog({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let run_tool_id = Rc::clone(&run_tool_id);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = tool_id_at(&visible, index) else {
                return;
            };
            open_run_dialog(&ui, &store.borrow(), &run_tool_id, &tool_id);
        }
    });

    // A Home lista as favoritas numa ordem própria: o índice vem dessa lista, e
    // por isso é resolvido contra `favorites_visible`, não contra `visible`.
    ui.on_home_favorite_run_requested({
        let store = Rc::clone(&store);
        let favorites_visible = Rc::clone(&favorites_visible);
        let run_tool_id = Rc::clone(&run_tool_id);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = tool_id_at(&favorites_visible, index) else {
                return;
            };
            open_run_dialog(&ui, &store.borrow(), &run_tool_id, &tool_id);
        }
    });

    ui.on_close_run_dialog({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_run_open(false);
            }
        }
    });

    ui.on_run_selection_changed({
        let store = Rc::clone(&store);
        let run_tool_id = Rc::clone(&run_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let tool_id = run_tool_id.borrow().clone();
                bind_run_parameter(&ui, &store.borrow(), tool_id.as_deref());
            }
        }
    });

    ui.on_run_quick_action({
        let store = Rc::clone(&store);
        let runtime = Rc::clone(&runtime);
        let run_tool_id = Rc::clone(&run_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            // Execução simples roda sempre no Workdir global — para outro
            // diretório o caminho é a tela da ferramenta (📁).
            let workdir = PathBuf::from(ui.get_workdir_text().as_str());
            let Some(tool_id) = run_tool_id.borrow().clone() else {
                return;
            };
            let command_index = ui.get_run_selected_command().max(0) as usize;
            let param_text = ui.get_run_param_text().to_string();

            ui.set_busy(true);
            let result = {
                let store = store.borrow();
                run_tool_command(
                    &store,
                    &runtime,
                    &tool_id,
                    command_index,
                    &param_text,
                    &workdir,
                )
            };
            ui.set_busy(false);

            ui.set_run_status(result.status.into());
            ui.set_run_exit_code(result.exit_code.into());
            ui.set_run_output(result.output.into());
            ui.set_status_text(
                if result.record_error.is_empty() {
                    "Execução concluída.".into()
                } else {
                    format!("Execução concluída (histórico: {})", result.record_error).into()
                },
            );
        }
    });

    // ---- 📁 tela da ferramenta: ferramenta + Workdir próprio.
    ui.on_open_tool_detail({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = tool_id_at(&visible, index) else {
                return;
            };
            open_tool_detail(&ui, &store.borrow(), &detail_tool_id, &tool_id);
        }
    });

    ui.on_home_favorite_open_requested({
        let store = Rc::clone(&store);
        let favorites_visible = Rc::clone(&favorites_visible);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = tool_id_at(&favorites_visible, index) else {
                return;
            };
            open_tool_detail(&ui, &store.borrow(), &detail_tool_id, &tool_id);
        }
    });

    ui.on_close_tool_detail({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_tool_detail_open(false);
            }
        }
    });

    ui.on_tool_selection_changed({
        let store = Rc::clone(&store);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let tool_id = detail_tool_id.borrow().clone();
                bind_parameter_field(&ui, &store.borrow(), tool_id.as_deref());
            }
        }
    });

    // ---- ⋮ ações da ferramenta.
    ui.on_open_tool_actions({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let actions_tool_id = Rc::clone(&actions_tool_id);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = tool_id_at(&visible, index) else {
                return;
            };
            open_tool_actions(&ui, &store.borrow(), &actions_tool_id, &tool_id);
        }
    });

    ui.on_home_favorite_actions_requested({
        let store = Rc::clone(&store);
        let favorites_visible = Rc::clone(&favorites_visible);
        let actions_tool_id = Rc::clone(&actions_tool_id);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = tool_id_at(&favorites_visible, index) else {
                return;
            };
            open_tool_actions(&ui, &store.borrow(), &actions_tool_id, &tool_id);
        }
    });

    ui.on_close_tool_actions({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_tool_actions_open(false);
            }
        }
    });

    ui.on_tool_actions_toggle_favorite({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let favorites_visible = Rc::clone(&favorites_visible);
        let actions_tool_id = Rc::clone(&actions_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = actions_tool_id.borrow().clone() else {
                return;
            };

            let result = {
                let store = store.borrow();
                match store.active_workspace() {
                    Ok(workspace) => {
                        let favorite = !workspace.favorite_tool_ids.contains(&tool_id);
                        store
                            .set_tool_favorite(&workspace.id, &tool_id, favorite)
                            .map(|()| favorite)
                    }
                    Err(err) => Err(err),
                }
            };

            match result {
                Ok(favorite) => {
                    refresh_tools(&ui, &store.borrow(), &visible);
                    refresh_home_favorites(&ui, &store.borrow(), &favorites_visible);
                    // O diálogo fica aberto: a pessoa pode querer outra ação.
                    let store = store.borrow();
                    refresh_tool_actions_state(&ui, &store, &tool_id);
                    ui.set_status_text(
                        if favorite {
                            "Ferramenta marcada como favorita."
                        } else {
                            "Favorito removido."
                        }
                        .into(),
                    );
                }
                Err(err) => ui.set_status_text(err.to_string().into()),
            }
        }
    });

    ui.on_tool_actions_toggle_workspace({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let actions_tool_id = Rc::clone(&actions_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = actions_tool_id.borrow().clone() else {
                return;
            };

            let result = {
                let store = store.borrow();
                match store.active_workspace() {
                    Ok(workspace) => {
                        if workspace.tool_ids.contains(&tool_id) {
                            store.remove_tool_from_workspace(&workspace.id, &tool_id)
                        } else {
                            store.add_tool_to_workspace(&workspace.id, &tool_id)
                        }
                    }
                    Err(err) => Err(err),
                }
            };

            match result {
                Ok(()) => {
                    refresh_tools(&ui, &store.borrow(), &visible);
                    // O filtro "Do workspace" pode esconder a ferramenta agora;
                    // fechar o diálogo evita agir sobre algo que saiu da lista.
                    ui.set_tool_actions_open(false);
                    ui.set_status_text("Workspace atualizado.".into());
                }
                Err(err) => ui.set_status_text(err.to_string().into()),
            }
        }
    });

    ui.on_tool_actions_create_profile({
        let store = Rc::clone(&store);
        let actions_tool_id = Rc::clone(&actions_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = actions_tool_id.borrow().clone() else {
                return;
            };

            ui.set_tool_actions_open(false);
            prepare_create_profile_dialog(&ui, &store.borrow());
            // Pré-seleciona a ferramenta de onde o menu foi aberto.
            if let Some(index) = store.borrow().library().index_of(&tool_id) {
                ui.set_create_profile_selected_tool(index as i32);
                refresh_create_profile_selection(&ui, &store.borrow());
            }
            ui.set_create_profile_error("".into());
            ui.set_create_profile_open(true);
        }
    });

    ui.on_history_selection_changed({
        let store = Rc::clone(&store);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let tool_id = detail_tool_id.borrow().clone();
                if let Err(err) = apply_selected_history(&ui, &store.borrow(), &tool_id) {
                    ui.set_status_text(err.to_string().into());
                }
            }
        }
    });

    ui.on_profile_selection_changed({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                if let Err(err) =
                    apply_selected_profile(&ui, &store.borrow(), &visible, &detail_tool_id)
                {
                    ui.set_status_text(err.to_string().into());
                }
            }
        }
    });

    ui.on_save_workdir({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let workdir = PathBuf::from(ui.get_workdir_text().as_str());
            save_global_workdir(&ui, &store.borrow(), &workdir);
        }
    });

    // ---- seleção do Workdir por navegação de pastas. O mesmo diálogo serve ao
    // global e ao de uma ferramenta: o destino e o texto de apoio saem daqui.
    ui.on_open_choose_workdir({
        let workdir_entries = Rc::clone(&workdir_entries);
        let workdir_target = Rc::clone(&workdir_target);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            *workdir_target.borrow_mut() = WorkdirTarget::Global;
            ui.set_choose_workdir_title(GLOBAL_WORKDIR_TITLE.into());
            ui.set_choose_workdir_hint(GLOBAL_WORKDIR_HINT.into());

            let workdir = PathBuf::from(ui.get_workdir_text().as_str());
            open_choose_workdir(&ui, &workdir_entries, &workdir);
        }
    });

    ui.on_open_choose_tool_workdir({
        let workdir_entries = Rc::clone(&workdir_entries);
        let workdir_target = Rc::clone(&workdir_target);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            *workdir_target.borrow_mut() = WorkdirTarget::Tool;
            ui.set_choose_workdir_title(TOOL_WORKDIR_TITLE.into());
            ui.set_choose_workdir_hint(TOOL_WORKDIR_HINT.into());

            // Ponto de partida é o Workdir da própria ferramenta: abrir o seletor
            // em outro diretório faria a pessoa navegar de volta até onde estava.
            let workdir = PathBuf::from(ui.get_tool_workdir_text().as_str());
            open_choose_workdir(&ui, &workdir_entries, &workdir);
        }
    });

    ui.on_close_choose_workdir({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_choose_workdir_open(false);
            }
        }
    });

    ui.on_choose_workdir_entry_selected({
        let workdir_entries = Rc::clone(&workdir_entries);
        let ui_weak = ui_weak.clone();
        move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(path) = workdir_entry_at(&workdir_entries, index) else {
                return;
            };
            load_workdir_entries(&ui, &workdir_entries, &path);
        }
    });

    ui.on_choose_workdir_go({
        let workdir_entries = Rc::clone(&workdir_entries);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let typed = PathBuf::from(ui.get_choose_workdir_path().as_str());
            if !typed.is_dir() {
                ui.set_choose_workdir_error("Diretório não encontrado.".into());
                return;
            }
            load_workdir_entries(&ui, &workdir_entries, &typed);
        }
    });

    ui.on_choose_workdir_apply({
        let store = Rc::clone(&store);
        let workdir_target = Rc::clone(&workdir_target);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            // O campo é o que vale: quem digitou um caminho e confirmou não
            // deveria precisar clicar em "Ir" antes.
            let typed = PathBuf::from(ui.get_choose_workdir_path().as_str());
            if !typed.is_dir() {
                ui.set_choose_workdir_error("Diretório não encontrado.".into());
                return;
            }
            match *workdir_target.borrow() {
                // O global é preferência do Workspace: vale para as execuções
                // simples e sobrevive ao fechamento do app. Gravar é o que o
                // botão "Escolher" da barra sempre fez.
                WorkdirTarget::Global => {
                    ui.set_workdir_text(SharedString::from(typed.display().to_string()));
                    save_global_workdir(&ui, &store.borrow(), &typed);
                }
                // O da ferramenta é contexto da tela, não preferência: vive nesta
                // sessão, como o caminho digitado à mão no campo. Gravá-lo seria
                // prometer persistência que a tela nunca teve.
                WorkdirTarget::Tool => {
                    ui.set_tool_workdir_text(SharedString::from(typed.display().to_string()));
                }
            }
            ui.set_choose_workdir_open(false);
        }
    });

    ui.on_workspace_selection_changed({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let favorites_visible = Rc::clone(&favorites_visible);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let index = ui.get_selected_workspace().max(0) as usize;
            let target = {
                let store = store.borrow();
                match store.list_workspaces() {
                    Ok(workspaces) => workspaces.get(index).map(|workspace| workspace.id.clone()),
                    Err(err) => {
                        drop(store);
                        ui.set_status_text(err.to_string().into());
                        return;
                    }
                }
            };

            let Some(target) = target else {
                return;
            };

            if let Err(err) = store.borrow().set_active_workspace(&target) {
                ui.set_status_text(err.to_string().into());
                return;
            }

            refresh_tools(&ui, &store.borrow(), &visible);
            reload_workspace_scope(&ui, &store.borrow(), &favorites_visible);
            ui.set_status_text("Workspace alterado.".into());
        }
    });

    ui.on_open_create_workspace({
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            ui.set_create_workspace_name("".into());
            ui.set_create_workspace_error("".into());
            // Sugerir o workdir atual: na maioria das vezes o novo Workspace
            // começa no mesmo lugar.
            let current = ui.get_workdir_text();
            ui.set_create_workspace_workdir(current);
            ui.set_create_workspace_open(true);
        }
    });

    ui.on_close_create_workspace({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_create_workspace_open(false);
            }
        }
    });

    ui.on_confirm_create_workspace({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let favorites_visible = Rc::clone(&favorites_visible);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let name = ui.get_create_workspace_name().to_string();
            if name.trim().is_empty() {
                ui.set_create_workspace_error("Informe um nome.".into());
                return;
            }

            let workdir_text = ui.get_create_workspace_workdir().to_string();
            let workdir = if workdir_text.trim().is_empty() {
                PathBuf::from(ui.get_workdir_text().as_str())
            } else {
                PathBuf::from(workdir_text.trim())
            };

            let workspace = Workspace {
                id: Workspace::new_id(&name),
                name: name.trim().to_string(),
                global_workdir: workdir,
                tool_ids: Vec::new(),
                profile_ids: Vec::new(),
                favorite_tool_ids: Vec::new(),
            };

            {
                let store = store.borrow();
                if let Err(err) = store.save_workspace(&workspace) {
                    ui.set_create_workspace_error(err.to_string().into());
                    return;
                }
                if let Err(err) = store.set_active_workspace(&workspace.id) {
                    ui.set_create_workspace_error(err.to_string().into());
                    return;
                }
            }

            ui.set_create_workspace_open(false);
            refresh_tools(&ui, &store.borrow(), &visible);
            reload_workspace_scope(&ui, &store.borrow(), &favorites_visible);
            ui.set_status_text("Workspace criado.".into());
        }
    });

    ui.on_rename_workspace({
        let store = Rc::clone(&store);
        let favorites_visible = Rc::clone(&favorites_visible);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let name = ui.get_workspace_name().to_string();

            let result = {
                let store = store.borrow();
                store.active_workspace().and_then(|mut workspace| {
                    workspace.name = name.trim().to_string();
                    store.save_workspace(&workspace)
                })
            };

            match result {
                Ok(()) => {
                    reload_workspace_scope(&ui, &store.borrow(), &favorites_visible);
                    ui.set_status_text("Workspace renomeado.".into());
                }
                Err(err) => ui.set_status_text(err.to_string().into()),
            }
        }
    });

    ui.on_delete_workspace({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let favorites_visible = Rc::clone(&favorites_visible);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let result = {
                let store = store.borrow();
                let active = match store.active_workspace() {
                    Ok(active) => active,
                    Err(err) => {
                        drop(store);
                        ui.set_status_text(err.to_string().into());
                        return;
                    }
                };
                store.delete_workspace(&active.id)
            };

            match result {
                Ok(()) => {
                    refresh_tools(&ui, &store.borrow(), &visible);
                    reload_workspace_scope(&ui, &store.borrow(), &favorites_visible);
                    ui.set_status_text("Workspace removido.".into());
                }
                Err(err) => ui.set_status_text(err.to_string().into()),
            }
        }
    });

    ui.on_open_create_profile({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            prepare_create_profile_dialog(&ui, &store.borrow());
            ui.set_create_profile_error("".into());
            ui.set_create_profile_open(true);
        }
    });

    ui.on_close_create_profile({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_create_profile_open(false);
                ui.set_create_profile_error("".into());
            }
        }
    });

    ui.on_create_profile_selection_changed({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                refresh_create_profile_selection(&ui, &store.borrow());
            }
        }
    });

    ui.on_confirm_create_profile({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            match save_profile_from_dialog(&ui, &store.borrow()) {
                Ok(name) => {
                    if let Err(err) = refresh_profiles(&ui, &store.borrow()) {
                        ui.set_create_profile_error(err.to_string().into());
                        return;
                    }
                    ui.set_create_profile_open(false);
                    ui.set_create_profile_name("".into());
                    ui.set_create_profile_error("".into());
                    ui.set_status_text(format!("Perfil `{name}` salvo.").into());
                }
                Err(err) => ui.set_create_profile_error(err.into()),
            }
        }
    });

    ui.on_run_action({
        let store = Rc::clone(&store);
        let runtime = Rc::clone(&runtime);
        let detail_tool_id = Rc::clone(&detail_tool_id);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(tool_id) = detail_tool_id.borrow().clone() else {
                return;
            };

            let command_index = ui.get_selected_command().max(0) as usize;
            let param_text = ui.get_param_text().to_string();
            // A tela da ferramenta usa o Workdir dela, não o global.
            let workdir = PathBuf::from(ui.get_tool_workdir_text().as_str());

            ui.set_busy(true);
            ui.set_execution_status("Running…".into());
            ui.set_execution_exit_code("-".into());
            ui.set_execution_output("".into());

            let result = {
                let store = store.borrow();
                run_tool_command(
                    &store,
                    &runtime,
                    &tool_id,
                    command_index,
                    &param_text,
                    &workdir,
                )
            };

            ui.set_busy(false);
            ui.set_execution_status(result.status.into());
            ui.set_execution_exit_code(result.exit_code.into());
            ui.set_execution_output(result.output.into());

            let store = store.borrow();
            if !result.record_error.is_empty() {
                ui.set_status_text(format!("Histórico: {}", result.record_error).into());
            } else if let Err(err) = refresh_tool_history(&ui, &store, &tool_id) {
                ui.set_status_text(err.to_string().into());
            }
        }
    });

    ui.on_discover_manifests({
        let store = Rc::clone(&store);
        let discovered = Rc::clone(&discovered);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let dir = manifests_dir(&ui);
            if let Err(err) = std::fs::create_dir_all(&dir) {
                discovered.borrow_mut().clear();
                refresh_discovered_candidates(&ui, &store.borrow(), &discovered.borrow());
                let message = format!("Não foi possível usar `{}`: {err}", dir.display());
                ui.set_candidate_detail_text(message.clone().into());
                ui.set_status_text(message.into());
                return;
            }

            let candidates = discover_manifests(&dir);
            let total = candidates.len();
            let valid = candidates
                .iter()
                .filter(|candidate| candidate.is_valid())
                .count();

            // Descoberta é uma lista nova: volta para a primeira página em vez de
            // abrir no meio do resultado anterior.
            *discovered.borrow_mut() = candidates;
            let info = ui.get_candidate_page_info();
            ui.set_candidate_page_info(page_request(info, 0));
            ui.set_selected_candidate(0);
            refresh_discovered_candidates(&ui, &store.borrow(), &discovered.borrow());

            if total == 0 {
                ui.set_candidate_detail_text(
                    format!("Nenhum arquivo .json em {}.", dir.display()).into(),
                );
            }

            ui.set_status_text(
                format!(
                    "{total} arquivo(s) em {} · {valid} importável(is).",
                    dir.display()
                )
                .into(),
            );
        }
    });

    ui.on_candidate_selection_changed({
        let store = Rc::clone(&store);
        let discovered = Rc::clone(&discovered);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                refresh_candidate_detail(&ui, &store.borrow(), &discovered.borrow());
            }
        }
    });

    ui.on_import_manifest({
        let store = Rc::clone(&store);
        let visible = Rc::clone(&visible);
        let discovered = Rc::clone(&discovered);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let index = ui.get_selected_candidate() as usize;
            let path = discovered
                .borrow()
                .get(index)
                .map(|candidate| candidate.path.clone());
            let Some(path) = path else {
                ui.set_candidate_detail_text(
                    "Selecione um manifest; use “Procurar” para atualizar a lista.".into(),
                );
                return;
            };

            match store.borrow_mut().import_manifest_file(&path) {
                Ok(outcome) => {
                    let message = if outcome.created_version {
                        format!(
                            "Ferramenta `{}` importada (manifest versão {}).",
                            outcome.name, outcome.version
                        )
                    } else {
                        format!(
                            "`{}` já estava na versão {} — nada mudou.",
                            outcome.name, outcome.version
                        )
                    };

                    refresh_tools(&ui, &store.borrow(), &visible);
                    refresh_candidate_detail(&ui, &store.borrow(), &discovered.borrow());
                    ui.set_status_text(message.into());
                }
                Err(err) => {
                    let message = format!("Importação falhou: {err}");
                    ui.set_candidate_detail_text(message.clone().into());
                    ui.set_status_text(message.into());
                }
            }
        }
    });

    ui.on_workflow_selection_changed({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            match apply_selected_workflow(&ui, &store.borrow()) {
                Ok(name) => ui.set_status_text(format!("Workflow `{name}` selecionado.").into()),
                Err(err) => ui.set_status_text(err.into()),
            }
        }
    });

    ui.on_workflow_history_selection_changed({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                match apply_selected_workflow_history(&ui, &store.borrow()) {
                    Ok(label) if !label.is_empty() => ui.set_status_text(label.into()),
                    Ok(_) => {}
                    Err(err) => ui.set_status_text(err.to_string().into()),
                }
            }
        }
    });

    ui.on_run_workflow({
        let store = Rc::clone(&store);
        let runtime = Rc::clone(&runtime);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            ui.set_busy(true);
            ui.set_status_text("Executando workflow…".into());

            let result = run_selected_workflow(&ui, &store.borrow(), &runtime);
            ui.set_busy(false);

            match result {
                Ok(summary) => {
                    ui.set_status_text(summary.into());
                    if let Err(err) = refresh_workflow_history(&ui, &store.borrow()) {
                        ui.set_status_text(err.to_string().into());
                        return;
                    }
                    // A execução recém-gravada é a mais recente: mostrar suas etapas.
                    let _ = apply_selected_workflow_history(&ui, &store.borrow());
                }
                Err(err) => {
                    ui.set_status_text(err.into());
                    set_workflow_step_results(&ui, Vec::<String>::new());
                }
            }
        }
    });

    ui.on_open_create_workflow({
        let store = Rc::clone(&store);
        let draft = Rc::clone(&workflow_draft);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let mut draft = draft.borrow_mut();
            prepare_create_workflow_dialog(&ui, &store.borrow(), &mut draft);
            ui.set_create_workflow_error("".into());
            ui.set_create_workflow_open(true);
        }
    });

    ui.on_close_create_workflow({
        let draft = Rc::clone(&workflow_draft);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_create_workflow_open(false);
                // A etapa em construção é parte do rascunho: se ficou aberta, sai
                // junto. Sem isto, um estado deixado para trás reabriria o modal
                // por cima da próxima criação.
                ui.set_create_workflow_step_open(false);
                ui.set_create_workflow_error("".into());
                ui.set_create_workflow_step_error("".into());
                draft.borrow_mut().clear();
            }
        }
    });

    ui.on_create_workflow_selection_changed({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                refresh_create_workflow_selection(&ui, &store.borrow());
            }
        }
    });

    // A etapa é detalhada em modal, aberto pela tela de criação. Abrir não cria
    // nada: só prepara os campos com a ferramenta e a ação já escolhidas.
    ui.on_open_workflow_step({
        let store = Rc::clone(&store);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let store = store.borrow();
            // Sem `prepare`: a seleção de ferramenta e ação da etapa anterior é
            // preservada de propósito — repetir a ferramenta é o caso comum.
            // Só o parâmetro volta ao padrão da ação, porque a etapa é nova.
            refresh_create_workflow_selection(&ui, &store);
            ui.set_create_workflow_step_workdir("".into());
            ui.set_create_workflow_step_error("".into());
            ui.set_create_workflow_step_open(true);
        }
    });

    ui.on_close_workflow_step({
        let ui_weak = ui_weak.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                // A lista de etapas é do rascunho, não do modal: fechar sem
                // confirmar descarta só a etapa em construção.
                ui.set_create_workflow_step_open(false);
                ui.set_create_workflow_step_error("".into());
            }
        }
    });

    ui.on_add_workflow_step({
        let store = Rc::clone(&store);
        let draft = Rc::clone(&workflow_draft);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let mut draft = draft.borrow_mut();
            add_workflow_step(&ui, &store.borrow(), &mut draft);
        }
    });

    ui.on_remove_workflow_step({
        let draft = Rc::clone(&workflow_draft);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let mut draft = draft.borrow_mut();
            draft.pop();
            set_workflow_step_labels(&ui, &draft);
            ui.set_create_workflow_step_error("".into());
        }
    });

    ui.on_confirm_create_workflow({
        let store = Rc::clone(&store);
        let draft = Rc::clone(&workflow_draft);
        let ui_weak = ui_weak.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let result = {
                let draft = draft.borrow();
                save_workflow_from_dialog(&ui, &store.borrow(), &draft[..])
            };

            match result {
                Ok(name) => {
                    if let Err(err) = refresh_workflows(&ui, &store.borrow(), Some(&name)) {
                        ui.set_create_workflow_error(err.to_string().into());
                        return;
                    }
                    workflow_draft.borrow_mut().clear();
                    ui.set_create_workflow_open(false);
                    ui.set_create_workflow_step_open(false);
                    ui.set_create_workflow_name("".into());
                    ui.set_create_workflow_error("".into());
                    ui.set_status_text(format!("Workflow `{name}` salvo.").into());
                }
                Err(err) => ui.set_create_workflow_error(err.into()),
            }
        }
    });

    // Slint não expõe posicionamento de janela; centralizamos via backend winit
    // assim que o event loop estiver ativo e a janela existir.
    center_window_when_ready(&ui);

    ui.run()?;
    Ok(())
}

/// Agenda a centralização da janela até que o window do winit exista.
///
/// O window só existe depois que o event loop inicia, por isso tentamos algumas
/// vezes em `Timer` de curto intervalo até conseguir posicionar (ou desistir).
fn center_window_when_ready(ui: &AppWindow) {
    const ATTEMPTS: u32 = 20;
    const INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    let ui_weak = ui.as_weak();
    let timer = Rc::new(slint::Timer::default());
    let timer_handle = Rc::clone(&timer);
    let mut attempts = 0;

    timer.start(slint::TimerMode::Repeated, INTERVAL, move || {
        attempts += 1;
        let centered = ui_weak
            .upgrade()
            .map(|ui| center_window(ui.window()))
            .unwrap_or(true);

        if centered || attempts >= ATTEMPTS {
            timer_handle.stop();
        }
    });
}

/// Respiro reservado ao medir se a janela cabe na tela: bordas do sistema na
/// largura e barra de tarefas na altura.
const SCREEN_MARGIN_WIDTH: u32 = 16;
const SCREEN_MARGIN_HEIGHT: u32 = 80;

/// Centraliza a janela no monitor primário e a encolhe quando ela não cabe.
///
/// Em telas menores que o tamanho preferido, encolher é melhor do que deixar
/// parte da interface fora da área visível.
///
/// Retorna `false` enquanto o window do winit ainda não estiver disponível.
fn center_window(window: &slint::Window) -> bool {
    window
        .with_winit_window(|winit_window| {
            let Some(monitor) = winit_window
                .primary_monitor()
                .or_else(|| winit_window.current_monitor())
            else {
                return false;
            };

            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            let window_size = winit_window.outer_size();

            let max_width = monitor_size.width.saturating_sub(SCREEN_MARGIN_WIDTH);
            let max_height = monitor_size.height.saturating_sub(SCREEN_MARGIN_HEIGHT);
            let target_width = window_size.width.min(max_width);
            let target_height = window_size.height.min(max_height);

            if target_width != window_size.width || target_height != window_size.height {
                let _ = winit_window
                    .request_inner_size(winit::dpi::PhysicalSize::new(target_width, target_height));
            }

            // Centraliza pelo tamanho alvo: o redimensionamento é aplicado depois,
            // e usar o tamanho atual deixaria a janela deslocada.
            let x = monitor_position.x + (monitor_size.width as i32 - target_width as i32) / 2;
            let y = monitor_position.y + (monitor_size.height as i32 - target_height as i32) / 2;

            winit_window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
            true
        })
        .unwrap_or(false)
}

fn bind_workspace(ui: &AppWindow, store: &Store) -> Result<(), Box<dyn std::error::Error>> {
    let workspaces = store.list_workspaces()?;
    let active = store.active_workspace()?;

    let names: Vec<SharedString> = workspaces
        .iter()
        .map(|workspace| SharedString::from(workspace.name.as_str()))
        .collect();
    // Se o ativo não estiver na lista, cair no primeiro em vez de em índice inválido.
    let index = workspaces
        .iter()
        .position(|workspace| workspace.id == active.id)
        .unwrap_or(0);

    ui.set_workspace_names(ModelRc::new(VecModel::from(names)));
    ui.set_selected_workspace(index as i32);
    ui.set_workspace_name(SharedString::from(active.name.as_str()));
    ui.set_workspace_is_default(active.is_default());
    ui.set_workdir_text(SharedString::from(
        active.global_workdir.display().to_string(),
    ));
    Ok(())
}

/// Publica a lista de candidatos já descobertos, recortada na página atual.
///
/// Separada da descoberta porque as duas acontecem em momentos diferentes: a
/// descoberta lê a pasta, isto só redesenha a lista — e é o que a paginação pede.
fn refresh_discovered_candidates(
    ui: &AppWindow,
    store: &Store,
    discovered: &[ManifestCandidate],
) {
    let labels: Vec<SharedString> = discovered
        .iter()
        .map(|candidate| SharedString::from(candidate.label()))
        .collect();

    let (range, info) = Pager::requested(ui.get_candidate_page_info()).slice(labels.len());
    ui.set_manifest_candidates(ModelRc::new(VecModel::from(labels[range].to_vec())));
    ui.set_candidate_page_info(info);

    refresh_candidate_detail(ui, store, discovered);
}

/// Mostra o candidato selecionado já considerando o estado atual do app.
fn refresh_candidate_detail(ui: &AppWindow, store: &Store, discovered: &[ManifestCandidate]) {
    let index = ui.get_selected_candidate() as usize;
    let Some(candidate) = discovered.get(index) else {
        ui.set_candidate_detail_text("".into());
        return;
    };

    let mut detail = candidate.detail();
    if let Some(manifest) = &candidate.manifest {
        match store.library().get(&manifest.id) {
            Some(existing) if existing.bundled => detail.push_str(" · já embutido no app"),
            Some(_) => detail.push_str(" · já importado · importar de novo atualiza a versão"),
            None => {}
        }
    }
    ui.set_candidate_detail_text(detail.into());
}

/// Pasta de manifests informada na UI; vazia significa a pasta padrão.
fn manifests_dir(ui: &AppWindow) -> PathBuf {
    let typed = ui.get_manifests_dir_text().trim().to_string();
    if typed.is_empty() {
        default_manifests_dir()
    } else {
        PathBuf::from(typed)
    }
}

fn default_manifests_dir() -> PathBuf {
    dirs_next_data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("uige")
        .join("manifests")
}

/// Ferramentas visíveis na tela de Ferramentas, na ordem exibida.
///
/// O índice da lista na UI é a posição **aqui**, não na biblioteca: com busca ou
/// filtro ativos os dois não coincidem.
type VisibleTools = Rc<RefCell<Vec<String>>>;

/// Destino do caminho escolhido no diálogo de Workdir.
///
/// O seletor de pastas é um só — abrir dois diálogos quase iguais seria pior do
/// que dizer a que Workdir ele serve. O que muda entre os dois casos é o destino
/// do valor e o que acontece com ele: o global é preferência do Workspace e
/// sobrevive ao app; o da ferramenta é contexto da tela e vive na sessão.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WorkdirTarget {
    Global,
    Tool,
}

const GLOBAL_WORKDIR_TITLE: &str = "Workdir global";
const GLOBAL_WORKDIR_HINT: &str = "Pasta onde as execuções simples deste workspace vão rodar.";

const TOOL_WORKDIR_TITLE: &str = "Workdir da ferramenta";
const TOOL_WORKDIR_HINT: &str =
    "Pasta onde esta ferramenta vai rodar. Alterar aqui não muda o Workdir global.";

/// Teto de favoritas exibidas na Home.
///
/// A Home é resumo; a lista completa de ferramentas é a seção Ferramentas. O teto
/// também é o tamanho de página das listas (`page-size` nas `SelectList`).
const MAX_HOME_FAVORITES: usize = 10;

/// Itens por página das listas longas.
///
/// Um número só para todas: não há motivo para uma lista paginar diferente da
/// outra, e valores diferentes por tela viram dúvida sem ganho.
const PAGE_SIZE: usize = 10;

/// Recorte de página de uma lista longa.
///
/// A página é lida do estado publicado na UI (`PageInfo`) e devolvida já ajustada
/// ao tamanho atual da lista: uma propriedade só por lista, sem estado paralelo —
/// o pedido da UI e o recorte do app são o mesmo número em pontos diferentes do
/// mesmo ciclo. A UI não fatiar um modelo é o motivo de o recorte ser feito aqui.
#[derive(Clone, Copy)]
struct Pager {
    page: usize,
}

impl Pager {
    /// Página pedida, como a UI a deixou no estado publicado.
    fn requested(info: PageInfo) -> Self {
        Self {
            page: info.page.max(0) as usize,
        }
    }

    /// Volta para a primeira página. Vale quando a lista muda de assunto — uma
    /// busca nova não deve abrir no meio do resultado anterior.
    fn first_page(&mut self) {
        self.page = 0;
    }

    /// Põe a página onde está o item `index`. Uma seleção escolhida pelo app
    /// (a execução recém-gravada, o workflow recém-criado) precisa estar à vista.
    fn reveal(&mut self, index: usize) {
        self.page = index / PAGE_SIZE;
    }

    /// Faixa da lista inteira que esta página mostra, e o estado a publicar de
    /// volta na UI.
    fn slice(&mut self, total: usize) -> (std::ops::Range<usize>, PageInfo) {
        self.page = self.page.min(last_page(total));
        let start = (self.page * PAGE_SIZE).min(total);
        let range = start..(start + PAGE_SIZE).min(total);
        let info = PageInfo {
            page: self.page as i32,
            page_count: page_count(total) as i32,
            total: total as i32,
            offset: start as i32,
        };
        (range, info)
    }
}

/// Página publicada na UI com um novo pedido, mantendo o resto do estado.
fn page_request(info: PageInfo, page: i32) -> PageInfo {
    PageInfo { page, ..info }
}

fn last_page(total: usize) -> usize {
    total.saturating_sub(1) / PAGE_SIZE
}

fn page_count(total: usize) -> usize {
    if total == 0 {
        0
    } else {
        last_page(total) + 1
    }
}

/// Id da ferramenta na posição `index` da lista visível.
///
/// O índice vem do callback da linha (▶ / 📁 / ⋮) e é a posição na lista
/// filtrada, não na biblioteca.
fn tool_id_at(visible: &VisibleTools, index: i32) -> Option<String> {
    if index < 0 {
        return None;
    }
    visible.borrow().get(index as usize).cloned()
}

fn compute_visible_tools(store: &Store, search: &str, workspace_only: bool) -> Vec<String> {
    let needle = search.trim().to_lowercase();
    let workspace = store.active_workspace().ok();

    store
        .tools()
        .iter()
        .filter(|tool| {
            needle.is_empty()
                || tool.name.to_lowercase().contains(&needle)
                || tool.id.to_lowercase().contains(&needle)
        })
        .filter(|tool| match (&workspace, workspace_only) {
            (_, false) => true,
            (Some(workspace), true) => workspace.tool_ids.contains(&tool.id),
            (None, true) => false,
        })
        .map(|tool| tool.id.clone())
        .collect()
}

/// Reconstrói a lista de ferramentas a partir da busca e do filtro atuais.
fn refresh_tools(ui: &AppWindow, store: &Store, visible: &VisibleTools) {
    let ids = compute_visible_tools(
        store,
        ui.get_tool_search_text().as_str(),
        ui.get_tool_workspace_only(),
    );

    let favorites = store
        .active_workspace()
        .map(|workspace| workspace.favorite_tool_ids)
        .unwrap_or_default();

    // O ★ vai no rótulo: a lista não guarda estado por linha. Id e rótulo são
    // montados juntos porque a página recorta os dois — em listas separadas, uma
    // ferramenta sem rótulo deslocaria todas as seguintes.
    let entries: Vec<(String, SharedString)> = ids
        .iter()
        .filter_map(|tool_id| {
            let tool = store.library().get(tool_id)?;
            let label = if favorites.contains(tool_id) {
                format!("★ {}", tool.name)
            } else {
                tool.name.clone()
            };
            Some((tool_id.clone(), SharedString::from(label)))
        })
        .collect();

    let labels: Vec<SharedString> = entries.iter().map(|(_, label)| label.clone()).collect();
    let (range, info) = Pager::requested(ui.get_tool_page_info()).slice(labels.len());
    ui.set_tool_names(ModelRc::new(VecModel::from(labels[range.clone()].to_vec())));
    ui.set_tool_page_info(info);

    // `visible` é o que está à mostra e é contra ela que os índices das linhas são
    // resolvidos: recortar as duas com a mesma faixa mantém os dois lados falando
    // da mesma página.
    *visible.borrow_mut() = entries[range]
        .iter()
        .map(|(tool_id, _)| tool_id.clone())
        .collect();
}

/// Reconstrói a lista de favoritas do Workspace ativo, exibida na Home.
///
/// Só entram ferramentas que ainda existem na biblioteca: um favorito órfão
/// (definição removida) viraria uma linha sem ação. A ordem é por nome, e não a
/// do banco, para a Home não depender da ordem de inserção.
///
/// A Home é ponto de partida, não a lista de Ferramentas: mostra no máximo
/// [`MAX_HOME_FAVORITES`] e informa quantas ficaram de fora. Cortar em silêncio
/// esconderia favoritos sem que a pessoa soubesse por quê.
fn refresh_home_favorites(ui: &AppWindow, store: &Store, favorites: &VisibleTools) {
    let mut entries: Vec<(String, String)> = store
        .active_workspace()
        .map(|workspace| {
            workspace
                .favorite_tool_ids
                .iter()
                .filter_map(|tool_id| {
                    let tool = store.library().get(tool_id)?;
                    Some((tool.name.clone(), tool_id.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    entries.sort_by_cached_key(|(name, _)| name.to_lowercase());

    let total = entries.len();
    entries.truncate(MAX_HOME_FAVORITES);

    let labels: Vec<SharedString> = entries
        .iter()
        .map(|(name, _)| SharedString::from(name.as_str()))
        .collect();
    ui.set_favorite_tools(ModelRc::new(VecModel::from(labels)));
    ui.set_favorite_tools_note(
        if total > MAX_HOME_FAVORITES {
            format!(
                "Mostrando {MAX_HOME_FAVORITES} de {total} favoritas · veja todas em Ferramentas."
            )
        } else {
            String::new()
        }
        .into(),
    );

    *favorites.borrow_mut() = entries.into_iter().map(|(_, tool_id)| tool_id).collect();
}

/// Resultado de uma execução, já formatado para exibição.
///
/// `record_error` é vazio quando o histórico foi gravado. Fora isso, carrega o
/// motivo da falha sem esconder o resultado da execução.
struct RunResult {
    status: String,
    exit_code: String,
    output: String,
    record_error: String,
}

impl RunResult {
    fn failure(message: impl Into<String>) -> Self {
        Self {
            status: "Failed".to_string(),
            exit_code: "-".to_string(),
            output: message.into(),
            record_error: String::new(),
        }
    }
}

fn status_label(status: ExecutionStatus) -> &'static str {
    match status {
        ExecutionStatus::Success => "Success",
        ExecutionStatus::Failed => "Failed",
    }
}

/// Executa um comando, registra no histórico e devolve o resultado formatado.
///
/// Compartilhado pelo modal ▶ e pela tela da ferramenta: os dois executam a
/// mesma coisa e só diferem no Workdir e em onde mostram o resultado.
fn run_tool_command(
    store: &Store,
    runtime: &tokio::runtime::Runtime,
    tool_id: &str,
    command_index: usize,
    param_text: &str,
    workdir: &Path,
) -> RunResult {
    let Some(tool) = store.library().get(tool_id) else {
        return RunResult::failure("Ferramenta não encontrada.");
    };
    let Some(command) = tool.manifest.commands.get(command_index) else {
        return RunResult::failure("Ação inválida.");
    };

    let values = collect_parameter_values(command, param_text);
    let context = ExecutionContext {
        workdir: if workdir.as_os_str().is_empty() {
            None
        } else {
            Some(workdir.to_path_buf())
        },
    };

    let outcome = match runtime.block_on(execute_command(
        &tool.manifest,
        command,
        &values,
        &context,
    )) {
        Ok(outcome) => outcome,
        Err(err) => return RunResult::failure(err.to_string()),
    };

    let workdir_ref = if workdir.as_os_str().is_empty() {
        None
    } else {
        Some(workdir)
    };
    let record_error = store
        .record_execution(
            &active_workspace_id(store),
            tool_id,
            &command.id,
            &values,
            workdir_ref,
            &outcome,
        )
        .err()
        .map(|err| err.to_string())
        .unwrap_or_default();

    RunResult {
        status: status_label(outcome.status).to_string(),
        exit_code: outcome
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "-".to_string()),
        output: format_output(&outcome.stdout, &outcome.stderr),
        record_error,
    }
}

/// Abre o modal de execução simples (▶) para `tool_id`.
fn open_run_dialog(
    ui: &AppWindow,
    store: &Store,
    run_tool_id: &Rc<RefCell<Option<String>>>,
    tool_id: &str,
) {
    let Some(tool) = store.library().get(tool_id) else {
        return;
    };

    ui.set_run_tool_name(SharedString::from(tool.name.as_str()));
    ui.set_run_command_names(ModelRc::new(VecModel::from(
        tool.manifest
            .commands
            .iter()
            .map(|command| SharedString::from(command.name.as_str()))
            .collect::<Vec<_>>(),
    )));
    ui.set_run_selected_command(0);
    // Um resultado anterior não deve aparecer como se fosse desta execução.
    ui.set_run_status("-".into());
    ui.set_run_exit_code("-".into());
    ui.set_run_output("".into());

    *run_tool_id.borrow_mut() = Some(tool_id.to_string());
    bind_run_parameter(ui, store, Some(tool_id));
    ui.set_run_open(true);
}

/// Ajusta o campo de parâmetro do modal ▶ à ação selecionada.
fn bind_run_parameter(ui: &AppWindow, store: &Store, tool_id: Option<&str>) {
    let command = tool_id
        .and_then(|tool_id| store.library().get(tool_id))
        .and_then(|tool| {
            tool.manifest
                .commands
                .get(ui.get_run_selected_command().max(0) as usize)
        });

    if let Some(command) = command {
        if let Some(parameter) = first_string_parameter(command) {
            ui.set_run_show_param(true);
            ui.set_run_param_label(SharedString::from(parameter.name.as_str()));
            ui.set_run_param_text(parameter.default.clone().unwrap_or_default().into());
        } else {
            ui.set_run_show_param(false);
            ui.set_run_param_text("".into());
        }
    } else {
        ui.set_run_show_param(false);
        ui.set_run_param_text("".into());
    }
}

/// Abre a tela da ferramenta (📁) para `tool_id`.
fn open_tool_detail(
    ui: &AppWindow,
    store: &Store,
    detail_tool_id: &Rc<RefCell<Option<String>>>,
    tool_id: &str,
) {
    let Some(tool) = store.library().get(tool_id) else {
        return;
    };

    ui.set_selected_tool_name(SharedString::from(tool.name.as_str()));

    let version = store.current_manifest_version(&tool.id).unwrap_or(0);
    ui.set_manifest_version_text(SharedString::from(format!("definição v{version}")));

    ui.set_command_names(ModelRc::new(VecModel::from(
        tool.manifest
            .commands
            .iter()
            .map(|command| SharedString::from(command.name.as_str()))
            .collect::<Vec<_>>(),
    )));
    ui.set_selected_command(0);
    // A tela começa no Workdir global; alterar aqui não muda o global.
    ui.set_tool_workdir_text(ui.get_workdir_text());
    ui.set_execution_status("-".into());
    ui.set_execution_exit_code("-".into());
    ui.set_execution_output("".into());

    *detail_tool_id.borrow_mut() = Some(tool_id.to_string());
    bind_parameter_field(ui, store, Some(tool_id));

    if let Err(err) = refresh_tool_history(ui, store, tool_id) {
        ui.set_status_text(err.to_string().into());
    }
    ui.set_tool_detail_open(true);
}

/// Ajusta o campo de parâmetro da tela da ferramenta à ação selecionada.
fn bind_parameter_field(ui: &AppWindow, store: &Store, tool_id: Option<&str>) {
    let command = tool_id
        .and_then(|tool_id| store.library().get(tool_id))
        .and_then(|tool| {
            tool.manifest
                .commands
                .get(ui.get_selected_command().max(0) as usize)
        });

    if let Some(command) = command {
        if let Some(parameter) = first_string_parameter(command) {
            ui.set_show_param(true);
            ui.set_param_label(SharedString::from(parameter.name.as_str()));
            ui.set_param_text(parameter.default.clone().unwrap_or_default().into());
        } else {
            ui.set_show_param(false);
            ui.set_param_text("".into());
        }
    } else {
        ui.set_show_param(false);
        ui.set_param_text("".into());
    }
}

/// Abre o modal de ações (⋮) para `tool_id`.
fn open_tool_actions(
    ui: &AppWindow,
    store: &Store,
    actions_tool_id: &Rc<RefCell<Option<String>>>,
    tool_id: &str,
) {
    if store.library().get(tool_id).is_none() {
        return;
    }

    *actions_tool_id.borrow_mut() = Some(tool_id.to_string());
    refresh_tool_actions_state(ui, store, tool_id);
    ui.set_tool_actions_open(true);
}

/// Reflete no modal ⋮ o estado atual da ferramenta.
fn refresh_tool_actions_state(ui: &AppWindow, store: &Store, tool_id: &str) {
    let Some(tool) = store.library().get(tool_id) else {
        return;
    };
    ui.set_tool_actions_name(SharedString::from(tool.name.as_str()));

    let workspace = store.active_workspace().ok();
    ui.set_tool_actions_is_favorite(
        workspace
            .as_ref()
            .map(|workspace| workspace.favorite_tool_ids.iter().any(|id| id == tool_id))
            .unwrap_or(false),
    );
    ui.set_tool_actions_in_workspace(
        workspace
            .as_ref()
            .map(|workspace| workspace.tool_ids.iter().any(|id| id == tool_id))
            .unwrap_or(false),
    );
}

/// Histórico da tela da ferramenta: só as execuções daquela ferramenta.
fn refresh_tool_history(
    ui: &AppWindow,
    store: &Store,
    tool_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = store.list_recent_executions_for_tool(
        &active_workspace_id(store),
        tool_id,
        20,
    )?;
    let labels: Vec<SharedString> = records
        .iter()
        .map(|record| SharedString::from(record.label()))
        .collect();

    // A lista é o histórico de uma ferramenta recém-aberta, ou com uma execução
    // recém-gravada: começar pelo mais recente é o que se espera das duas vezes.
    let mut pager = Pager::requested(ui.get_history_page_info());
    pager.first_page();

    let (range, info) = pager.slice(labels.len());
    ui.set_history_labels(ModelRc::new(VecModel::from(labels[range].to_vec())));
    ui.set_history_page_info(info);
    ui.set_selected_history(0);
    Ok(())
}

fn reload_workspace_scope(ui: &AppWindow, store: &Store, favorites_visible: &VisibleTools) {
    if let Err(err) = bind_workspace(ui, store) {
        ui.set_status_text(err.to_string().into());
    }
    if let Err(err) = refresh_profiles(ui, store) {
        ui.set_status_text(err.to_string().into());
    }
    if let Err(err) = refresh_history(ui, store) {
        ui.set_status_text(err.to_string().into());
    }
    if let Err(err) = refresh_workflows(ui, store, None) {
        ui.set_status_text(err.to_string().into());
    }
    // As favoritas são do Workspace: trocar de Workspace troca a lista da Home.
    refresh_home_favorites(ui, store, favorites_visible);
}

fn active_workspace_id(store: &Store) -> String {
    store
        .active_workspace()
        .map(|workspace| workspace.id)
        .unwrap_or_else(|_| uige_core::DEFAULT_WORKSPACE_ID.to_string())
}

/// Grava o Workdir global do Workspace ativo e informa o resultado na barra.
fn save_global_workdir(ui: &AppWindow, store: &Store, workdir: &Path) {
    let workspace_id = active_workspace_id(store);
    match store.set_global_workdir(&workspace_id, workdir) {
        Ok(()) => ui.set_status_text("Workdir salvo.".into()),
        Err(err) => ui.set_status_text(err.to_string().into()),
    }
}

/// Preenche o diálogo de Workdir com o conteúdo de `path`.
///
/// A primeira linha é a pasta acima, quando existe: subir e descer viram a mesma
/// ação e a lista não precisa de um botão extra.
fn load_workdir_entries(
    ui: &AppWindow,
    entries: &Rc<RefCell<Vec<(String, PathBuf)>>>,
    path: &Path,
) {
    let mut list: Vec<(String, PathBuf)> = Vec::new();
    if let Some(parent) = path.parent() {
        list.push(("..".to_string(), parent.to_path_buf()));
    }
    for directory in list_subdirectories(path) {
        let label = directory
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| directory.display().to_string());
        list.push((label, directory));
    }

    let labels: Vec<SharedString> = list
        .iter()
        .map(|(label, _)| SharedString::from(label.as_str()))
        .collect();

    ui.set_choose_workdir_path(SharedString::from(path.display().to_string()));
    ui.set_choose_workdir_entries(ModelRc::new(VecModel::from(labels)));
    // Nada selecionado: a primeira linha é a pasta acima, e destacá-la sugeriria
    // que subir é a escolha esperada.
    ui.set_choose_workdir_selected(-1);
    ui.set_choose_workdir_error("".into());

    *entries.borrow_mut() = list;
}

/// Abre o diálogo de Workdir a partir do diretório atual do Workspace.
fn open_choose_workdir(
    ui: &AppWindow,
    entries: &Rc<RefCell<Vec<(String, PathBuf)>>>,
    workdir: &Path,
) {
    // Caminho inexistente (ou que não é pasta) não serve de ponto de partida: a
    // listagem viria vazia e pareceria defeito do app.
    let start = if workdir.is_dir() {
        workdir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };
    load_workdir_entries(ui, entries, &start);
    ui.set_choose_workdir_open(true);
}

/// Pasta na posição `index` da lista do diálogo de Workdir.
fn workdir_entry_at(entries: &Rc<RefCell<Vec<(String, PathBuf)>>>, index: i32) -> Option<PathBuf> {
    if index < 0 {
        return None;
    }
    entries
        .borrow()
        .get(index as usize)
        .map(|(_, path)| path.clone())
}

fn refresh_history(ui: &AppWindow, store: &Store) -> Result<(), Box<dyn std::error::Error>> {
    let records = store.list_recent_executions(&active_workspace_id(store), 20)?;
    let labels: Vec<SharedString> = records
        .iter()
        .map(|record| SharedString::from(record.label()))
        .collect();
    ui.set_history_labels(ModelRc::new(VecModel::from(labels)));
    if !records.is_empty() {
        ui.set_selected_history(0);
    }
    Ok(())
}

fn apply_selected_history(
    ui: &AppWindow,
    store: &Store,
    tool_id: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_id = active_workspace_id(store);
    // A lista da tela da ferramenta é filtrada por ferramenta.
    let records = match tool_id {
        Some(tool_id) => store.list_recent_executions_for_tool(&workspace_id, tool_id, 20)?,
        None => store.list_recent_executions(&workspace_id, 20)?,
    };

    let index = ui.get_selected_history() as usize;
    let Some(record) = records.get(index) else {
        return Ok(());
    };

    ui.set_execution_status(status_label(record.status).into());
    ui.set_execution_exit_code(
        record
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "-".to_string())
            .into(),
    );
    ui.set_execution_output(format_output(&record.stdout, &record.stderr).into());
    Ok(())
}

fn refresh_profiles(ui: &AppWindow, store: &Store) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store.list_profiles(&active_workspace_id(store))?;
    let names: Vec<SharedString> = profiles
        .iter()
        .map(|profile| SharedString::from(profile.name.as_str()))
        .collect();

    let (range, info) = Pager::requested(ui.get_profile_page_info()).slice(names.len());
    ui.set_profile_names(ModelRc::new(VecModel::from(names[range].to_vec())));
    ui.set_profile_page_info(info);

    // O índice da seleção continua sendo a posição na lista inteira: a página é
    // recorte de exibição, não muda o significado do que está selecionado.
    let previous = ui.get_selected_profile();
    if profiles.is_empty() {
        ui.set_selected_profile(0);
    } else if (previous as usize) >= profiles.len() {
        ui.set_selected_profile((profiles.len() - 1) as i32);
    }
    Ok(())
}

fn prepare_create_profile_dialog(ui: &AppWindow, store: &Store) {
    let names: Vec<SharedString> = store
        .tools()
        .iter()
        .map(|tool| SharedString::from(tool.name.as_str()))
        .collect();
    ui.set_create_profile_tool_names(ModelRc::new(VecModel::from(names)));
    ui.set_create_profile_selected_tool(0);
    ui.set_create_profile_selected_command(0);
    ui.set_create_profile_name("".into());
    ui.set_create_profile_param_text("".into());
    refresh_create_profile_selection(ui, store);
}

fn refresh_create_profile_selection(ui: &AppWindow, store: &Store) {
    let tool_index = ui.get_create_profile_selected_tool() as usize;
    let Some(tool) = store.tools().get(tool_index) else {
        ui.set_create_profile_command_names(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
        ui.set_create_profile_show_param(false);
        return;
    };

    let names: Vec<SharedString> = tool
        .manifest
        .commands
        .iter()
        .map(|command| SharedString::from(command.name.as_str()))
        .collect();
    ui.set_create_profile_command_names(ModelRc::new(VecModel::from(names)));

    let mut command_index = ui.get_create_profile_selected_command() as usize;
    if command_index >= tool.manifest.commands.len() {
        command_index = 0;
        ui.set_create_profile_selected_command(0);
    }

    if let Some(command) = tool.manifest.commands.get(command_index) {
        bind_create_profile_parameter(ui, command);
    } else {
        ui.set_create_profile_show_param(false);
        ui.set_create_profile_param_text("".into());
    }
}

fn bind_create_profile_parameter(ui: &AppWindow, command: &Command) {
    if let Some(parameter) = first_string_parameter(command) {
        ui.set_create_profile_show_param(true);
        ui.set_create_profile_param_label(SharedString::from(parameter.name.as_str()));
        ui.set_create_profile_param_text(parameter.default.clone().unwrap_or_default().into());
    } else {
        ui.set_create_profile_show_param(false);
        ui.set_create_profile_param_label("Parâmetro".into());
        ui.set_create_profile_param_text("".into());
    }
}

fn save_profile_from_dialog(ui: &AppWindow, store: &Store) -> Result<String, String> {
    let name = ui.get_create_profile_name().trim().to_string();
    if name.is_empty() {
        return Err("Informe um nome para o perfil.".into());
    }

    let tool_index = ui.get_create_profile_selected_tool() as usize;
    let command_index = ui.get_create_profile_selected_command() as usize;
    let tool = store
        .tools()
        .get(tool_index)
        .ok_or_else(|| "Ferramenta inválida.".to_string())?;
    let command = tool
        .manifest
        .commands
        .get(command_index)
        .ok_or_else(|| "Comando inválido.".to_string())?;

    let profile = Profile {
        id: Profile::new_id(&name),
        name: name.clone(),
        tool_id: tool.id.clone(),
        command_id: command.id.clone(),
        parameters: collect_parameter_values(command, &ui.get_create_profile_param_text()),
        workdir: None,
    };

    store
        .save_profile(&active_workspace_id(store), &profile)
        .map_err(|err| err.to_string())?;
    Ok(name)
}

/// Carrega o perfil na tela da ferramenta: abre a ferramenta, seleciona a ação,
/// preenche o parâmetro e adota o Workdir do perfil.
fn apply_selected_profile(
    ui: &AppWindow,
    store: &Store,
    visible: &VisibleTools,
    detail_tool_id: &Rc<RefCell<Option<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = store.list_profiles(&active_workspace_id(store))?;
    let index = ui.get_selected_profile() as usize;
    let Some(profile) = profiles.get(index) else {
        return Ok(());
    };

    let Some(tool) = store.library().get(&profile.tool_id) else {
        ui.set_status_text(format!("Ferramenta `{}` não encontrada.", profile.tool_id).into());
        return Ok(());
    };

    // A ferramenta do perfil pode estar escondida pela busca ou pelo filtro.
    // Limpar os dois deixa a lista coerente para reencontrá-la depois.
    ui.set_tool_search_text("".into());
    ui.set_tool_workspace_only(false);
    refresh_tools(ui, store, visible);

    ui.set_selected_tool_name(SharedString::from(tool.name.as_str()));
    let version = store.current_manifest_version(&tool.id).unwrap_or(0);
    ui.set_manifest_version_text(SharedString::from(format!("definição v{version}")));
    ui.set_command_names(ModelRc::new(VecModel::from(
        tool.manifest
            .commands
            .iter()
            .map(|command| SharedString::from(command.name.as_str()))
            .collect::<Vec<_>>(),
    )));

    let command_index = tool
        .manifest
        .commands
        .iter()
        .position(|command| command.id == profile.command_id)
        .unwrap_or(0);
    ui.set_selected_command(command_index as i32);

    *detail_tool_id.borrow_mut() = Some(tool.id.clone());
    bind_parameter_field(ui, store, Some(&tool.id));

    if let Ok(command) = find_command(&tool.manifest, &profile.command_id) {
        if let Some(parameter) = first_string_parameter(command) {
            if let Some(value) = profile.parameters.get(&parameter.id) {
                ui.set_param_text(value.clone().into());
            }
        }
    }

    // O Workdir do perfil é o da ferramenta; o global não é tocado.
    match &profile.workdir {
        Some(workdir) => {
            ui.set_tool_workdir_text(SharedString::from(workdir.display().to_string()))
        }
        None => ui.set_tool_workdir_text(ui.get_workdir_text()),
    }

    ui.set_execution_status("-".into());
    ui.set_execution_exit_code("-".into());
    ui.set_execution_output("".into());

    if let Err(err) = refresh_tool_history(ui, store, &tool.id) {
        ui.set_status_text(err.to_string().into());
    }

    ui.set_tool_detail_open(true);
    ui.set_status_text(format!("Perfil `{}` carregado.", profile.name).into());
    Ok(())
}

/// Atualiza a lista de Workflows na UI.
///
/// `select_name` permite deixar selecionado o Workflow que acabou de ser criado,
/// em vez de manter o índice anterior.
fn refresh_workflows(
    ui: &AppWindow,
    store: &Store,
    select_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workflows = store.list_workflows()?;
    let names: Vec<SharedString> = workflows
        .iter()
        .map(|workflow| SharedString::from(workflow.name.as_str()))
        .collect();
    let previous = ui.get_selected_workflow();

    let mut pager = Pager::requested(ui.get_workflow_page_info());

    // O workflow recém-criado pode estar em outra página; mostrá-lo é o ponto de
    // tê-lo criado.
    let created = select_name
        .and_then(|name| workflows.iter().position(|workflow| workflow.name == name));
    if let Some(index) = created {
        pager.reveal(index);
    }

    let (range, info) = pager.slice(names.len());
    ui.set_workflow_names(ModelRc::new(VecModel::from(names[range].to_vec())));
    ui.set_workflow_page_info(info);

    if workflows.is_empty() {
        ui.set_selected_workflow(0);
        ui.set_workflow_steps_title("Etapas".into());
        set_workflow_step_results(ui, Vec::<String>::new());
        return Ok(());
    }

    if let Some(index) = created {
        ui.set_selected_workflow(index as i32);
    } else if (previous as usize) >= workflows.len() {
        ui.set_selected_workflow((workflows.len() - 1) as i32);
    }

    // Manter a área de etapas coerente com o Workflow que está selecionado.
    let _ = apply_selected_workflow(ui, store);
    Ok(())
}

fn refresh_workflow_history(
    ui: &AppWindow,
    store: &Store,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = store.list_recent_workflow_executions(&active_workspace_id(store), 20)?;
    let labels: Vec<SharedString> = records
        .iter()
        .map(|record| SharedString::from(record.label()))
        .collect();

    // A execução recém-gravada é a mais recente: começar por ela é o que se espera
    // depois de executar.
    let mut pager = Pager::requested(ui.get_workflow_history_page_info());
    pager.first_page();

    let (range, info) = pager.slice(labels.len());
    ui.set_workflow_history_labels(ModelRc::new(VecModel::from(labels[range].to_vec())));
    ui.set_workflow_history_page_info(info);
    if !records.is_empty() {
        ui.set_selected_workflow_history(0);
    }
    Ok(())
}

/// Mostra as etapas do Workflow selecionado (definição) e devolve o nome dele.
fn apply_selected_workflow(ui: &AppWindow, store: &Store) -> Result<String, String> {
    let workflows = store.list_workflows().map_err(|err| err.to_string())?;
    let index = ui.get_selected_workflow() as usize;
    let workflow = workflows
        .get(index)
        .ok_or_else(|| "Selecione um workflow.".to_string())?;

    let labels = workflow
        .steps
        .iter()
        .enumerate()
        .map(|(position, step)| workflow_step_definition_label(position, step));

    ui.set_workflow_steps_title("Etapas do workflow selecionado".into());
    set_workflow_step_results(ui, labels);
    Ok(workflow.name.clone())
}

/// Mostra as etapas da execução selecionada e devolve o rótulo dela.
fn apply_selected_workflow_history(
    ui: &AppWindow,
    store: &Store,
) -> Result<String, Box<dyn std::error::Error>> {
    let records = store.list_recent_workflow_executions(&active_workspace_id(store), 20)?;
    let index = ui.get_selected_workflow_history() as usize;
    let Some(record) = records.get(index) else {
        return Ok(String::new());
    };

    ui.set_workflow_steps_title("Etapas da execução".into());
    set_workflow_step_results(ui, record.steps.iter().map(|step| step.label()));
    Ok(record.label())
}

fn set_workflow_step_results<I>(ui: &AppWindow, labels: I)
where
    I: IntoIterator<Item = String>,
{
    let labels: Vec<SharedString> = labels.into_iter().map(SharedString::from).collect();
    ui.set_workflow_step_results(ModelRc::new(VecModel::from(labels)));
}

/// Traduz uma etapa gravada no Workflow em definição executável (Tool + Command).
fn resolve_workflow_step(
    store: &Store,
    step: &WorkflowStep,
) -> Result<(Manifest, Command), WorkflowError> {
    let tool = store
        .library()
        .get(&step.tool_id)
        .ok_or_else(|| WorkflowError::UnknownTool(step.tool_id.clone()))?;
    let command = find_command(&tool.manifest, &step.command_id)?.clone();
    Ok((tool.manifest.clone(), command))
}

fn run_selected_workflow(
    ui: &AppWindow,
    store: &Store,
    runtime: &tokio::runtime::Runtime,
) -> Result<String, String> {
    let workflows = store.list_workflows().map_err(|err| err.to_string())?;
    let index = ui.get_selected_workflow() as usize;
    let workflow = workflows
        .get(index)
        .ok_or_else(|| "Selecione um workflow.".to_string())?
        .clone();

    let workdir = PathBuf::from(ui.get_workdir_text().as_str());
    let context = ExecutionContext {
        workdir: if workdir.as_os_str().is_empty() {
            None
        } else {
            Some(workdir)
        },
    };

    let outcome = runtime
        .block_on(execute_workflow(&workflow, &context, |step| {
            resolve_workflow_step(store, step)
        }))
        .map_err(|err| err.to_string())?;

    let summary = if outcome.success() {
        format!(
            "Workflow `{}` concluído · {} etapa(s).",
            workflow.name,
            outcome.steps.len()
        )
    } else {
        format!(
            "Workflow `{}` falhou · {} falha(s) em {} etapa(s).",
            workflow.name,
            outcome.failed_steps(),
            outcome.steps.len()
        )
    };

    store
        .record_workflow_execution(&active_workspace_id(store), &outcome)
        .map_err(|err| format!("{summary} (histórico: {err})"))?;

    Ok(summary)
}

fn prepare_create_workflow_dialog(ui: &AppWindow, store: &Store, draft: &mut Vec<WorkflowStep>) {
    draft.clear();

    let names: Vec<SharedString> = store
        .tools()
        .iter()
        .map(|tool| SharedString::from(tool.name.as_str()))
        .collect();
    ui.set_create_workflow_tool_names(ModelRc::new(VecModel::from(names)));
    ui.set_create_workflow_selected_tool(0);
    ui.set_create_workflow_selected_command(0);
    ui.set_create_workflow_selected_on_error(0);
    ui.set_create_workflow_name("".into());
    ui.set_create_workflow_param_text("".into());
    ui.set_create_workflow_step_error("".into());
    // Começar a criação fecha a etapa em construção, se ficou aberta da vez
    // anterior, e zera o Workdir dela: são dados do rascunho que foi descartado.
    ui.set_create_workflow_step_open(false);
    ui.set_create_workflow_step_workdir("".into());
    set_workflow_step_labels(ui, draft);
    refresh_create_workflow_selection(ui, store);
}

fn refresh_create_workflow_selection(ui: &AppWindow, store: &Store) {
    let tool_index = ui.get_create_workflow_selected_tool() as usize;
    let Some(tool) = store.tools().get(tool_index) else {
        ui.set_create_workflow_command_names(ModelRc::new(VecModel::from(
            Vec::<SharedString>::new(),
        )));
        ui.set_create_workflow_show_param(false);
        return;
    };

    let names: Vec<SharedString> = tool
        .manifest
        .commands
        .iter()
        .map(|command| SharedString::from(command.name.as_str()))
        .collect();
    ui.set_create_workflow_command_names(ModelRc::new(VecModel::from(names)));

    let mut command_index = ui.get_create_workflow_selected_command() as usize;
    if command_index >= tool.manifest.commands.len() {
        command_index = 0;
        ui.set_create_workflow_selected_command(0);
    }

    if let Some(command) = tool.manifest.commands.get(command_index) {
        bind_create_workflow_parameter(ui, command);
    } else {
        ui.set_create_workflow_show_param(false);
        ui.set_create_workflow_param_text("".into());
    }
}

fn bind_create_workflow_parameter(ui: &AppWindow, command: &Command) {
    if let Some(parameter) = first_string_parameter(command) {
        ui.set_create_workflow_show_param(true);
        ui.set_create_workflow_param_label(SharedString::from(parameter.name.as_str()));
        ui.set_create_workflow_param_text(parameter.default.clone().unwrap_or_default().into());
    } else {
        ui.set_create_workflow_show_param(false);
        ui.set_create_workflow_param_label("Parâmetro".into());
        ui.set_create_workflow_param_text("".into());
    }
}

fn add_workflow_step(ui: &AppWindow, store: &Store, draft: &mut Vec<WorkflowStep>) {
    let tool_index = ui.get_create_workflow_selected_tool() as usize;
    let command_index = ui.get_create_workflow_selected_command() as usize;

    let Some(tool) = store.tools().get(tool_index) else {
        ui.set_create_workflow_step_error("Ferramenta inválida.".into());
        return;
    };
    let Some(command) = tool.manifest.commands.get(command_index) else {
        ui.set_create_workflow_step_error("Comando inválido.".into());
        return;
    };

    // Índices fixos espelhados no diálogo: 0 = parar, 1 = continuar.
    let on_error = match ui.get_create_workflow_selected_on_error() {
        1 => StepFailurePolicy::Continue,
        _ => StepFailurePolicy::Stop,
    };

    // Vazio herda o Workdir do contexto de execução (`Some` só quando há caminho).
    let workdir = {
        let text = ui.get_create_workflow_step_workdir().trim().to_string();
        (!text.is_empty()).then(|| std::path::PathBuf::from(text))
    };

    draft.push(WorkflowStep {
        id: format!("passo-{}", draft.len() + 1),
        tool_id: tool.id.clone(),
        command_id: command.id.clone(),
        parameters: collect_parameter_values(command, &ui.get_create_workflow_param_text()),
        workdir,
        on_error,
    });

    ui.set_create_workflow_step_error("".into());
    ui.set_create_workflow_step_workdir("".into());
    // A etapa entrou: o modal fecha e o campo de parâmetro volta ao padrão, para
    // a próxima etapa não nascer com o valor da anterior já preenchido.
    ui.set_create_workflow_step_open(false);
    refresh_create_workflow_selection(ui, store);
    set_workflow_step_labels(ui, draft);
}

fn set_workflow_step_labels(ui: &AppWindow, draft: &[WorkflowStep]) {
    let labels: Vec<SharedString> = draft
        .iter()
        .enumerate()
        .map(|(index, step)| SharedString::from(workflow_step_definition_label(index, step)))
        .collect();
    ui.set_create_workflow_step_labels(ModelRc::new(VecModel::from(labels)));
}

/// Rótulo de uma etapa enquanto definição: posição, ferramenta, ação, parâmetro,
/// workdir próprio (quando há) e política.
///
/// Vale para a etapa em construção no diálogo e para a etapa já salva — é a
/// mesma informação, então não deve existir em duas formatações.
fn workflow_step_definition_label(position: usize, step: &WorkflowStep) -> String {
    let policy = match step.on_error {
        StepFailurePolicy::Stop => "parar se falhar",
        StepFailurePolicy::Continue => "continuar se falhar",
    };
    let mut parameter = step
        .parameters
        .values()
        .find(|value| !value.is_empty())
        .cloned()
        .unwrap_or_default();
    if !parameter.is_empty() {
        parameter = format!(" `{parameter}`");
    }

    // O workdir próprio muda onde a etapa roda: se está definido, aparece. Sem
    // ele a etapa herda o Workdir global, e dizer "herdado" em toda linha seria
    // ruído no caso comum.
    let workdir = match &step.workdir {
        Some(path) => format!(" · em `{}`", path.display()),
        None => String::new(),
    };

    format!(
        "{}. {} {}{} · {policy}{workdir}",
        position + 1,
        step.tool_id,
        step.command_id,
        parameter
    )
}

fn save_workflow_from_dialog(
    ui: &AppWindow,
    store: &Store,
    draft: &[WorkflowStep],
) -> Result<String, String> {
    let name = ui.get_create_workflow_name().trim().to_string();
    if name.is_empty() {
        return Err("Informe um nome para o workflow.".into());
    }
    if draft.is_empty() {
        return Err("Adicione ao menos uma etapa.".into());
    }

    let workflow = Workflow {
        id: Workflow::new_id(&name),
        name: name.clone(),
        description: None,
        steps: draft.to_vec(),
    };

    store
        .save_workflow(&workflow)
        .map_err(|err| err.to_string())?;
    Ok(name)
}

fn first_string_parameter(command: &Command) -> Option<&uige_core::Parameter> {
    command
        .parameters
        .iter()
        .find(|parameter| matches!(parameter.param_type, ParameterType::String))
}

fn collect_parameter_values(command: &Command, param_text: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    if let Some(parameter) = first_string_parameter(command) {
        let value = if param_text.is_empty() {
            parameter.default.clone().unwrap_or_default()
        } else {
            param_text.to_string()
        };
        values.insert(parameter.id.clone(), value);
    }
    values
}

fn format_output(stdout: &str, stderr: &str) -> String {
    let full = match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "(sem output)".to_string(),
        (false, true) => stdout.to_string(),
        (true, false) => format!("[stderr]\n{stderr}"),
        (false, false) => format!("{stdout}\n\n[stderr]\n{stderr}"),
    };
    last_n_lines(&full, 5)
}

/// Preview curto para a UI — o histórico no SQLite mantém o output completo.
fn last_n_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= n {
        return text.trim_end().to_string();
    }
    lines[lines.len() - n..].join("\n")
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn default_database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = dirs_next_data_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join("uige").join("uige.sqlite"))
}

fn dirs_next_data_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg));
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| PathBuf::from(home).join(".local").join("share"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Estado publicado com apenas a página pedida: é só isso que o recorte lê.
    fn requested(page: i32) -> PageInfo {
        page_request(
            PageInfo {
                page: 0,
                page_count: 0,
                total: 0,
                offset: 0,
            },
            page,
        )
    }

    #[test]
    fn a_short_list_has_a_single_page() {
        let (range, info) = Pager::requested(requested(0)).slice(3);
        assert_eq!(range, 0..3);
        assert_eq!(info.page_count, 1);
        assert_eq!(info.offset, 0);
    }

    #[test]
    fn pages_cover_the_list_without_overlap() {
        // 25 itens em páginas de 10: 10, 10 e 5.
        let total = 25;
        let mut covered = Vec::new();
        for page in 0..page_count(total) as i32 {
            let (range, _) = Pager::requested(requested(page)).slice(total);
            assert_eq!(range.len(), if page == 2 { 5 } else { PAGE_SIZE });
            covered.extend(range);
        }
        assert_eq!(covered, (0..total).collect::<Vec<_>>());
    }

    #[test]
    fn a_page_past_the_end_is_clamped_to_the_last_one() {
        // A lista encolheu (busca, remoção) depois de a pessoa ir para o fim:
        // mostrar página vazia pareceria lista sem resultado.
        let (range, info) = Pager::requested(requested(9)).slice(12);
        assert_eq!(info.page, 1);
        assert_eq!(range, 10..12);
    }

    #[test]
    fn revealing_an_index_moves_to_the_page_that_contains_it() {
        // O workflow recém-criado está na terceira página: criá-lo e não vê-lo
        // seria o mesmo que não ter criado.
        let mut pager = Pager::requested(requested(0));
        pager.reveal(21);
        let (range, info) = pager.slice(30);
        assert_eq!(info.page, 2);
        assert_eq!(range, 20..30);
    }

    #[test]
    fn an_empty_list_has_no_pages() {
        let (range, info) = Pager::requested(requested(0)).slice(0);
        assert!(range.is_empty());
        assert_eq!(info.page_count, 0);
    }
}
