// Tauri application entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scripted_prompt_lib::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

// Application state
struct AppState {
    storage: Mutex<Storage>,
    scripts: Mutex<Vec<Script>>,
    templates: Mutex<Vec<Template>>,
    history: Mutex<HistoryManager>,
    tags: Mutex<TagManager>,
    clipboard: ClipboardManager,
}

// Tauri commands

#[tauri::command]
fn get_all_scripts(state: State<AppState>) -> Result<Vec<Script>, String> {
    let scripts = state.scripts.lock().map_err(|e| e.to_string())?;
    Ok(scripts.clone())
}

#[tauri::command]
fn get_all_templates(state: State<AppState>) -> Result<Vec<Template>, String> {
    let templates = state.templates.lock().map_err(|e| e.to_string())?;
    Ok(templates.clone())
}

#[tauri::command]
fn search_scripts(
    query: String,
    tag_filter: Option<String>,
    state: State<AppState>,
) -> Result<Vec<Script>, String> {
    let scripts = state.scripts.lock().map_err(|e| e.to_string())?;

    let results = fuzzy_search_scripts(
        &scripts,
        &query,
        tag_filter.as_deref(),
    );

    Ok(results.into_iter().cloned().collect())
}

#[tauri::command]
fn search_templates(
    query: String,
    tag_filter: Option<String>,
    state: State<AppState>,
) -> Result<Vec<Template>, String> {
    let templates = state.templates.lock().map_err(|e| e.to_string())?;

    let results = fuzzy_search_templates(
        &templates,
        &query,
        tag_filter.as_deref(),
    );

    Ok(results.into_iter().cloned().collect())
}

#[tauri::command]
fn create_script(
    name: String,
    tags: String,
    content: String,
    state: State<AppState>,
) -> Result<Script, String> {
    eprintln!("[create_script] Creating new script: '{}'", name);
    eprintln!("[create_script]   - Tags: {}", tags);
    eprintln!("[create_script]   - Content length: {} bytes", content.len());

    let script = Script::new(name, tags, content);
    eprintln!("[create_script] Generated script ID: {}", script.id);

    let mut scripts = state.scripts.lock().map_err(|e| {
        eprintln!("[create_script] Failed to lock scripts: {}", e);
        e.to_string()
    })?;
    scripts.push(script.clone());
    eprintln!("[create_script] Added script to list (total: {})", scripts.len());

    // Update tags
    let mut tag_manager = state.tags.lock().map_err(|e| {
        eprintln!("[create_script] Failed to lock tags: {}", e);
        e.to_string()
    })?;
    tag_manager.update_from_scripts(&scripts);
    eprintln!("[create_script] Updated tags");

    // Save to storage
    drop(scripts);
    drop(tag_manager);

    eprintln!("[create_script] Saving data to disk...");
    save_all_data(&state).map_err(|e| {
        eprintln!("[create_script] Failed to save data: {}", e);
        e
    })?;

    eprintln!("[create_script] Successfully created script '{}' with ID: {}", script.name, script.id);
    Ok(script)
}

#[tauri::command]
fn update_script(
    id: String,
    name: Option<String>,
    tags: Option<String>,
    content: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    eprintln!("[update_script] Starting update for script ID: {}", id);

    let mut scripts = state.scripts.lock().map_err(|e| {
        eprintln!("[update_script] Failed to lock scripts: {}", e);
        e.to_string()
    })?;

    let script = scripts.iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| {
            eprintln!("[update_script] Script not found: {}", id);
            "Script not found".to_string()
        })?;

    let script_name = script.name.clone();
    eprintln!("[update_script] Updating script '{}'", script_name);

    if let Some(ref n) = name {
        eprintln!("[update_script]   - Name: {} -> {}", script_name, n);
    }
    if let Some(ref t) = tags {
        eprintln!("[update_script]   - Tags: {}", t);
    }
    if content.is_some() {
        eprintln!("[update_script]   - Content updated");
    }

    script.update(name, tags, content);

    // Update tags
    let mut tag_manager = state.tags.lock().map_err(|e| {
        eprintln!("[update_script] Failed to lock tags: {}", e);
        e.to_string()
    })?;
    tag_manager.update_from_scripts(&scripts);
    eprintln!("[update_script] Updated tags");

    drop(scripts);
    drop(tag_manager);

    eprintln!("[update_script] Saving data to disk...");
    save_all_data(&state).map_err(|e| {
        eprintln!("[update_script] Failed to save data: {}", e);
        e
    })?;

    eprintln!("[update_script] Successfully updated script '{}'", script_name);
    Ok(())
}

#[tauri::command]
fn delete_script(id: String, state: State<AppState>) -> Result<(), String> {
    eprintln!("[delete_script] Starting deletion for script ID: {}", id);

    // Lock scripts and find the script to delete
    let mut scripts = state.scripts.lock().map_err(|e| {
        eprintln!("[delete_script] Failed to lock scripts: {}", e);
        e.to_string()
    })?;

    let index = scripts.iter()
        .position(|s| s.id == id)
        .ok_or_else(|| {
            eprintln!("[delete_script] Script not found: {}", id);
            "Script not found".to_string()
        })?;

    let script_name = scripts[index].name.clone();
    scripts.remove(index);
    eprintln!("[delete_script] Removed script '{}' from scripts list", script_name);

    // Clean up template references to this script
    {
        let mut templates = state.templates.lock().map_err(|e| {
            eprintln!("[delete_script] Failed to lock templates: {}", e);
            e.to_string()
        })?;

        let mut affected_templates = 0;
        for template in templates.iter_mut() {
            let before_count = template.script_ids.len();
            template.remove_script_id(&id);
            let after_count = template.script_ids.len();

            if before_count != after_count {
                affected_templates += 1;
                eprintln!("[delete_script] Removed script reference from template '{}'", template.name);
            }
        }
        eprintln!("[delete_script] Cleaned {} template(s)", affected_templates);
    }

    // Update tags
    let mut tag_manager = state.tags.lock().map_err(|e| {
        eprintln!("[delete_script] Failed to lock tags: {}", e);
        e.to_string()
    })?;
    tag_manager.update_from_scripts(&scripts);
    eprintln!("[delete_script] Updated tags");

    drop(scripts);
    drop(tag_manager);

    // Save all data to disk
    eprintln!("[delete_script] Saving data to disk...");
    save_all_data(&state).map_err(|e| {
        eprintln!("[delete_script] Failed to save data: {}", e);
        e
    })?;

    eprintln!("[delete_script] Successfully deleted script '{}' with ID: {}", script_name, id);
    Ok(())
}

#[tauri::command]
fn create_template(
    name: String,
    tags: String,
    script_ids: Vec<String>,
    variable_values: HashMap<String, String>,
    state: State<AppState>,
) -> Result<Template, String> {
    let template = Template::new(name, tags, script_ids, variable_values);

    let mut templates = state.templates.lock().map_err(|e| e.to_string())?;
    templates.push(template.clone());

    // Update tags
    let mut tag_manager = state.tags.lock().map_err(|e| e.to_string())?;
    tag_manager.update_from_templates(&templates);

    drop(templates);
    drop(tag_manager);
    save_all_data(&state)?;

    Ok(template)
}

#[tauri::command]
fn update_template(
    id: String,
    name: Option<String>,
    tags: Option<String>,
    script_ids: Option<Vec<String>>,
    variable_values: Option<HashMap<String, String>>,
    state: State<AppState>,
) -> Result<(), String> {
    let mut templates = state.templates.lock().map_err(|e| e.to_string())?;

    let template = templates.iter_mut()
        .find(|t| t.id == id)
        .ok_or("Template not found")?;

    template.update(name, tags, script_ids, variable_values);

    // Update tags
    let mut tag_manager = state.tags.lock().map_err(|e| e.to_string())?;
    tag_manager.update_from_templates(&templates);

    drop(templates);
    drop(tag_manager);
    save_all_data(&state)?;

    Ok(())
}

#[tauri::command]
fn delete_template(id: String, state: State<AppState>) -> Result<(), String> {
    eprintln!("[delete_template] Starting deletion for template ID: {}", id);

    // Lock templates and find the template to delete
    let mut templates = state.templates.lock().map_err(|e| {
        eprintln!("[delete_template] Failed to lock templates: {}", e);
        e.to_string()
    })?;

    let index = templates.iter()
        .position(|t| t.id == id)
        .ok_or_else(|| {
            eprintln!("[delete_template] Template not found: {}", id);
            "Template not found".to_string()
        })?;

    let template_name = templates[index].name.clone();
    templates.remove(index);
    eprintln!("[delete_template] Removed template '{}' from templates list", template_name);

    // Update tags
    let mut tag_manager = state.tags.lock().map_err(|e| {
        eprintln!("[delete_template] Failed to lock tags: {}", e);
        e.to_string()
    })?;
    tag_manager.update_from_templates(&templates);
    eprintln!("[delete_template] Updated tags");

    drop(templates);
    drop(tag_manager);

    // Save all data to disk
    eprintln!("[delete_template] Saving data to disk...");
    save_all_data(&state).map_err(|e| {
        eprintln!("[delete_template] Failed to save data: {}", e);
        e
    })?;

    eprintln!("[delete_template] Successfully deleted template '{}' with ID: {}", template_name, id);
    Ok(())
}

#[tauri::command]
fn get_all_tags(state: State<AppState>) -> Result<Vec<(String, usize)>, String> {
    let tag_manager = state.tags.lock().map_err(|e| e.to_string())?;
    Ok(tag_manager.get_all_tags())
}

#[tauri::command]
fn replace_script_variables(
    content: String,
    variables: HashMap<String, String>,
) -> Result<String, String> {
    Ok(replace_variables(&content, &variables, |var| format!("{{{{{}}}}}", var)))
}

#[tauri::command]
fn parse_script_variables(content: String) -> Result<Vec<String>, String> {
    Ok(parse_variables(&content))
}

#[tauri::command]
fn parse_variables_with_defaults_command(content: String) -> Result<Vec<Variable>, String> {
    Ok(parse_variables_with_defaults(&content))
}

#[derive(Serialize)]
struct VariableUsageInfo {
    variable_name: String,
    script_names: Vec<String>, // 使用该变量的 Script 名称列表
}

#[derive(Serialize)]
struct TagRenamePreview {
    script_tags: Vec<String>,
    script_count: usize,
    template_tags: Vec<String>,
    template_count: usize,
}

#[derive(Serialize)]
struct TagDeletePreview {
    script_tags: Vec<String>,
    script_count: usize,
    template_tags: Vec<String>,
    template_count: usize,
}

#[tauri::command]
fn get_tag_rename_preview(
    old_segment: String,
    state: State<AppState>,
) -> Result<TagRenamePreview, String> {
    let scripts = state.scripts.lock().map_err(|e| e.to_string())?;
    let templates = state.templates.lock().map_err(|e| e.to_string())?;

    let script_arcs: Vec<_> = scripts.iter().cloned().map(std::sync::Arc::new).collect();
    let template_arcs: Vec<_> = templates.iter().cloned().map(std::sync::Arc::new).collect();

    let tag_manager = state.tags.lock().map_err(|e| e.to_string())?;
    let impact = tag_manager.calculate_rename_impact(&old_segment, &script_arcs, &template_arcs);

    let mut script_tags: Vec<_> = impact.script_tags.into_iter().collect();
    script_tags.sort();
    let mut template_tags: Vec<_> = impact.template_tags.into_iter().collect();
    template_tags.sort();

    Ok(TagRenamePreview {
        script_tags,
        script_count: impact.script_count,
        template_tags,
        template_count: impact.template_count,
    })
}

#[tauri::command]
fn rename_tag_segment_command(
    old_segment: String,
    new_segment: String,
    state: State<AppState>,
) -> Result<(), String> {
    let trimmed_old = old_segment.trim();
    let trimmed_new = new_segment.trim();

    if trimmed_old.is_empty() || trimmed_new.is_empty() {
        return Err("Tag segment cannot be empty".to_string());
    }

    let mut scripts = state.scripts.lock().map_err(|e| e.to_string())?;
    let mut templates = state.templates.lock().map_err(|e| e.to_string())?;
    let mut tag_manager = state.tags.lock().map_err(|e| e.to_string())?;

    let mut script_arcs: Vec<_> = scripts.iter().cloned().map(std::sync::Arc::new).collect();
    let mut template_arcs: Vec<_> = templates.iter().cloned().map(std::sync::Arc::new).collect();

    let changed = tag_manager.rename_tag_segment(trimmed_old, trimmed_new, &mut script_arcs, &mut template_arcs);
    if !changed {
        return Err("No matching tags found to rename".to_string());
    }

    *scripts = script_arcs.into_iter().map(|s| (*s).clone()).collect();
    *templates = template_arcs.into_iter().map(|t| (*t).clone()).collect();

    drop(scripts);
    drop(templates);
    drop(tag_manager);

    save_all_data(&state)?;
    Ok(())
}

#[tauri::command]
fn get_tag_delete_preview(
    tag_path: String,
    state: State<AppState>,
) -> Result<TagDeletePreview, String> {
    let scripts = state.scripts.lock().map_err(|e| e.to_string())?;
    let templates = state.templates.lock().map_err(|e| e.to_string())?;

    let script_arcs: Vec<_> = scripts.iter().cloned().map(std::sync::Arc::new).collect();
    let template_arcs: Vec<_> = templates.iter().cloned().map(std::sync::Arc::new).collect();

    let tag_manager = state.tags.lock().map_err(|e| e.to_string())?;
    let impact = tag_manager.calculate_delete_impact(&tag_path, &script_arcs, &template_arcs);

    let mut script_tags = impact.script_tags;
    script_tags.sort();
    let mut template_tags = impact.template_tags;
    template_tags.sort();

    Ok(TagDeletePreview {
        script_tags,
        script_count: impact.script_count,
        template_tags,
        template_count: impact.template_count,
    })
}

#[tauri::command]
fn cascade_delete_tag_command(tag_path: String, state: State<AppState>) -> Result<(), String> {
    let trimmed_tag = tag_path.trim();
    if trimmed_tag.is_empty() {
        return Err("Tag path cannot be empty".to_string());
    }

    let mut scripts = state.scripts.lock().map_err(|e| e.to_string())?;
    let mut templates = state.templates.lock().map_err(|e| e.to_string())?;
    let mut tag_manager = state.tags.lock().map_err(|e| e.to_string())?;

    let mut script_arcs: Vec<_> = scripts.iter().cloned().map(std::sync::Arc::new).collect();
    let mut template_arcs: Vec<_> = templates.iter().cloned().map(std::sync::Arc::new).collect();

    let changed = tag_manager.cascade_delete_tag(trimmed_tag, &mut script_arcs, &mut template_arcs);
    if !changed {
        return Err("No matching tag branch found to delete".to_string());
    }

    *scripts = script_arcs.into_iter().map(|s| (*s).clone()).collect();
    *templates = template_arcs.into_iter().map(|t| (*t).clone()).collect();

    drop(scripts);
    drop(templates);
    drop(tag_manager);

    save_all_data(&state)?;
    Ok(())
}

#[tauri::command]
fn get_variable_usage_locations(
    script_ids: Vec<String>,
    state: State<AppState>,
) -> Result<Vec<VariableUsageInfo>, String> {
    let scripts = state.scripts.lock().map_err(|e| e.to_string())?;

    // 找到所有相关的 scripts
    let selected_scripts: Vec<_> = scripts
        .iter()
        .filter(|s| script_ids.contains(&s.id))
        .collect();

    // 收集所有变量及其使用位置
    let mut usage_map: HashMap<String, Vec<String>> = HashMap::new();

    for script in selected_scripts {
        let variables = parse_variables(&script.content);
        for var_name in variables {
            usage_map
                .entry(var_name)
                .or_insert_with(Vec::new)
                .push(script.name.clone());
        }
    }

    // 转换为结果格式
    let mut result: Vec<VariableUsageInfo> = usage_map
        .into_iter()
        .map(|(variable_name, script_names)| VariableUsageInfo {
            variable_name,
            script_names,
        })
        .collect();

    // 按变量名排序以保持一致性
    result.sort_by(|a, b| a.variable_name.cmp(&b.variable_name));

    Ok(result)
}

#[tauri::command]
fn copy_script_to_clipboard(script_id: String, text: String, state: State<AppState>) -> Result<(), String> {
    match state.clipboard.copy_text(&text) {
        utils::clipboard::ClipboardResult::Success => {
            let mut scripts = state.scripts.lock().map_err(|e| e.to_string())?;
            let script = scripts.iter_mut()
                .find(|script| script.id == script_id)
                .ok_or("Script not found")?;

            script.increment_use_count();

            let mut history = state.history.lock().map_err(|e| e.to_string())?;
            history.record_script_usage(script, None);

            drop(history);
            drop(scripts);
            save_all_data(&state)?;
            Ok(())
        }
        utils::clipboard::ClipboardResult::Failure(e) => Err(e),
        utils::clipboard::ClipboardResult::Unsupported => Err("Clipboard not supported".to_string()),
    }
}

#[tauri::command]
fn copy_template_preview_to_clipboard(template_id: String, text: String, state: State<AppState>) -> Result<(), String> {
    match state.clipboard.copy_text(&text) {
        utils::clipboard::ClipboardResult::Success => {
            let mut templates = state.templates.lock().map_err(|e| e.to_string())?;
            let template = templates.iter_mut()
                .find(|template| template.id == template_id)
                .ok_or("Template not found")?;

            template.increment_use_count();

            let mut history = state.history.lock().map_err(|e| e.to_string())?;
            history.record_template_usage(template, None);

            drop(history);
            drop(templates);
            save_all_data(&state)?;
            Ok(())
        }
        utils::clipboard::ClipboardResult::Failure(e) => Err(e),
        utils::clipboard::ClipboardResult::Unsupported => Err("Clipboard not supported".to_string()),
    }
}

#[tauri::command]
fn copy_to_clipboard(text: String, state: State<AppState>) -> Result<(), String> {
    match state.clipboard.copy_text(&text) {
        utils::clipboard::ClipboardResult::Success => Ok(()),
        utils::clipboard::ClipboardResult::Failure(e) => Err(e),
        utils::clipboard::ClipboardResult::Unsupported => Err("Clipboard not supported".to_string()),
    }
}

#[tauri::command]
fn toggle_favorite_script(id: String, state: State<AppState>) -> Result<(), String> {
    let mut scripts = state.scripts.lock().map_err(|e| e.to_string())?;

    let script = scripts.iter_mut()
        .find(|s| s.id == id)
        .ok_or("Script not found")?;

    script.is_favorite = !script.is_favorite;

    drop(scripts);
    save_all_data(&state)?;

    Ok(())
}

#[tauri::command]
fn toggle_favorite_template(id: String, state: State<AppState>) -> Result<(), String> {
    let mut templates = state.templates.lock().map_err(|e| e.to_string())?;

    let template = templates.iter_mut()
        .find(|t| t.id == id)
        .ok_or("Template not found")?;

    template.is_favorite = !template.is_favorite;

    drop(templates);
    save_all_data(&state)?;

    Ok(())
}

#[tauri::command]
fn export_data(export_path: String, state: State<AppState>) -> Result<(), String> {
    eprintln!("[export_data] Starting export to: {}", export_path);

    let storage = state.storage.lock().map_err(|e| e.to_string())?;

    storage.export_all(&export_path).map_err(|e| {
        eprintln!("[export_data] Export failed: {}", e);
        e.to_string()
    })?;

    eprintln!("[export_data] Export completed successfully");
    Ok(())
}

#[tauri::command]
fn import_data(import_path: String, state: State<AppState>) -> Result<(), String> {
    eprintln!("[import_data] Starting import from: {}", import_path);

    let storage = state.storage.lock().map_err(|e| e.to_string())?;

    storage.import_all(&import_path).map_err(|e| {
        eprintln!("[import_data] Import failed: {}", e);
        e.to_string()
    })?;

    // Reload data into state
    drop(storage);

    eprintln!("[import_data] Reloading data into state...");
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let (scripts, templates, history, tags) = storage.load_all().map_err(|e| e.to_string())?;

    let mut scripts_state = state.scripts.lock().map_err(|e| e.to_string())?;
    let mut templates_state = state.templates.lock().map_err(|e| e.to_string())?;
    let mut history_state = state.history.lock().map_err(|e| e.to_string())?;
    let mut tags_state = state.tags.lock().map_err(|e| e.to_string())?;

    *scripts_state = scripts;
    *templates_state = templates;
    *history_state = history;
    *tags_state = tags;

    eprintln!("[import_data] Import completed successfully");
    Ok(())
}

// Helper function to save all data
fn save_all_data(state: &State<AppState>) -> Result<(), String> {
    eprintln!("[save_all_data] Acquiring locks...");

    let storage = state.storage.lock().map_err(|e| {
        let err_msg = format!("Failed to lock storage: {}", e);
        eprintln!("[save_all_data] {}", err_msg);
        err_msg
    })?;

    let scripts = state.scripts.lock().map_err(|e| {
        let err_msg = format!("Failed to lock scripts: {}", e);
        eprintln!("[save_all_data] {}", err_msg);
        err_msg
    })?;

    let templates = state.templates.lock().map_err(|e| {
        let err_msg = format!("Failed to lock templates: {}", e);
        eprintln!("[save_all_data] {}", err_msg);
        err_msg
    })?;

    let history = state.history.lock().map_err(|e| {
        let err_msg = format!("Failed to lock history: {}", e);
        eprintln!("[save_all_data] {}", err_msg);
        err_msg
    })?;

    let tags = state.tags.lock().map_err(|e| {
        let err_msg = format!("Failed to lock tags: {}", e);
        eprintln!("[save_all_data] {}", err_msg);
        err_msg
    })?;

    eprintln!("[save_all_data] All locks acquired successfully");
    eprintln!("[save_all_data]   - Scripts: {}", scripts.len());
    eprintln!("[save_all_data]   - Templates: {}", templates.len());

    storage.save_all_with_data(
        scripts.iter().map(|s| std::sync::Arc::new(s.clone())).collect(),
        templates.iter().map(|t| std::sync::Arc::new(t.clone())).collect(),
        history.clone(),
        tags.clone(),
    ).map_err(|e| {
        let err_msg = format!("Failed to save data to disk: {}", e);
        eprintln!("[save_all_data] {}", err_msg);
        err_msg
    })?;

    eprintln!("[save_all_data] Data successfully saved to disk");
    Ok(())
}

fn main() {
    // Initialize storage
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("scripted-prompt");

    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let storage = Storage::new(data_dir);

    // Load data
    let (scripts, templates, history, tags) = storage.load_all()
        .unwrap_or_else(|_| (vec![], vec![], HistoryManager::default(), TagManager::new()));

    let app_state = AppState {
        storage: Mutex::new(storage),
        scripts: Mutex::new(scripts),
        templates: Mutex::new(templates),
        history: Mutex::new(history),
        tags: Mutex::new(tags),
        clipboard: ClipboardManager::new(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_all_scripts,
            get_all_templates,
            search_scripts,
            search_templates,
            create_script,
            update_script,
            delete_script,
            create_template,
            update_template,
            delete_template,
            get_all_tags,
            replace_script_variables,
            parse_script_variables,
            parse_variables_with_defaults_command,
            get_variable_usage_locations,
            copy_script_to_clipboard,
            copy_template_preview_to_clipboard,
            copy_to_clipboard,
            toggle_favorite_script,
            toggle_favorite_template,
            get_tag_rename_preview,
            rename_tag_segment_command,
            get_tag_delete_preview,
            cascade_delete_tag_command,
            export_data,
            import_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
