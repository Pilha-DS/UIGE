//! Identificadores gerados para entidades nomeadas pelo usuário.
//!
//! O id precisa ser estável e único, mas não é a identidade visível: o usuário
//! vê o nome.

/// Gera `{prefix}-{timestamp}-{slug}` a partir de um nome.
pub(crate) fn slug_id(prefix: &str, name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    format!("{prefix}-{stamp}-{slug}")
}
