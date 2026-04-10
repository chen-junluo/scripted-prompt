const STORAGE_KEY = 'scripted-prompt-pane-layout';
const NARROW_BREAKPOINT = 1180;
const DIVIDER_SIZE = 12;
const MIN_WIDE_LEFT = 220;
const MIN_WIDE_MIDDLE = 260;
const MIN_WIDE_RIGHT = 300;
const MIN_NARROW_LEFT = 220;
const MIN_NARROW_RIGHT = 260;
const MIN_TOP_HEIGHT = 260;
const MIN_BOTTOM_HEIGHT = 240;

const defaultLayout = {
    wide: {
        left: 0.3,
        middle: 0.33,
        right: 0.37,
    },
    narrow: {
        left: 0.45,
        right: 0.55,
        bottomHeight: 0.4,
    },
};

function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
}

function parseStoredLayout() {
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) return null;
        const parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== 'object') return null;
        return parsed;
    } catch {
        return null;
    }
}

function loadLayoutState() {
    const stored = parseStoredLayout();
    return {
        wide: {
            left: clamp(Number(stored?.wide?.left) || defaultLayout.wide.left, 0.15, 0.7),
            middle: clamp(Number(stored?.wide?.middle) || defaultLayout.wide.middle, 0.15, 0.7),
            right: clamp(Number(stored?.wide?.right) || defaultLayout.wide.right, 0.15, 0.7),
        },
        narrow: {
            left: clamp(Number(stored?.narrow?.left) || defaultLayout.narrow.left, 0.2, 0.8),
            right: clamp(Number(stored?.narrow?.right) || defaultLayout.narrow.right, 0.2, 0.8),
            bottomHeight: clamp(Number(stored?.narrow?.bottomHeight) || defaultLayout.narrow.bottomHeight, 0.2, 0.8),
        },
    };
}

function saveLayoutState(layout) {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(layout));
}

function sum(values) {
    return values.reduce((total, value) => total + value, 0);
}

function normalizeWide(layout) {
    const total = sum([layout.left, layout.middle, layout.right]) || 1;
    layout.left /= total;
    layout.middle /= total;
    layout.right /= total;
}

function normalizeNarrow(layout) {
    const total = sum([layout.left, layout.right]) || 1;
    layout.left /= total;
    layout.right /= total;
}

function applyWideLayout(container, layout) {
    normalizeWide(layout);
    container.style.setProperty('--pane-left', `${layout.left}fr`);
    container.style.setProperty('--pane-middle', `${layout.middle}fr`);
    container.style.setProperty('--pane-right', `${layout.right}fr`);
}

function applyNarrowLayout(container, layout) {
    normalizeNarrow(layout);
    container.style.setProperty('--top-left', `${layout.left}fr`);
    container.style.setProperty('--top-right', `${layout.right}fr`);
    container.style.setProperty('--bottom-pane-height', `${(layout.bottomHeight * 100).toFixed(2)}%`);
}

function getLayoutMode(container) {
    return container.clientWidth < NARROW_BREAKPOINT ? 'narrow' : 'wide';
}

function setResizeState(active, direction) {
    document.body.classList.toggle('is-resizing', active);
    if (direction) {
        document.body.style.cursor = direction === 'row' ? 'row-resize' : 'col-resize';
    } else {
        document.body.style.cursor = '';
    }
}

function getAvailableWideWidth(rect) {
    return rect.width - DIVIDER_SIZE * 2;
}

function getAvailableNarrowWidth(rect) {
    return rect.width - DIVIDER_SIZE;
}

function getAvailableNarrowHeight(rect) {
    return rect.height - DIVIDER_SIZE;
}

export function initPaneLayout() {
    const container = document.getElementById('appContainer');
    const leftDivider = document.getElementById('leftPaneDivider');
    const rightDivider = document.getElementById('rightPaneDivider');
    const bottomDivider = document.getElementById('bottomPaneDivider');

    if (!container || !leftDivider || !rightDivider || !bottomDivider) {
        return;
    }

    const layout = loadLayoutState();
    let mode = getLayoutMode(container);

    function applyLayout() {
        mode = getLayoutMode(container);
        container.classList.toggle('is-wide', mode === 'wide');
        container.classList.toggle('is-narrow', mode === 'narrow');
        applyWideLayout(container, layout.wide);
        applyNarrowLayout(container, layout.narrow);
    }

    function persistLayout() {
        saveLayoutState(layout);
    }

    function startDrag(divider, direction, getInitialState, updateLayoutFromDelta, isAllowed) {
        divider.addEventListener('pointerdown', (event) => {
            if (event.button !== 0 || (isAllowed && !isAllowed())) {
                return;
            }

            event.preventDefault();
            const initial = getInitialState();
            const startX = event.clientX;
            const startY = event.clientY;

            divider.setPointerCapture(event.pointerId);
            container.classList.add('is-resizing');
            setResizeState(true, direction);

            const handlePointerMove = (moveEvent) => {
                const deltaX = moveEvent.clientX - startX;
                const deltaY = moveEvent.clientY - startY;
                updateLayoutFromDelta(deltaX, deltaY, initial);
                applyLayout();
            };

            const stopDragging = () => {
                divider.removeEventListener('pointermove', handlePointerMove);
                divider.removeEventListener('pointerup', handlePointerUp);
                divider.removeEventListener('pointercancel', handlePointerUp);
                divider.removeEventListener('lostpointercapture', handlePointerUp);
                container.classList.remove('is-resizing');
                setResizeState(false);
                persistLayout();
            };

            const handlePointerUp = () => {
                stopDragging();
            };

            divider.addEventListener('pointermove', handlePointerMove);
            divider.addEventListener('pointerup', handlePointerUp);
            divider.addEventListener('pointercancel', handlePointerUp);
            divider.addEventListener('lostpointercapture', handlePointerUp);
        });
    }

    startDrag(
        leftDivider,
        'col',
        () => {
            const rect = container.getBoundingClientRect();
            if (mode === 'wide') {
                const totalWidth = getAvailableWideWidth(rect);
                return {
                    totalWidth,
                    leftWidth: layout.wide.left * totalWidth,
                    middleWidth: layout.wide.middle * totalWidth,
                    rightWidth: layout.wide.right * totalWidth,
                };
            }

            const totalWidth = getAvailableNarrowWidth(rect);
            return {
                totalWidth,
                leftWidth: layout.narrow.left * totalWidth,
            };
        },
        (deltaX, _deltaY, initial) => {
            if (mode === 'wide') {
                const nextLeft = clamp(initial.leftWidth + deltaX, MIN_WIDE_LEFT, initial.totalWidth - MIN_WIDE_MIDDLE - initial.rightWidth);
                const nextMiddle = initial.totalWidth - initial.rightWidth - nextLeft;
                layout.wide.left = nextLeft / initial.totalWidth;
                layout.wide.middle = nextMiddle / initial.totalWidth;
                layout.wide.right = initial.rightWidth / initial.totalWidth;
                return;
            }

            const nextLeft = clamp(initial.leftWidth + deltaX, MIN_NARROW_LEFT, initial.totalWidth - MIN_NARROW_RIGHT);
            const nextRight = initial.totalWidth - nextLeft;
            layout.narrow.left = nextLeft / initial.totalWidth;
            layout.narrow.right = nextRight / initial.totalWidth;
        }
    );

    startDrag(
        rightDivider,
        'col',
        () => {
            const rect = container.getBoundingClientRect();
            const totalWidth = getAvailableWideWidth(rect);
            return {
                totalWidth,
                leftWidth: layout.wide.left * totalWidth,
                middleWidth: layout.wide.middle * totalWidth,
                rightWidth: layout.wide.right * totalWidth,
            };
        },
        (deltaX, _deltaY, initial) => {
            const nextMiddle = clamp(initial.middleWidth + deltaX, MIN_WIDE_MIDDLE, initial.totalWidth - initial.leftWidth - MIN_WIDE_RIGHT);
            const nextRight = initial.totalWidth - initial.leftWidth - nextMiddle;
            layout.wide.left = initial.leftWidth / initial.totalWidth;
            layout.wide.middle = nextMiddle / initial.totalWidth;
            layout.wide.right = nextRight / initial.totalWidth;
        },
        () => mode === 'wide'
    );

    startDrag(
        bottomDivider,
        'row',
        () => {
            const rect = container.getBoundingClientRect();
            const totalHeight = getAvailableNarrowHeight(rect);
            return {
                totalHeight,
                bottomHeight: layout.narrow.bottomHeight * totalHeight,
            };
        },
        (_deltaX, deltaY, initial) => {
            const nextBottom = clamp(initial.bottomHeight - deltaY, MIN_BOTTOM_HEIGHT, initial.totalHeight - MIN_TOP_HEIGHT);
            layout.narrow.bottomHeight = nextBottom / initial.totalHeight;
        },
        () => mode === 'narrow'
    );

    window.addEventListener('resize', applyLayout);
    applyLayout();
}
