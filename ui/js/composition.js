import { AppState } from './state.js';
import { Utils } from './utils.js';
import { refreshLibraryViews } from './api.js';
import { selectScript } from './script-editor.js';
import { updateCompositionPreview, confirmDiscardTemplateEditorChanges } from './preview.js';
import { showConfirm } from './modals.js';

export function clearCompositionDraftTemplate() {
    AppState.compositionDraftTemplate = null;
}

export function setCompositionDraftTemplate(patch = {}) {
    AppState.compositionDraftTemplate = {
        name: '',
        tags: '',
        variableValues: {},
        ...(AppState.compositionDraftTemplate || {}),
        ...patch,
    };
}

export async function selectTemplate(template) {
    if (!(await confirmDiscardTemplateEditorChanges())) return;
    clearCompositionDraftTemplate();
    AppState.selectedTemplate = template;
    AppState.selectedScript = null;
    AppState.currentTemplateId = template.id;
    AppState.compositionTemplateId = template.id;
    AppState.compositionScripts = template.script_ids
        .map(id => AppState.scripts.find(s => s.id === id))
        .filter(Boolean);
    AppState.variableValues = template.variable_values || {};
    switchToCompositionMode();
    renderCompositionCards();
    updateCompositionPreview();
}

export function switchToCompositionMode() {
    AppState.workspaceMode = 'composition';
    document.getElementById('templateTree').classList.add('hidden');
    document.getElementById('compositionWorkspace').classList.remove('hidden');
    if (AppState.compositionScripts.length > 0) {
        updateCompositionPreview();
    } else {
        Utils.showView('empty');
    }
}

export function switchToBrowseMode() {
    AppState.workspaceMode = 'browse';
    AppState.compositionScripts = [];
    AppState.currentTemplateId = null;
    AppState.compositionTemplateId = null;
    AppState.variableValues = {};
    clearCompositionDraftTemplate();
    document.getElementById('templateTree').classList.remove('hidden');
    document.getElementById('compositionWorkspace').classList.add('hidden');
    Utils.showView('empty');
}

export function addScriptToComposition(script) {
    if (AppState.compositionScripts.find(s => s.id === script.id)) {
        Utils.showStatus('Script already in composition', 'normal');
        return;
    }

    AppState.compositionScripts.push(script);
    if (AppState.workspaceMode === 'browse') {
        switchToCompositionMode();
    }

    renderCompositionCards();
    updateCompositionPreview();
    Utils.showStatus(`Added: ${script.name}`, 'success');
}

export function moveCompositionCard(fromIndex, toIndex) {
    if (fromIndex === toIndex || fromIndex < 0 || toIndex < 0) return;
    const [movedScript] = AppState.compositionScripts.splice(fromIndex, 1);
    AppState.compositionScripts.splice(toIndex, 0, movedScript);
    renderCompositionCards();
    updateCompositionPreview();
}

export function renderCompositionCards() {
    const container = document.getElementById('cardsContainer');

    if (AppState.compositionScripts.length === 0) {
        container.innerHTML = '';
        return;
    }

    container.innerHTML = AppState.compositionScripts.map((script, index) => {
        const preview = script.content.substring(0, 50) + (script.content.length > 50 ? '...' : '');
        const isFirst = index === 0;
        const isLast = index === AppState.compositionScripts.length - 1;
        return `
        <div class="script-card" data-index="${index}" data-script-id="${script.id}">
            <div class="card-header">
                <div class="card-controls">
                    <button class="card-move-btn" data-direction="up" data-index="${index}" title="Move up" ${isFirst ? 'disabled' : ''}>↑</button>
                    <button class="card-move-btn" data-direction="down" data-index="${index}" title="Move down" ${isLast ? 'disabled' : ''}>↓</button>
                </div>
                <span class="card-name">${Utils.escapeHtml(script.name)}</span>
                <span class="item-separator">|</span>
                <span class="card-tags-inline">${Utils.escapeHtml(script.tags || 'No tags')}</span>
                <button class="card-remove" data-index="${index}">×</button>
            </div>
            <div class="card-preview">${Utils.escapeHtml(preview)}</div>
        </div>`;
    }).join('');

    container.querySelectorAll('.card-move-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const index = parseInt(btn.dataset.index, 10);
            const direction = btn.dataset.direction;
            const targetIndex = direction === 'up' ? index - 1 : index + 1;
            moveCompositionCard(index, targetIndex);
        });
    });

    container.querySelectorAll('.card-remove').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            removeCard(parseInt(btn.dataset.index, 10));
        });
    });

    container.querySelectorAll('.script-card').forEach(card => {
        card.addEventListener('click', async (e) => {
            if (e.target.closest('.card-remove') || e.target.closest('.card-move-btn')) return;
            if (!(await confirmDiscardTemplateEditorChanges())) return;
            const script = AppState.scripts.find(s => s.id === card.dataset.scriptId);
            if (script) {
                selectScript(script);
            }
        });
    });
}

export function removeCard(index) {
    AppState.compositionScripts.splice(index, 1);
    if (AppState.compositionScripts.length === 0) {
        switchToBrowseMode();
    } else {
        renderCompositionCards();
        updateCompositionPreview();
    }
}

export function collectCurrentCompositionVariableValues() {
    const combinedContent = AppState.compositionScripts.map(script => script.content).join('\n\n');
    const variablePattern = /\{\{\s*([^}:]+)(?::([^}]+))?\s*\}\}/g;
    const variableNames = new Set();
    let match;

    while ((match = variablePattern.exec(combinedContent)) !== null) {
        variableNames.add(match[1].trim());
    }

    return Object.fromEntries(
        Object.entries(AppState.variableValues).filter(([name]) => variableNames.has(name))
    );
}

export async function createNewTemplate() {
    if (!(await confirmDiscardTemplateEditorChanges())) return;
    if (AppState.compositionScripts.length === 0) {
        Utils.showStatus('Add at least one script before creating a template', 'error');
        return;
    }

    AppState.selectedTemplate = null;
    AppState.selectedScript = null;
    AppState.currentTemplateId = null;
    AppState.compositionTemplateId = null;
    setCompositionDraftTemplate({
        name: '',
        tags: '',
        variableValues: collectCurrentCompositionVariableValues(),
    });
    AppState.variableValues = { ...AppState.compositionDraftTemplate.variableValues };
    switchToCompositionMode();
    renderCompositionCards();
    updateCompositionPreview();
}

export async function clearWorkspace() {
    if (!(await confirmDiscardTemplateEditorChanges())) return;
    AppState.compositionScripts = [];
    AppState.currentTemplateId = null;
    AppState.selectedTemplate = null;
    AppState.compositionTemplateId = null;
    AppState.variableValues = {};
    AppState.selectedScript = null;
    clearCompositionDraftTemplate();
    switchToBrowseMode();
    document.getElementById('previewTitle').textContent = 'Preview/Edit';
    document.getElementById('actionButtons').innerHTML = '';
    Utils.showStatus('Composition cleared', 'success');
}

export async function duplicateTemplate(template) {
    try {
        const newTemplate = await AppState.invoke('create_template', {
            name: template.name + ' - Copy',
            tags: template.tags || '',
            scriptIds: template.script_ids || [],
            variableValues: template.variable_values || {},
        });
        await refreshLibraryViews();
        const createdTemplate = AppState.templates.find(t => t.id === newTemplate.id);
        if (createdTemplate) await selectTemplate(createdTemplate);
        Utils.showStatus('Duplicated successfully', 'success');
    } catch (error) {
        Utils.showStatus('Duplicate failed: ' + error, 'error');
    }
}

export async function deleteTemplate(template) {
    const confirmed = await showConfirm('Confirm Deletion', `Are you sure you want to delete template "${template.name}"?`);
    if (!confirmed) return;
    try {
        await AppState.invoke('delete_template', { id: template.id });
        await refreshLibraryViews();
        if (AppState.currentTemplateId === template.id) switchToBrowseMode();
        Utils.showStatus('Deleted successfully', 'success');
    } catch (error) {
        Utils.showStatus('Delete failed: ' + error, 'error');
    }
}
