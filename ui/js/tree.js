import { AppState } from './state.js';
import { Utils } from './utils.js';
import { selectScript } from './script-editor.js';
import { selectTemplate, addScriptToComposition } from './composition.js';
import { confirmDiscardTemplateEditorChanges } from './preview.js';
import { showTagContextMenu, showItemContextMenu } from './modals.js';

export function buildTagTree(items) {
    const root = { children: new Map(), items: [] };

    items.forEach(item => {
        const tags = item.tags ? item.tags.split('/').filter(t => t.trim()) : ['Uncategorized'];
        let node = root;

        tags.forEach((tag, index) => {
            if (!node.children.has(tag)) {
                node.children.set(tag, {
                    name: tag,
                    path: tags.slice(0, index + 1).join('/'),
                    children: new Map(),
                    items: []
                });
            }
            node = node.children.get(tag);
        });

        node.items.push(item);
    });

    return root;
}

export function countItems(node) {
    let count = node.items.length;
    node.children.forEach(child => {
        count += countItems(child);
    });
    return count;
}

export function getVisibleScripts() {
    const base = AppState.scriptFavoritesOnly
        ? AppState.scripts.filter(script => script.is_favorite)
        : AppState.scripts;

    const query = document.getElementById('scriptSearch')?.value.trim().toLowerCase() || '';
    if (!query) return base;

    return base.filter(script =>
        script.name.toLowerCase().includes(query) ||
        script.tags.toLowerCase().includes(query) ||
        script.content.toLowerCase().includes(query)
    );
}

export function getVisibleTemplates() {
    const base = AppState.templateFavoritesOnly
        ? AppState.templates.filter(template => template.is_favorite)
        : AppState.templates;

    const query = document.getElementById('templateSearch')?.value.trim().toLowerCase() || '';
    if (!query) return base;

    return base.filter(template =>
        template.name.toLowerCase().includes(query) ||
        template.tags.toLowerCase().includes(query)
    );
}

export function renderScriptTree(filteredScripts = null) {
    const container = document.getElementById('scriptTree');
    const scripts = filteredScripts || getVisibleScripts();

    if (scripts.length === 0) {
        container.innerHTML = '<div class="empty-hint" style="padding: 20px;">No scripts</div>';
        return;
    }

    const tree = buildTagTree(scripts);
    container.innerHTML = '';
    renderTreeNode(container, tree, 'script');
}

export function renderTemplateTree(filteredTemplates = null) {
    const container = document.getElementById('templateTree');
    const templates = filteredTemplates || getVisibleTemplates();

    if (templates.length === 0) {
        container.innerHTML = '<div class="empty-hint" style="padding: 20px;">No templates</div>';
        return;
    }

    const tree = buildTagTree(templates);
    container.innerHTML = '';
    renderTreeNode(container, tree, 'template');
}

export function renderTreeNode(container, node, type) {
    node.children.forEach((child, tagName) => {
        const folderDiv = document.createElement('div');
        folderDiv.className = 'tree-folder';

        const headerDiv = document.createElement('div');
        headerDiv.className = 'folder-header';
        headerDiv.innerHTML = `
            <span class="folder-toggle"></span>
            <span class="folder-icon">📁</span>
            <span class="folder-label">${Utils.escapeHtml(tagName)}</span>
            <span class="folder-count">${countItems(child)}</span>
        `;

        headerDiv.addEventListener('click', (e) => {
            if (!e.target.classList.contains('folder-header') &&
                !e.target.classList.contains('folder-toggle')) return;
            folderDiv.classList.toggle('collapsed');
        });

        headerDiv.addEventListener('contextmenu', (e) => {
            e.preventDefault();
            showTagContextMenu(e, child.path, type);
        });

        folderDiv.appendChild(headerDiv);

        const childrenDiv = document.createElement('div');
        childrenDiv.className = 'folder-children';
        renderTreeNode(childrenDiv, child, type);
        folderDiv.appendChild(childrenDiv);
        container.appendChild(folderDiv);
    });

    node.items.forEach(item => {
        const itemDiv = document.createElement('div');
        itemDiv.className = 'tree-item';
        itemDiv.dataset.id = item.id;

        if (type === 'script') {
            const preview = item.content.substring(0, 40) + (item.content.length > 40 ? '...' : '');
            itemDiv.innerHTML = `
                <span class="item-name">${item.is_favorite ? '<span class="favorite-badge">★</span>' : ''}${Utils.escapeHtml(item.name)}</span>
                <span class="item-separator">|</span>
                <span class="item-preview">${Utils.escapeHtml(preview)}</span>
                <button class="item-add-btn" data-id="${item.id}" title="Add to composition">→</button>
            `;

            const addBtn = itemDiv.querySelector('.item-add-btn');
            addBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                addScriptToComposition(item);
            });
        } else {
            itemDiv.innerHTML = `
                <span class="item-name">${item.is_favorite ? '<span class="favorite-badge">★</span>' : ''}${Utils.escapeHtml(item.name)}</span>
            `;
        }

        itemDiv.addEventListener('click', () => {
            if (type === 'script') {
                selectScript(item);
            } else {
                selectTemplate(item);
            }
        });

        itemDiv.addEventListener('contextmenu', (e) => showItemContextMenu(e, item, type));
        container.appendChild(itemDiv);
    });
}
