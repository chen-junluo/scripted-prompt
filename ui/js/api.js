import { AppState } from './state.js';
import { Utils } from './utils.js';
import { setupEventListeners } from './main.js';
import { renderScriptTree, renderTemplateTree } from './tree.js';
import { renderRecentScripts, renderRecentTemplates } from './recent.js';

export async function init() {
    try {
        if (!window.__TAURI__ || !window.__TAURI__.core) {
            throw new Error('Tauri API not loaded');
        }

        AppState.invoke = window.__TAURI__.core.invoke;
        Utils.showStatus('Loading data...', 'normal');

        await loadData();
        setupEventListeners();
        renderScriptTree();
        renderTemplateTree();
        renderRecentScripts();
        renderRecentTemplates();

        if (window.lucide) {
            window.lucide.createIcons();
        }

        Utils.showStatus('Ready', 'success');
    } catch (error) {
        Utils.showStatus('Initialization failed: ' + error.message, 'error');
        console.error('Init error:', error);
    }
}

export async function loadData() {
    try {
        AppState.scripts = await AppState.invoke('get_all_scripts');
        AppState.templates = await AppState.invoke('get_all_templates');

        document.getElementById('scriptsCount').textContent = `(${AppState.scripts.length})`;
        document.getElementById('templatesCount').textContent = `(${AppState.templates.length})`;
    } catch (error) {
        console.error('Failed to load data:', error);
        throw error;
    }
}
