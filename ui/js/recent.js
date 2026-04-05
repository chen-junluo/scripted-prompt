import { AppState } from './state.js';
import { Utils } from './utils.js';
import { selectScript } from './script-editor.js';
import { selectTemplate } from './composition.js';

export function renderRecentScripts() {
    const container = document.getElementById('recentScripts').querySelector('.recent-scroll') ||
                     document.getElementById('recentScripts');

    const recent = [...AppState.scripts]
        .sort((a, b) => {
            if ((b.is_favorite ? 1 : 0) !== (a.is_favorite ? 1 : 0)) {
                return (b.is_favorite ? 1 : 0) - (a.is_favorite ? 1 : 0);
            }
            const lastUsedDiff = new Date(b.last_used || 0).getTime() - new Date(a.last_used || 0).getTime();
            if (lastUsedDiff !== 0) return lastUsedDiff;
            return (b.use_count || 0) - (a.use_count || 0);
        })
        .slice(0, 10);

    if (recent.length === 0) {
        container.innerHTML = '<div class="empty-hint">No recent items</div>';
        return;
    }

    container.innerHTML = recent.map(script => `
        <span class="recent-item" data-id="${script.id}">${script.is_favorite ? '<span class="favorite-badge">★</span>' : ''}${Utils.escapeHtml(script.name)}</span>
    `).join('');

    container.querySelectorAll('.recent-item').forEach(item => {
        item.addEventListener('click', () => {
            const script = AppState.scripts.find(s => s.id === item.dataset.id);
            if (script) selectScript(script);
        });
    });
}

export function renderRecentTemplates() {
    const container = document.getElementById('recentTemplates').querySelector('.recent-scroll') ||
                     document.getElementById('recentTemplates');

    const recent = [...AppState.templates]
        .sort((a, b) => {
            if ((b.is_favorite ? 1 : 0) !== (a.is_favorite ? 1 : 0)) {
                return (b.is_favorite ? 1 : 0) - (a.is_favorite ? 1 : 0);
            }
            const lastUsedDiff = new Date(b.last_used || 0).getTime() - new Date(a.last_used || 0).getTime();
            if (lastUsedDiff !== 0) return lastUsedDiff;
            return (b.use_count || 0) - (a.use_count || 0);
        })
        .slice(0, 10);

    if (recent.length === 0) {
        container.innerHTML = '<div class="empty-hint">No recent items</div>';
        return;
    }

    container.innerHTML = recent.map(template => `
        <span class="recent-item" data-id="${template.id}">${template.is_favorite ? '<span class="favorite-badge">★</span>' : ''}${Utils.escapeHtml(template.name)}</span>
    `).join('');

    container.querySelectorAll('.recent-item').forEach(item => {
        item.addEventListener('click', () => {
            const template = AppState.templates.find(t => t.id === item.dataset.id);
            if (template) selectTemplate(template);
        });
    });
}
