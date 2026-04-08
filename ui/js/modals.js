import { AppState } from './state.js';
import { Utils } from './utils.js';
import { loadData } from './api.js';
import { renderScriptTree, renderTemplateTree } from './tree.js';
import { renderRecentScripts, renderRecentTemplates } from './recent.js';
import { duplicateScript, showScriptEditor } from './script-editor.js';
import { duplicateTemplate, deleteTemplate, switchToBrowseMode } from './composition.js';
import { updateCompositionPreview } from './preview.js';

export async function toggleFavorite(type, id) {
    const command = type === 'script' ? 'toggle_favorite_script' : 'toggle_favorite_template';
    try {
        await AppState.invoke(command, { id });
        await loadData();
        renderScriptTree();
        renderTemplateTree();
        renderRecentScripts();
        renderRecentTemplates();

        if (type === 'script' && AppState.selectedScript?.id === id) {
            const updated = AppState.scripts.find(script => script.id === id);
            if (updated) {
                AppState.selectedScript = updated;
                showScriptEditor(updated);
            }
        } else if (type === 'script' && AppState.selectedScript) {
            const refreshedSelection = AppState.scripts.find(script => script.id === AppState.selectedScript.id);
            if (refreshedSelection) {
                AppState.selectedScript = refreshedSelection;
                showScriptEditor(refreshedSelection);
            }
        }

        if (type === 'template') {
            const updated = AppState.templates.find(template => template.id === id);
            if (updated && AppState.compositionTemplateId === id) {
                AppState.selectedTemplate = updated;
                AppState.compositionTemplateId = updated.id;
                updateCompositionPreview();
            } else if (updated && AppState.selectedTemplate?.id === id) {
                AppState.selectedTemplate = updated;
                updateCompositionPreview();
            } else if (AppState.selectedTemplate) {
                const refreshedSelection = AppState.templates.find(template => template.id === AppState.selectedTemplate.id);
                if (refreshedSelection) {
                    AppState.selectedTemplate = refreshedSelection;
                    updateCompositionPreview();
                }
            }
        }

        Utils.showStatus(type === 'script' ? 'Script favorite updated' : 'Template favorite updated', 'success');
    } catch (error) {
        Utils.showStatus('Favorite update failed: ' + error, 'error');
    }
}

export function buildImpactSection(title, tags, itemType) {
    if (!tags || tags.length === 0) return '';
    return `
        <div class="impact-summary"><strong>${title}</strong></div>
        <div class="impact-list">
            ${tags.map(tag => `<div class="impact-item ${itemType === 'delete' ? 'impact-delete' : ''}">${Utils.escapeHtml(tag)}</div>`).join('')}
        </div>
    `;
}

export async function renameTag(tagPath) {
    const tagSegment = tagPath.split('/').filter(Boolean).pop() || tagPath;
    try {
        const preview = await AppState.invoke('get_tag_rename_preview', { oldSegment: tagSegment });
        if (preview.script_count === 0 && preview.template_count === 0) {
            Utils.showStatus('No matching tags found to rename', 'error');
            return;
        }

        showModal('Rename Tag', `
            <div class="form-group">
                <label class="form-label">Current tag segment</label>
                <input type="text" class="form-input" value="${Utils.escapeHtml(tagSegment)}" disabled>
            </div>
            <div class="form-group">
                <label class="form-label">New tag segment</label>
                <input type="text" class="form-input" id="renameTagInput" value="${Utils.escapeHtml(tagSegment)}" placeholder="Enter new tag segment">
            </div>
            <div class="impact-summary"><strong>This will update ${preview.script_count} script(s) and ${preview.template_count} template(s).</strong></div>
            ${buildImpactSection('Affected script tags', preview.script_tags, 'rename')}
            ${buildImpactSection('Affected template tags', preview.template_tags, 'rename')}
        `, async () => {
            const newSegment = document.getElementById('renameTagInput').value.trim();
            if (!newSegment) {
                Utils.showStatus('Tag segment cannot be empty', 'error');
                return false;
            }
            await AppState.invoke('rename_tag_segment_command', { oldSegment: tagSegment, newSegment });
            await loadData();
            renderScriptTree();
            renderTemplateTree();
            renderRecentScripts();
            renderRecentTemplates();
            Utils.showView('empty');
            Utils.showStatus('Tag renamed successfully', 'success');
        });
    } catch (error) {
        Utils.showStatus('Rename tag failed: ' + error, 'error');
    }
}

export async function deleteTagCascade(tagPath) {
    try {
        const preview = await AppState.invoke('get_tag_delete_preview', { tagPath });
        if (preview.script_count === 0 && preview.template_count === 0) {
            Utils.showStatus('No matching tag branch found to delete', 'error');
            return;
        }

        const confirmed = await showConfirm('Delete Tag Branch', `Delete tag branch "${tagPath}" and its descendants? This will remove ${preview.script_count} script(s) and ${preview.template_count} template(s).`);
        if (!confirmed) return;

        showModal('Delete Tag Impact', `
            <div class="impact-warning"><div><strong>This action will permanently delete ${preview.script_count} script(s) and ${preview.template_count} template(s).</strong></div></div>
            ${buildImpactSection('Script tags to delete', preview.script_tags, 'delete')}
            ${buildImpactSection('Template tags to delete', preview.template_tags, 'delete')}
        `, async () => {
            await AppState.invoke('cascade_delete_tag_command', { tagPath });
            await loadData();
            renderScriptTree();
            renderTemplateTree();
            renderRecentScripts();
            renderRecentTemplates();
            switchToBrowseMode();
            Utils.showStatus('Tag branch deleted successfully', 'success');
        });
    } catch (error) {
        Utils.showStatus('Delete tag failed: ' + error, 'error');
    }
}

export function showModal(title, bodyHtml, onConfirm) {
    const overlay = document.getElementById('modalOverlay');
    document.getElementById('modalTitle').textContent = title;
    document.getElementById('modalBody').innerHTML = bodyHtml;
    document.getElementById('modalFooter').innerHTML = `
        <button class="btn btn-secondary" id="modalCancelBtn">Cancel</button>
        <button class="btn btn-primary" id="modalConfirmBtn">Confirm</button>
    `;
    overlay.classList.remove('hidden');

    const close = () => overlay.classList.add('hidden');
    document.getElementById('modalClose').onclick = close;
    document.getElementById('modalCancelBtn').onclick = close;
    document.getElementById('modalConfirmBtn').onclick = async () => {
        const confirmBtn = document.getElementById('modalConfirmBtn');
        const cancelBtn = document.getElementById('modalCancelBtn');
        const closeBtn = document.getElementById('modalClose');
        if (onConfirm) {
            try {
                confirmBtn.disabled = true;
                cancelBtn.disabled = true;
                closeBtn.style.pointerEvents = 'none';
                closeBtn.style.opacity = '0.5';
                const originalText = confirmBtn.textContent;
                confirmBtn.textContent = 'Working...';
                const result = await onConfirm();
                if (result === false) {
                    confirmBtn.disabled = false;
                    cancelBtn.disabled = false;
                    closeBtn.style.pointerEvents = '';
                    closeBtn.style.opacity = '';
                    confirmBtn.textContent = originalText;
                    return;
                }
                close();
                if (result?.nextModal) {
                    result.nextModal();
                }
            } catch (error) {
                confirmBtn.disabled = false;
                cancelBtn.disabled = false;
                closeBtn.style.pointerEvents = '';
                closeBtn.style.opacity = '';
                confirmBtn.textContent = 'Confirm';
                throw error;
            }
        } else {
            close();
        }
    };
    overlay.onclick = (e) => {
        if (e.target === overlay) close();
    };
}

export async function showAiSettingsModal() {
    const current = AppState.settings || await AppState.invoke('get_settings');
    AppState.settings = current;
    showModal('AI Settings', `
        <div class="form-group">
            <label class="form-label">Provider</label>
            <input type="text" class="form-input" id="aiProvider" value="${Utils.escapeHtml(current.provider || 'openai-compatible')}" placeholder="openai-compatible">
        </div>
        <div class="form-group">
            <label class="form-label">Base URL</label>
            <input type="text" class="form-input" id="aiBaseUrl" value="${Utils.escapeHtml(current.base_url || '')}" placeholder="https://api.poe.com/v1">
        </div>
        <div class="form-group">
            <label class="form-label">API Key</label>
            <input type="password" class="form-input" id="aiApiKey" value="${Utils.escapeHtml(current.api_key || '')}" placeholder="Enter API key">
            <div class="variable-hint">Stored locally in settings.json. Export does not include this value.</div>
        </div>
        <div class="form-group">
            <label class="form-label">Model</label>
            <input type="text" class="form-input" id="aiModel" value="${Utils.escapeHtml(current.model || '')}" placeholder="Model name from your provider">
        </div>
        <div class="form-group">
            <label class="form-label">Temperature</label>
            <input type="number" step="0.1" min="0" max="2" class="form-input" id="aiTemperature" value="${current.temperature ?? 0.2}">
        </div>
        <div class="form-group">
            <label class="form-label">Max output tokens</label>
            <input type="number" min="1" class="form-input" id="aiMaxOutputTokens" value="${current.max_output_tokens ?? 1200}">
        </div>
    `, async () => {
        const payload = collectAiSettingsPayload();
        await AppState.invoke('update_ai_settings', { payload });
        AppState.settings = payload;
        Utils.showStatus('AI settings saved', 'success');
    });
}

function collectAiSettingsPayload() {
    return {
        provider: document.getElementById('aiProvider').value.trim() || 'openai-compatible',
        base_url: document.getElementById('aiBaseUrl').value.trim(),
        api_key: document.getElementById('aiApiKey').value.trim(),
        model: document.getElementById('aiModel').value.trim(),
        temperature: parseOptionalFloat(document.getElementById('aiTemperature').value),
        max_output_tokens: parseOptionalInt(document.getElementById('aiMaxOutputTokens').value),
    };
}

function parseOptionalFloat(value) {
    if (value === '') return null;
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : null;
}

function parseOptionalInt(value) {
    if (value === '') return null;
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? parsed : null;
}

export function showConfirm(title, message) {
    return new Promise((resolve) => {
        const overlay = document.getElementById('modalOverlay');
        document.getElementById('modalTitle').textContent = title;
        document.getElementById('modalBody').innerHTML = `<p style="margin: 20px 0;">${Utils.escapeHtml(message)}</p>`;
        document.getElementById('modalFooter').innerHTML = `
            <button class="btn btn-secondary" id="modalCancelBtn">Cancel</button>
            <button class="btn btn-danger" id="modalConfirmBtn">Confirm</button>
        `;
        overlay.classList.remove('hidden');

        const close = (result) => {
            overlay.classList.add('hidden');
            resolve(result);
        };
        document.getElementById('modalClose').onclick = () => close(false);
        document.getElementById('modalCancelBtn').onclick = () => close(false);
        document.getElementById('modalConfirmBtn').onclick = () => close(true);
        overlay.onclick = (e) => {
            if (e.target === overlay) close(false);
        };
    });
}

export function showTagContextMenu(e, tagPath, type) {
    e.preventDefault();
    const menu = document.getElementById('contextMenu');
    menu.innerHTML = `
        <div class="context-menu-item" data-action="rename">Rename Tag</div>
        <div class="context-menu-item" data-action="delete">Delete Tag</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" data-action="newTag">New Sub-tag</div>
    `;
    const menuWidth = 200;
    const menuHeight = 150;
    const x = e.pageX + menuWidth > window.innerWidth ? window.innerWidth - menuWidth : e.pageX;
    const y = e.pageY + menuHeight > window.innerHeight ? window.innerHeight - menuHeight : e.pageY;
    menu.style.left = x + 'px';
    menu.style.top = y + 'px';
    menu.classList.remove('hidden');

    const handleClick = function(event) {
        event.stopPropagation();
        menu.classList.add('hidden');
        menu.removeEventListener('click', handleClick, true);
        const target = event.target.closest('.context-menu-item');
        if (!target) return;
        const action = target.dataset.action;
        if (action === 'rename') renameTag(tagPath);
        else if (action === 'delete') deleteTagCascade(tagPath);
        else if (action === 'newTag') {
            const separator = tagPath ? '/' : '';
            showModal('New Sub-tag', `
                <div class="form-group">
                    <label class="form-label">Parent tag</label>
                    <input type="text" class="form-input" value="${Utils.escapeHtml(tagPath || 'root')}" disabled>
                </div>
                <div class="form-group">
                    <label class="form-label">New sub-tag path</label>
                    <input type="text" class="form-input" value="${Utils.escapeHtml(tagPath + separator)}" id="newSubTagPath">
                </div>
                <p class="variable-hint">Create a new item under this tag path by using it when saving or creating content.</p>
            `);
        }
    };
    menu.addEventListener('click', handleClick, true);
}

export function showItemContextMenu(e, item, type) {
    e.preventDefault();
    const menu = document.getElementById('contextMenu');
    menu.innerHTML = `
        <div class="context-menu-item" data-action="copy">Copy Content</div>
        <div class="context-menu-item" data-action="favorite">${item.is_favorite ? 'Remove from Favorites' : 'Add to Favorites'}</div>
        <div class="context-menu-item" data-action="duplicate">Duplicate ${type === 'script' ? 'Script' : 'Template'}</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" data-action="delete">Delete</div>
    `;
    const menuWidth = 200;
    const menuHeight = 150;
    const x = e.pageX + menuWidth > window.innerWidth ? window.innerWidth - menuWidth : e.pageX;
    const y = e.pageY + menuHeight > window.innerHeight ? window.innerHeight - menuHeight : e.pageY;
    menu.style.left = x + 'px';
    menu.style.top = y + 'px';
    menu.classList.remove('hidden');

    const handleClick = async function(event) {
        event.stopPropagation();
        menu.classList.add('hidden');
        menu.removeEventListener('click', handleClick, true);
        const target = event.target.closest('.context-menu-item');
        if (!target) return;
        const action = target.dataset.action;

        if (action === 'copy' && type === 'script') {
            await AppState.invoke('copy_script_to_clipboard', { scriptId: item.id, text: item.content });
            await loadData();
            renderRecentScripts();
            Utils.showStatus('Copied to clipboard', 'success');
        } else if (action === 'favorite') {
            await toggleFavorite(type, item.id);
        } else if (action === 'duplicate') {
            if (type === 'script') await duplicateScript(item);
            else await duplicateTemplate(item);
        } else if (action === 'delete') {
            if (type === 'script') {
                const confirmed = await showConfirm('Confirm Deletion', `Are you sure you want to delete script "${item.name}"? Templates referencing it will automatically remove this reference.`);
                if (confirmed) {
                    await AppState.invoke('delete_script', { id: item.id });
                    await loadData();
                    renderScriptTree();
                    renderRecentScripts();
                    Utils.showView('empty');
                    Utils.showStatus('Deleted successfully', 'success');
                }
            } else {
                await deleteTemplate(item);
            }
        }
    };
    menu.addEventListener('click', handleClick, true);
}
