import { AppState } from './state.js';
import { Utils } from './utils.js';
import { loadData } from './api.js';
import { renderScriptTree } from './tree.js';
import { renderRecentScripts } from './recent.js';
import { toggleFavorite, showConfirm } from './modals.js';

export function selectScript(script) {
    document.querySelectorAll('.tree-item.selected').forEach(el => el.classList.remove('selected'));
    const item = document.querySelector(`.tree-item[data-id="${script.id}"]`);
    if (item) item.classList.add('selected');

    AppState.selectedScript = script;
    AppState.editMode = 'view';
    showScriptEditor(script);
}

export function showScriptEditor(script) {
    Utils.showView('scriptEditorMode');
    document.getElementById('previewTitle').textContent = 'Script Editor';
    document.getElementById('scriptName').value = script.name || '';
    document.getElementById('scriptTags').value = script.tags || '';
    document.getElementById('scriptContent').value = script.content || '';
    setScriptEditorReadOnly(true);
    document.getElementById('scriptMetadata').innerHTML = `
        <div><strong>Use count:</strong> ${script.use_count || 0}</div>
        <div><strong>Created:</strong> ${Utils.formatTimestamp(script.created_at)}</div>
        <div><strong>Updated:</strong> ${Utils.formatTimestamp(script.updated_at)}</div>
    `;
    document.getElementById('actionButtons').innerHTML = `
        <button class="btn btn-secondary" id="favoriteScriptBtn">${script.is_favorite ? 'Unfavorite' : 'Favorite'}</button>
        <button class="btn btn-primary" id="editScriptBtn">Edit</button>
        <button class="btn btn-secondary" id="copyScriptBtn">Copy</button>
        <button class="btn btn-danger" id="deleteScriptBtn">Delete</button>
    `;

    document.getElementById('favoriteScriptBtn')?.addEventListener('click', () => toggleFavorite('script', script.id));
    document.getElementById('editScriptBtn')?.addEventListener('click', toggleScriptEditMode);
    document.getElementById('copyScriptBtn')?.addEventListener('click', copyScriptContent);
    document.getElementById('deleteScriptBtn')?.addEventListener('click', deleteScript);
}

export function setScriptEditorReadOnly(readonly) {
    document.getElementById('scriptName').readOnly = readonly;
    document.getElementById('scriptTags').readOnly = readonly;
    document.getElementById('scriptContent').readOnly = readonly;
}

export function toggleScriptEditMode() {
    if (AppState.editMode === 'view') {
        AppState.editMode = 'edit';
        AppState.originalScriptData = {
            name: AppState.selectedScript.name,
            tags: AppState.selectedScript.tags,
            content: AppState.selectedScript.content,
        };

        setScriptEditorReadOnly(false);
        document.getElementById('actionButtons').innerHTML = `
            <button class="btn btn-primary" id="saveScriptBtn">Save</button>
            <button class="btn btn-secondary" id="cancelEditBtn">Cancel</button>
        `;

        document.getElementById('saveScriptBtn').addEventListener('click', saveScript);
        document.getElementById('cancelEditBtn').addEventListener('click', cancelScriptEdit);
        document.getElementById('scriptName').focus();
    }
}

export async function saveScript() {
    const name = document.getElementById('scriptName').value.trim();
    const tags = document.getElementById('scriptTags').value.trim();
    const content = document.getElementById('scriptContent').value;

    if (!name) {
        alert('Please enter script name');
        return;
    }

    try {
        await AppState.invoke('update_script', { id: AppState.selectedScript.id, name, tags, content });
        AppState.editMode = 'view';
        await loadData();
        renderScriptTree();
        renderRecentScripts();
        const updated = AppState.scripts.find(s => s.id === AppState.selectedScript.id);
        if (updated) {
            AppState.selectedScript = updated;
            showScriptEditor(updated);
        }
        Utils.showStatus('Saved successfully', 'success');
    } catch (error) {
        Utils.showStatus('Save failed: ' + error, 'error');
    }
}

export function cancelScriptEdit() {
    if (AppState.originalScriptData) {
        document.getElementById('scriptName').value = AppState.originalScriptData.name;
        document.getElementById('scriptTags').value = AppState.originalScriptData.tags;
        document.getElementById('scriptContent').value = AppState.originalScriptData.content;
    }
    AppState.editMode = 'view';
    AppState.originalScriptData = null;
    showScriptEditor(AppState.selectedScript);
}

export async function copyScriptContent() {
    const content = document.getElementById('scriptContent').value;
    try {
        const selectedId = AppState.selectedScript.id;
        await AppState.invoke('copy_script_to_clipboard', { scriptId: selectedId, text: content });
        Utils.showStatus('Copied to clipboard', 'success');
        await loadData();
        renderRecentScripts();
        renderScriptTree();
        const updated = AppState.scripts.find(script => script.id === selectedId);
        if (updated) {
            AppState.selectedScript = updated;
            showScriptEditor(updated);
        }
    } catch (error) {
        Utils.showStatus('Copy failed: ' + error, 'error');
    }
}

export async function duplicateScript(script) {
    try {
        const newScript = await AppState.invoke('create_script', {
            name: script.name + ' - Copy',
            tags: script.tags || '',
            content: script.content || '',
        });
        await loadData();
        renderScriptTree();
        renderRecentScripts();
        selectScript(newScript);
        Utils.showStatus('Duplicated successfully', 'success');
    } catch (error) {
        Utils.showStatus('Duplicate failed: ' + error, 'error');
    }
}

export async function deleteScript() {
    const confirmed = await showConfirm(
        'Confirm Deletion',
        'Are you sure you want to delete this script? Templates referencing it will automatically remove this reference.'
    );
    if (!confirmed) return;

    try {
        await AppState.invoke('delete_script', { id: AppState.selectedScript.id });
        await loadData();
        renderScriptTree();
        renderRecentScripts();
        Utils.showView('empty');
        Utils.showStatus('Deleted successfully', 'success');
    } catch (error) {
        Utils.showStatus('Delete failed: ' + error, 'error');
    }
}
