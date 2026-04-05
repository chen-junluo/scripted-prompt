import { AppState } from './state.js';
import { Utils } from './utils.js';
import { loadData } from './api.js';
import { renderScriptTree, renderTemplateTree } from './tree.js';
import { renderRecentScripts, renderRecentTemplates } from './recent.js';
import { selectScript, toggleScriptEditMode } from './script-editor.js';
import { switchToBrowseMode } from './composition.js';
import { showModal, showConfirm } from './modals.js';

export async function createNewScript() {
    showModal('New Script', `
        <div class="form-group">
            <label class="form-label">Name</label>
            <input type="text" class="form-input" id="newScriptName" placeholder="Enter script name">
        </div>
        <div class="form-group">
            <label class="form-label">Tags</label>
            <input type="text" class="form-input" id="newScriptTags" placeholder="e.g.: coding/python/debug">
        </div>
    `, async () => {
        const name = document.getElementById('newScriptName').value.trim();
        const tags = document.getElementById('newScriptTags').value.trim();
        if (!name) {
            alert('Please enter script name');
            return false;
        }
        try {
            const newScript = await AppState.invoke('create_script', { name, tags: tags || '', content: '' });
            await loadData();
            renderScriptTree();
            renderRecentScripts();
            selectScript(newScript);
            toggleScriptEditMode();
            Utils.showStatus('Created successfully', 'success');
            return true;
        } catch (error) {
            Utils.showStatus('Create failed: ' + error, 'error');
            return false;
        }
    });
}

export async function exportData() {
    try {
        Utils.showStatus('Selecting export location...', 'normal');
        const { save } = window.__TAURI__.dialog;
        const filePath = await save({
            title: 'Export Data',
            defaultPath: 'scripted-prompt-export.json',
            filters: [{ name: 'JSON', extensions: ['json'] }]
        });
        if (!filePath) {
            Utils.showStatus('Export cancelled', 'normal');
            return;
        }
        await AppState.invoke('export_data', { exportPath: filePath });
        Utils.showStatus('Data exported successfully', 'success');
    } catch (error) {
        Utils.showStatus('Export failed: ' + error, 'error');
    }
}

export async function importData() {
    const confirmed = await showConfirm(
        'Import Data',
        'Importing will merge scripts and templates into existing local data. Conflicting IDs will be remapped, and local history will be kept. Continue?'
    );
    if (!confirmed) return;

    try {
        Utils.showStatus('Selecting import file...', 'normal');
        const { open } = window.__TAURI__.dialog;
        const filePath = await open({
            title: 'Import Data',
            multiple: false,
            filters: [{ name: 'JSON', extensions: ['json'] }]
        });
        if (!filePath) {
            Utils.showStatus('Import cancelled', 'normal');
            return;
        }
        await AppState.invoke('import_data', { importPath: filePath });
        await loadData();
        renderScriptTree();
        renderTemplateTree();
        renderRecentScripts();
        renderRecentTemplates();
        switchToBrowseMode();
        Utils.showStatus('Data imported successfully', 'success');
    } catch (error) {
        Utils.showStatus('Import failed: ' + error, 'error');
    }
}
