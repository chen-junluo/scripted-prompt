import { AppState } from './state.js';
import { Utils } from './utils.js';
import { loadData } from './api.js';
import { renderRecentScripts, renderRecentTemplates } from './recent.js';
import { renderScriptTree, renderTemplateTree } from './tree.js';
import { toggleFavorite, showAiSettingsModal, showModal } from './modals.js';
import { selectTemplate } from './composition.js';
import { selectScript } from './script-editor.js';

function getActiveCompositionTemplate() {
    if (!AppState.compositionTemplateId) return null;
    return AppState.templates.find(template => template.id === AppState.compositionTemplateId) || null;
}

function getTemplateEditorName() {
    return document.getElementById('templateEditorName');
}

function getTemplateEditorTags() {
    return document.getElementById('templateEditorTags');
}

function getTemplateEditorSnapshot() {
    return {
        name: getTemplateEditorName()?.value.trim() || '',
        tags: getTemplateEditorTags()?.value.trim() || '',
        variableValues: { ...AppState.variableValues },
    };
}

function isTemplateEditorDirty() {
    if (!AppState.templateEditorOriginalData) return false;
    return JSON.stringify(getTemplateEditorSnapshot()) !== JSON.stringify(AppState.templateEditorOriginalData);
}

function updateTemplateEditorSaveButtonState() {
    const saveButton = document.getElementById('saveTemplateEditorBtn');
    if (!saveButton) return;
    saveButton.disabled = !isTemplateEditorDirty();
    saveButton.title = saveButton.disabled ? 'No unsaved changes' : 'Save template changes';
}

export function hasUnsavedTemplateEditorChanges() {
    const saveButton = document.getElementById('saveTemplateEditorBtn');
    if (!saveButton) return false;
    return !saveButton.disabled;
}

export async function confirmDiscardTemplateEditorChanges() {
    if (!hasUnsavedTemplateEditorChanges()) return true;
    return window.confirm('You have unsaved template changes. Discard them?');
}

function bindTemplateEditorDirtyTracking(activeTemplate) {
    if (!activeTemplate) return;
    getTemplateEditorName()?.addEventListener('input', updateTemplateEditorSaveButtonState);
    getTemplateEditorTags()?.addEventListener('input', updateTemplateEditorSaveButtonState);
    document.querySelectorAll('#variablesList .variable-input').forEach(input => {
        input.addEventListener('input', updateTemplateEditorSaveButtonState);
    });
    updateTemplateEditorSaveButtonState();
}

function buildCompressionPrompt(templateName, templateTags, orderedScripts, composedContent, options) {
    const sourceVariables = parseVariablesWithDefaults(composedContent);
    const scriptSection = orderedScripts
        .map((script, index) => `Script ${index + 1}\nName: ${script.name}\nContent:\n${script.content}`)
        .join('\n\n');

    const variableSection = sourceVariables.length === 0
        ? '[]'
        : JSON.stringify(sourceVariables, null, 2);

    const suggestedName = options.suggestedName?.trim() || '(none)';
    const suggestedTags = options.suggestedTags?.trim() || templateTags;

    return `You are converting a multi-part prompt template into one concise reusable prompt script.

Your goal:
- Extract the core task from the source template.
- Remove repetition, scaffolding, and overly specific workflow details.
- Preserve reusable instructions that affect output quality.
- Organize the final prompt with short bullet points when listing requirements, constraints, or steps.
- Keep the final prompt shorter than the source unless shortening would remove essential instructions.

Variable rules:
- Preserve existing variables that are still necessary.
- Keep variables in this exact syntax: {{name}} or {{name:default}}.
- Do not rename variables unless the original name is unclear.
- Do not invent new variables unless absolutely necessary.
- If a variable is removed from content, omit it from variable_defaults.
- If preserve_variables is true, strongly prefer keeping all source variables.

Tag rules:
- result.tags must be exactly one tag path string.
- A tag path may contain multiple hierarchy levels separated by /, for example research/literature/review.
- Do not return multiple tags.
- Do not separate tags with commas, semicolons, pipes, spaces, or arrays.
- If unsure, return the single best tag path.

Output format:
- Return exactly one JSON object.
- Do not wrap it in markdown.
- Do not include explanations before or after it.
- The first character of your response must be { and the last character must be }.

JSON schema:
{
  "version": "1",
  "result": {
    "script_name": "string",
    "tags": "string",
    "content": "string",
    "variable_defaults": {
      "variable_name": "default value"
    },
    "summary": "string"
  }
}

Example output:
{
  "version": "1",
  "result": {
    "script_name": "Literature Review Prompt",
    "tags": "research/literature",
    "content": "Write a focused literature review on {{topic}}.\n\n- Identify the main debates.\n- Compare key findings.\n- Note gaps and limitations.\n- End with 3 possible research questions.",
    "variable_defaults": {
      "topic": ""
    },
    "summary": "Compressed the source template into a reusable literature review prompt."
  }
}

Content requirements:
- result.content must be the compressed prompt script.
- Use bullet points when listing requirements, constraints, or steps.
- Keep line breaks readable.
- Keep only actionable instructions.
- Preserve placeholders exactly, for example {{topic}} or {{audience:beginner}}.
- result.tags must be a single slash-delimited tag path, not a comma-separated list.
- Do not use markdown fences.

Template metadata:
- Template name: ${templateName}
- Template tags: ${templateTags}
- Suggested script name: ${suggestedName}
- Suggested tags: ${suggestedTags}
- preserve_variables: ${options.preserveVariables ? 'true' : 'false'}

Ordered source scripts:
${scriptSection}

Composed source prompt:
${composedContent}

Parsed source variables:
${variableSection}
`;
}

function parseVariablesWithDefaults(content) {
    const regex = /\{\{\s*([^}:]+)(?::([^}]+))?\s*\}\}/g;
    const seen = new Map();
    for (const match of content.matchAll(regex)) {
        const name = match[1]?.trim();
        if (!name || seen.has(name)) continue;
        seen.set(name, {
            name,
            default_value: match[2]?.trim() || '',
        });
    }
    return Array.from(seen.values());
}

async function requestCompressionFromProvider(settings, prompt) {
    const baseUrl = settings.base_url.trim().replace(/\/$/, '');
    const url = baseUrl.endsWith('/chat/completions') ? baseUrl : `${baseUrl}/chat/completions`;
    const body = {
        model: settings.model.trim(),
        messages: [
            { role: 'system', content: 'You return strict JSON only.' },
            { role: 'user', content: prompt },
        ],
        temperature: settings.temperature,
        max_tokens: settings.max_output_tokens,
        max_completion_tokens: settings.max_output_tokens,
        stream: false,
    };

    const response = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'Authorization': `Bearer ${settings.api_key.trim()}`,
        },
        body: JSON.stringify(body),
    });

    const text = await response.text();
    if (!response.ok) {
        throw new Error(formatProviderError(response.status, url, text));
    }

    let parsed;
    try {
        parsed = JSON.parse(text);
    } catch (error) {
        throw new Error(`Failed to parse AI response: ${error}`);
    }

    const content = parsed?.choices?.[0]?.message?.content;
    if (!content) {
        throw new Error('AI response did not contain any choices');
    }
    return content;
}

function formatProviderError(status, url, body) {
    const providerHint = {
        400: 'Bad request. Check model name and request fields.',
        401: 'Authentication failed. Check the API key.',
        402: 'Insufficient Poe credits or subscription points.',
        403: 'Request was forbidden by the provider.',
        404: 'Endpoint or model was not found. For Poe use base URL https://api.poe.com/v1 and a valid model name like Claude-Sonnet-4.6.',
        408: 'The provider timed out before the model started.',
        413: 'The request was too large for the model context window.',
        429: 'The provider rate limit was hit. Try again shortly.',
        500: 'Provider returned an internal error, which can also happen for invalid upstream requests.',
        502: 'Upstream model backend was unavailable.',
        529: 'Provider is temporarily overloaded.',
    }[status] || 'Provider returned an unexpected error.';

    const compactBody = (body || '').trim();
    return compactBody
        ? `AI request failed with status ${status} at ${url}. ${providerHint} Response: ${compactBody}`
        : `AI request failed with status ${status} at ${url}. ${providerHint}`;
}

export async function updateCompositionPreview() {
    if (AppState.compositionScripts.length === 0) {
        Utils.showView('empty');
        document.getElementById('previewTitle').textContent = 'Preview/Edit';
        document.getElementById('actionButtons').innerHTML = '';
        return;
    }

    const activeTemplate = getActiveCompositionTemplate();

    Utils.showView('compositionPreviewMode');
    document.getElementById('previewTitle').textContent = activeTemplate ? 'Template Editor' : 'Composition Preview';
    document.getElementById('actionButtons').innerHTML = activeTemplate
        ? `<button class="btn btn-secondary" id="favoriteTemplateBtn">${activeTemplate.is_favorite ? 'Unfavorite' : 'Favorite'}</button><button class="btn btn-secondary" id="compressTemplateBtn">${AppState.isCompressing ? 'Compressing…' : 'Compress with AI'}</button><button class="btn btn-primary" id="copyPreviewBtn">Copy</button><button class="btn btn-primary" id="saveTemplateEditorBtn">Save</button>`
        : `<button class="btn btn-primary" id="copyPreviewBtn">Copy</button>`;

    const templateNameInput = getTemplateEditorName();
    const templateTagsInput = getTemplateEditorTags();
    if (templateNameInput) {
        templateNameInput.value = activeTemplate?.name || '';
        templateNameInput.readOnly = !activeTemplate;
    }
    if (templateTagsInput) {
        templateTagsInput.value = activeTemplate?.tags || '';
        templateTagsInput.readOnly = !activeTemplate;
    }
    AppState.templateEditorOriginalData = activeTemplate
        ? {
            name: activeTemplate.name || '',
            tags: activeTemplate.tags || '',
            variableValues: { ...AppState.variableValues },
        }
        : null;

    document.getElementById('copyPreviewBtn').addEventListener('click', copyPreviewContent);
    const favoriteTemplateBtn = document.getElementById('favoriteTemplateBtn');
    if (favoriteTemplateBtn && activeTemplate) {
        favoriteTemplateBtn.addEventListener('click', () => toggleFavorite('template', activeTemplate.id));
    }
    const compressTemplateBtn = document.getElementById('compressTemplateBtn');
    if (compressTemplateBtn && activeTemplate) {
        compressTemplateBtn.disabled = AppState.isCompressing;
        compressTemplateBtn.addEventListener('click', compressSelectedTemplate);
    }
    const saveTemplateEditorBtn = document.getElementById('saveTemplateEditorBtn');
    if (saveTemplateEditorBtn && activeTemplate) {
        saveTemplateEditorBtn.addEventListener('click', saveTemplateEditor);
    }

    const combinedContent = AppState.compositionScripts.map(s => s.content).join('\n\n');
    try {
        const variables = await AppState.invoke('parse_variables_with_defaults_command', { content: combinedContent });
        const scriptIds = AppState.compositionScripts.map(s => s.id);
        const usageInfo = await AppState.invoke('get_variable_usage_locations', { scriptIds });
        renderVariablesPanel(variables, usageInfo, combinedContent);
        bindTemplateEditorDirtyTracking(activeTemplate);
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
            updateTemplateEditorSaveButtonState();
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

async function saveTemplateEditor() {
    const activeTemplate = getActiveCompositionTemplate();
    if (!activeTemplate) return;

    const name = getTemplateEditorName()?.value.trim() || '';
    const tags = getTemplateEditorTags()?.value.trim() || '';
    if (!name) {
        Utils.showStatus('Template name cannot be empty', 'error');
        return;
    }

    try {
        await AppState.invoke('update_template', {
            id: activeTemplate.id,
            name,
            tags,
            scriptIds: AppState.compositionScripts.map(script => script.id),
            variableValues: AppState.variableValues,
        });
        await loadData();
        renderTemplateTree();
        renderRecentTemplates();
        renderScriptTree();
        const updated = AppState.templates.find(template => template.id === activeTemplate.id);
        if (updated) {
            AppState.selectedTemplate = updated;
            AppState.currentTemplateId = updated.id;
            AppState.compositionTemplateId = updated.id;
            AppState.compositionScripts = updated.script_ids
                .map(id => AppState.scripts.find(script => script.id === id))
                .filter(Boolean);
            AppState.variableValues = updated.variable_values || {};
            AppState.templateEditorOriginalData = {
                name: updated.name || '',
                tags: updated.tags || '',
                variableValues: { ...(updated.variable_values || {}) },
            };
            renderRecentTemplates();
            await updateCompositionPreview();
        }
        Utils.showStatus('Template saved', 'success');
    } catch (error) {
        Utils.showStatus('Save failed: ' + error, 'error');
    }
}

export async function copyPreviewContent() {
    const content = document.getElementById('previewTextarea').value;
    const activeTemplate = getActiveCompositionTemplate();
    try {
        if (activeTemplate?.id) {
            const selectedId = activeTemplate.id;
            await AppState.invoke('copy_template_preview_to_clipboard', { templateId: selectedId, text: content });
            await loadData();
            renderRecentTemplates();
            const updated = AppState.templates.find(template => template.id === selectedId);
            if (updated) {
                AppState.selectedTemplate = updated;
                AppState.compositionTemplateId = updated.id;
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

async function compressSelectedTemplate() {
    const activeTemplate = getActiveCompositionTemplate();
    if (!activeTemplate) return;

    const currentSettings = AppState.settings || await AppState.invoke('get_settings');
    AppState.settings = currentSettings;
    if (!currentSettings.base_url || !currentSettings.api_key || !currentSettings.model) {
        await showAiSettingsModal();
        return;
    }

    const orderedScripts = AppState.compositionScripts.map(script => ({
        name: script.name,
        content: script.content,
    }));
    const composedContent = orderedScripts.map(script => script.content).join('\n\n');

    const suggestedName = `${activeTemplate.name} - Compressed`;
    showModal('Compress Template with AI', `
        <div class="form-group">
            <label class="form-label">Output script name</label>
            <input type="text" class="form-input" id="compressScriptName" value="${Utils.escapeHtml(suggestedName)}">
        </div>
        <div class="form-group">
            <label class="form-label">Tags</label>
            <input type="text" class="form-input" id="compressScriptTags" value="${Utils.escapeHtml(activeTemplate.tags || '')}">
        </div>
        <div class="form-group">
            <label class="favorite-filter">
                <input type="checkbox" id="compressPreserveVariables" checked>
                <span>Prefer preserving all variables</span>
            </label>
        </div>
        <div class="variable-hint" id="compressRequestHint">The app will request a compressed prompt from your configured model.</div>
    `, async () => {
        AppState.isCompressing = true;
        updateCompositionPreview();
        const hint = document.getElementById('compressRequestHint');
        if (hint) {
            hint.textContent = 'Contacting AI provider...';
        }
        try {
            const suggestedNameValue = document.getElementById('compressScriptName').value.trim();
            const suggestedTagsValue = document.getElementById('compressScriptTags').value.trim();
            const preserveVariables = document.getElementById('compressPreserveVariables').checked;
            const prompt = buildCompressionPrompt(
                activeTemplate.name,
                activeTemplate.tags || '',
                orderedScripts,
                composedContent,
                {
                    suggestedName: suggestedNameValue,
                    suggestedTags: suggestedTagsValue,
                    preserveVariables,
                },
            );
            const rawResponse = await requestCompressionFromProvider(currentSettings, prompt);
            const preview = await AppState.invoke('preview_template_compression', {
                templateId: activeTemplate.id,
                suggestedName: suggestedNameValue,
                suggestedTags: suggestedTagsValue,
                preserveVariables,
                rawResponse,
            });
            AppState.compressionPreview = preview;
            return {
                nextModal: () => showCompressionPreview(preview),
            };
        } catch (error) {
            if (hint) {
                hint.textContent = 'Compression request failed. Check your settings and try again.';
            }
            Utils.showStatus('Compression failed: ' + error, 'error');
            return false;
        } finally {
            AppState.isCompressing = false;
            updateCompositionPreview();
        }
    });
}

function showCompressionPreview(preview) {
    const warnings = preview.warnings?.length
        ? `<div class="impact-warning"><strong>Warnings</strong><div>${preview.warnings.map(item => `<div>${Utils.escapeHtml(item)}</div>`).join('')}</div></div>`
        : '<div class="empty-hint">No warnings</div>';

    showModal('Compression Preview', `
        <div class="form-group">
            <label class="form-label">Script name</label>
            <input type="text" class="form-input" id="compressionPreviewName" value="${Utils.escapeHtml(preview.script_name)}">
        </div>
        <div class="form-group">
            <label class="form-label">Tags</label>
            <input type="text" class="form-input" id="compressionPreviewTags" value="${Utils.escapeHtml(preview.tags || '')}">
        </div>
        <div class="form-group">
            <label class="form-label">Summary</label>
            <div class="variable-hint">${Utils.escapeHtml(preview.summary || '')}</div>
        </div>
        <div class="form-group">
            <label class="form-label">Source variables</label>
            <div class="variable-hint">${Utils.escapeHtml((preview.source_variable_names || []).join(', ') || 'None')}</div>
        </div>
        <div class="form-group">
            <label class="form-label">Output variables</label>
            <div class="variable-hint">${Utils.escapeHtml((preview.output_variable_names || []).join(', ') || 'None')}</div>
        </div>
        ${warnings}
        <div class="form-group">
            <label class="form-label">Content</label>
            <textarea class="form-textarea" id="compressionPreviewContent" rows="14">${Utils.escapeHtml(preview.content)}</textarea>
        </div>
        <div class="variable-hint">Length: ${preview.source_length} → ${preview.output_length}</div>
    `, async () => {
        try {
            const payload = {
                ...preview,
                script_name: document.getElementById('compressionPreviewName').value.trim(),
                tags: document.getElementById('compressionPreviewTags').value.trim(),
                content: document.getElementById('compressionPreviewContent').value,
            };
            const created = await AppState.invoke('apply_template_compression', { preview: payload });
            await loadData();
            renderScriptTree();
            renderRecentScripts();
            const refreshed = AppState.scripts.find(script => script.id === created.id) || created;
            selectScript(refreshed);
            Utils.showStatus('Compressed script created', 'success');
        } catch (error) {
            Utils.showStatus('Failed to create compressed script: ' + error, 'error');
            return false;
        }
    });
}
