import { AppState } from './state.js';
import { renderScriptTree, renderTemplateTree } from './tree.js';
import { updateCompositionPreview } from './preview.js';
import { createNewScript } from './new-items.js';
import { createNewTemplate, saveCompositionAsTemplate, clearWorkspace } from './composition.js';
import { exportData, importData } from './data-transfer.js';

export function handleScriptSearch() {
    renderScriptTree();
}

export function handleTemplateSearch() {
    renderTemplateTree();
}

export function setupEventListeners() {
    document.getElementById('newScriptBtn').addEventListener('click', createNewScript);
    document.getElementById('newTemplateBtn').addEventListener('click', createNewTemplate);
    document.getElementById('scriptSearch').addEventListener('input', handleScriptSearch);
    document.getElementById('templateSearch').addEventListener('input', handleTemplateSearch);
    document.getElementById('scriptFavoritesOnly').addEventListener('change', (e) => {
        AppState.scriptFavoritesOnly = e.target.checked;
        renderScriptTree();
    });
    document.getElementById('templateFavoritesOnly').addEventListener('change', (e) => {
        AppState.templateFavoritesOnly = e.target.checked;
        renderTemplateTree();
    });
    document.getElementById('previewBtn').addEventListener('click', () => {
        if (AppState.compositionScripts.length > 0) updateCompositionPreview();
    });
    document.getElementById('clearWorkspaceBtn').addEventListener('click', clearWorkspace);
    document.getElementById('saveTemplateBtn').addEventListener('click', saveCompositionAsTemplate);
    document.getElementById('exportDataBtn').addEventListener('click', exportData);
    document.getElementById('importDataBtn').addEventListener('click', importData);
    document.addEventListener('click', (e) => {
        if (!e.target.closest('#contextMenu')) {
            document.getElementById('contextMenu').classList.add('hidden');
        }
    });
}
