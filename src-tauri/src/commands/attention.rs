use crate::attention::{
    models::{AttentionFilters, AttentionItem},
    repository,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_attention_items(
    filters: Option<AttentionFilters>,
    state: State<'_, AppState>,
) -> Result<Vec<AttentionItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let filters = filters.unwrap_or_default();
    let items = repository::list_attention_items(&conn, &filters)?;

    // Reorder by role once the user has confirmed one. An unconfirmed or
    // missing profile leaves the repository's severity/recency order alone.
    let profile = match crate::role::repository::get_user_profile(&conn) {
        Ok(p) => p,
        Err(_) => return Ok(items),
    };
    if !profile.role_confirmed {
        return Ok(items);
    }
    let role = match profile.inferred_role.clone() {
        Some(r) => r,
        None => return Ok(items),
    };

    crate::role::ordering::order_activity_items(&conn, items, &role, &profile)
}

#[tauri::command]
pub async fn get_attention_count(state: State<'_, AppState>) -> Result<(u32, u32), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_attention_count(&conn)
}

#[tauri::command]
pub async fn dismiss_attention_item(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::dismiss_attention_item(&conn, &id)
}
