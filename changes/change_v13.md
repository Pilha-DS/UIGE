# Mudança

A janela principal passa a abrir centralizada no monitor primário.

# Arquivos

- `software/crates/app/Cargo.toml`
- `software/crates/app/src/main.rs`
- `changes/change_v13.md`

# Motivo

O Slint delega o posicionamento inicial ao window manager e não expõe API estável para centralizar, então a janela abria no canto superior esquerdo.

# Impacto

`uige` habilita a feature `unstable-winit-030` do Slint e, após o event loop iniciar, calcula o centro usando `primary_monitor()` + `outer_size()` e aplica `set_outer_position()`. A tentativa é repetida por um `Timer` curto (até ~1s) porque o window do winit ainda não existe no momento do `AppWindow::new()`. Sem monitor detectável, a janela mantém o comportamento padrão.

Nota: a feature é marcada como *unstable* pelo Slint e pode mudar em releases futuros.
