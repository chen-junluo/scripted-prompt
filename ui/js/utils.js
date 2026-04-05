export const Utils = {
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text || '';
        return div.innerHTML;
    },

    formatTimestamp(value) {
        if (!value) return 'N/A';
        const date = new Date(value);
        if (Number.isNaN(date.getTime())) return value;
        const pad = (num) => String(num).padStart(2, '0');
        return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
    },

    showStatus(message, type = 'normal') {
        const statusBar = document.getElementById('statusBar');
        statusBar.textContent = message;
        statusBar.className = 'status-bar';
        if (type === 'success') statusBar.classList.add('success');
        if (type === 'error') statusBar.classList.add('error');
    },

    hideAllViews() {
        document.getElementById('scriptEditorMode').classList.add('hidden');
        document.getElementById('compositionPreviewMode').classList.add('hidden');
        document.getElementById('emptyState').classList.add('hidden');
    },

    showView(viewId) {
        Utils.hideAllViews();
        if (viewId === 'empty') {
            document.getElementById('emptyState').classList.remove('hidden');
        } else {
            document.getElementById(viewId).classList.remove('hidden');
        }
    },
};
