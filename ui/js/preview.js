import { AppState } from './state.js';
import { Utils } from './utils.js';
import { loadData } from './api.js';
import { renderRecentTemplates } from './recent.js';
import { toggleFavorite } from './modals.js';
import { selectTemplate } from './composition.js';

export async function updateCompositionPreview() {
    if (AppState.compositionScripts.length === 0) {
        Utils.showView('empty');
        return;
    }

    Utils.showView('compositionPreviewMode');
    document.getElementById('previewTitle').textContent = AppState.selectedTemplate ? AppState.selectedTemplate.name : 'Composition Preview';
    document.getElementById('actionButtons').innerHTML = AppState.selectedTemplate
        ? `<button class="btn btn-secondary" id="favoriteTemplateBtn">${AppState.selectedTemplate.is_favorite ? 'Unfavorite' : 'Favorite'}</button><button class="btn btn-primary" id="copyPreviewBtn">Copy</button>`
        : `<button class="btn btn-primary" id="copyPreviewBtn">Copy</button>`;

    document.getElementById('copyPreviewBtn').addEventListener('click', copyPreviewContent);
    const favoriteTemplateBtn = document.getElementById('favoriteTemplateBtn');
    if (favoriteTemplateBtn && AppState.selectedTemplate) {
        favoriteTemplateBtn.addEventListener('click', () => toggleFavorite('template', AppState.selectedTemplate.id));
    }

    const combinedContent = AppState.compositionScripts.map(s => s.content).join('\n\n');
    try {
        const variables = await AppState.invoke('parse_variables_with_defaults_command', { content: combinedContent });
        const scriptIds = AppState.compositionScripts.map(s => s.id);
        const usageInfo = await AppState.invoke('get_variable_usage_locations', { scriptIds });
        renderVariablesPanel(variables, usageInfo, combinedContent);
    } catch (error) {
        console.error('Failed to parse variables:', error);
        document.getElementById('previewTextarea').value = combinedContent;
    }
}

export function renderVariablesPanel(variables, usageInfo, combinedContent) {
    const container = document.getElementById('variablesList');
    if (variables.length === 0) {
        container.innerHTML = '<div class="empty-hint">No variables</div>';
        document.getElementById('previewTextarea').value = combinedContent;
        return;
    }

    const allVariableNames = variables.map(v => v.name);
    const datalistHtml = `
        <datalist id="variable-suggestions">
            ${allVariableNames.map(name => `<option value="${Utils.escapeHtml(name)}">`).join('')}
        </datalist>
    `;

    container.innerHTML = datalistHtml + variables.map(variable => {
        const value = AppState.variableValues[variable.name] || '';
        const usage = usageInfo.find(u => u.variable_name === variable.name);
        const usageText = usage ? usage.script_names.join(', ') : '';
        const defaultValueText = variable.default_value ? `Default: ${variable.default_value}` : '';
        return `
            <div class="variable-item">
                <label class="variable-label">${Utils.escapeHtml(variable.name)}</label>
                <input type="text" class="variable-input" data-var="${Utils.escapeHtml(variable.name)}"
                       value="${Utils.escapeHtml(value)}"
                       placeholder="${variable.default_value ? Utils.escapeHtml(variable.default_value) : '输入值...'}"
                       list="variable-suggestions">
                ${defaultValueText ? `<div class="variable-hint">${Utils.escapeHtml(defaultValueText)}</div>` : ''}
                ${usageText ? `<div class="variable-usage">Used in: ${Utils.escapeHtml(usageText)}</div>` : ''}
            </div>`;
    }).join('');

    container.querySelectorAll('.variable-input').forEach(input => {
        input.addEventListener('input', async () => {
            AppState.variableValues[input.dataset.var] = input.value;
            await updatePreviewText(combinedContent);
        });
    });

    updatePreviewText(combinedContent);
}

export async function updatePreviewText(originalContent) {
    try {
        const replaced = await AppState.invoke('replace_script_variables', {
            content: originalContent,
            variables: AppState.variableValues,
        });
        document.getElementById('previewTextarea').value = replaced;
    } catch (error) {
        console.error('Failed to replace variables:', error);
        document.getElementById('previewTextarea').value = originalContent;
    }
}

export async function copyPreviewContent() {
    const content = document.getElementById('previewTextarea').value;
    try {
        if (AppState.selectedTemplate?.id) {
            const selectedId = AppState.selectedTemplate.id;
            await AppState.invoke('copy_template_preview_to_clipboard', { templateId: selectedId, text: content });
            await loadData();
            renderRecentTemplates();
            const updated = AppState.templates.find(template => template.id === selectedId);
            if (updated) {
                AppState.selectedTemplate = updated;
                selectTemplate(updated);
            }
        } else {
            await AppState.invoke('copy_to_clipboard', { text: content });
        }
        Utils.showStatus('Copied to clipboard', 'success');
    } catch (error) {
        Utils.showStatus('Copy failed: ' + error, 'error');
    }
}
