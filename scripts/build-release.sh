#!/bin/bash

# Scripted Prompt 自动化分发脚本 (Shell 版本)
#
# 使用方法:
#   ./scripts/build-release.sh              # 为当前平台构建
#   ./scripts/build-release.sh --help       # 显示帮助
#
# 要求:
#   - macOS: Xcode Command Line Tools
#   - Linux: build-essential, libssl-dev
#   - 已安装 Rust 和 Cargo

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%H:%M:%S') $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $(date '+%H:%M:%S') $1"
}

log_warn() {
    echo -e "${YELLOW}[⚠]${NC} $(date '+%H:%M:%S') $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $(date '+%H:%M:%S') $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}============================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}============================================================${NC}"
    echo ""
}

show_help() {
    cat << EOF
${GREEN}Scripted Prompt 自动化分发脚本${NC}

${GREEN}用法:${NC}
  ./scripts/build-release.sh [选项]

${GREEN}选项:${NC}
  --no-bundle          仅构建可执行文件，不生成安装程序
  --help               显示此帮助信息

${GREEN}示例:${NC}
  # 构建当前平台
  ./scripts/build-release.sh

  # 仅构建可执行文件（调试）
  ./scripts/build-release.sh --no-bundle

${GREEN}注意事项:${NC}
  • 构建产物位于 src-tauri/target/release/bundle 目录下
  • 分发版本会保留开发者工具（F12 / 右键检查元素）
  • 需要 Rust 工具链已安装
EOF
}

detect_platform() {
    case "$(uname -s)" in
        Darwin)
            echo "macos"
            ;;
        Linux)
            echo "linux"
            ;;
        MINGW*|MSYS*)
            echo "windows"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

setup_rust_environment() {
    # 添加 Rust 路径到 PATH
    if [ -d "$HOME/.cargo/bin" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
}

get_macos_target_dir() {
    echo "/tmp/scripted-prompt-target"
}

get_target_dir() {
    local NO_BUNDLE="$1"
    local PLATFORM="$(detect_platform)"

    if [ "$PLATFORM" = "macos" ] && [ "$NO_BUNDLE" != "true" ]; then
        get_macos_target_dir
    else
        echo "src-tauri/target"
    fi
}

patch_macos_dmg_bundler() {
    local TARGET_DIR="$1"
    local BUNDLE_SCRIPT="$TARGET_DIR/release/bundle/dmg/bundle_dmg.sh"

    if [ ! -f "$BUNDLE_SCRIPT" ]; then
        log_error "未找到 DMG 打包脚本: $BUNDLE_SCRIPT"
        exit 1
    fi

    python3 - "$BUNDLE_SCRIPT" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
script = path.read_text()
old = """if [[ -n \"$VOLUME_ICON_FILE\" ]]; then
	echo \"Copying volume icon file '$VOLUME_ICON_FILE'...\"
	cp \"$VOLUME_ICON_FILE\" \"$MOUNT_DIR/.VolumeIcon.icns\"
	SetFile -c icnC \"$MOUNT_DIR/.VolumeIcon.icns\"
fi"""
new = """if [[ -n \"$VOLUME_ICON_FILE\" ]]; then
	echo \"Skipping volume icon copy to keep DMG root clean...\"
fi"""
if old in script:
    path.write_text(script.replace(old, new, 1))
elif new not in script:
    raise SystemExit("未找到可替换的 volume icon 复制逻辑")
PY

    log_success "已修补 DMG 打包脚本，跳过 .VolumeIcon.icns"
}

build_clean_macos_dmg() {
    local TARGET_DIR="$1"
    local BUNDLE_ROOT="$TARGET_DIR/release/bundle"
    local DMG_DIR="$BUNDLE_ROOT/dmg"
    local MACOS_DIR="$BUNDLE_ROOT/macos"
    local APP_NAME="Scripted Prompt.app"
    local APP_SOURCE="$MACOS_DIR/$APP_NAME"
    local DMG_SCRIPT="$DMG_DIR/bundle_dmg.sh"
    local BACKGROUND_PATH="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/icons/dmg-background-clean.png"
    local TEMP_SOURCE_DIR="/tmp/scripted-prompt-dmg-clean"
    local PRODUCT_NAME="$(grep -o '"productName": "[^"]*' src-tauri/tauri.conf.json | cut -d'"' -f4)"
    local VERSION="$(grep -o '"version": "[^"]*' src-tauri/tauri.conf.json | head -1 | cut -d'"' -f4)"
    local ARCH="$(uname -m)"
    local DMG_NAME="${PRODUCT_NAME}_${VERSION}_${ARCH}.dmg"
    local DMG_OUTPUT="$DMG_DIR/$DMG_NAME"

    if [ "$ARCH" = "arm64" ]; then
        DMG_NAME="${PRODUCT_NAME}_${VERSION}_aarch64.dmg"
        DMG_OUTPUT="$DMG_DIR/$DMG_NAME"
    fi

    if [ ! -d "$APP_SOURCE" ]; then
        log_error "未找到应用包: $APP_SOURCE"
        exit 1
    fi

    log_section "重建 macOS Clean DMG"

    rm -rf "$TEMP_SOURCE_DIR"
    mkdir -p "$TEMP_SOURCE_DIR"
    cp -R "$APP_SOURCE" "$TEMP_SOURCE_DIR/$APP_NAME"
    rm -f "$DMG_DIR"/*.dmg

    local WIDTH="$(grep -A2 '"windowSize"' src-tauri/tauri.conf.json | grep '"width"' | grep -o '[0-9]\+')"
    local HEIGHT="$(grep -A2 '"windowSize"' src-tauri/tauri.conf.json | grep '"height"' | grep -o '[0-9]\+')"
    local APP_X="$(grep -A2 '"appPosition"' src-tauri/tauri.conf.json | grep '"x"' | grep -o '[0-9]\+')"
    local APP_Y="$(grep -A2 '"appPosition"' src-tauri/tauri.conf.json | grep '"y"' | grep -o '[0-9]\+')"
    local APPLICATIONS_X="$(grep -A2 '"applicationFolderPosition"' src-tauri/tauri.conf.json | grep '"x"' | grep -o '[0-9]\+')"
    local APPLICATIONS_Y="$(grep -A2 '"applicationFolderPosition"' src-tauri/tauri.conf.json | grep '"y"' | grep -o '[0-9]\+')"

    log_info "执行命令: $(basename "$DMG_SCRIPT") --background $(basename "$BACKGROUND_PATH") ..."

    (
        cd "$DMG_DIR"
        ./bundle_dmg.sh \
            --volname "$PRODUCT_NAME" \
            --background "$BACKGROUND_PATH" \
            --window-size "$WIDTH" "$HEIGHT" \
            --icon-size 128 \
            --icon "$APP_NAME" "$APP_X" "$APP_Y" \
            --app-drop-link "$APPLICATIONS_X" "$APPLICATIONS_Y" \
            "$DMG_OUTPUT" \
            "$TEMP_SOURCE_DIR"
    )

    rm -rf "$TEMP_SOURCE_DIR"
    log_success "macOS clean DMG 已生成: $DMG_OUTPUT"
}

check_prerequisites() {
    log_section "检查环境"

    PLATFORM=$(detect_platform)
    log_info "当前平台: $(echo $PLATFORM | tr '[:lower:]' '[:upper:]')"

    # 设置 Rust 环境
    setup_rust_environment

    # 检查 Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust 未安装或未在 PATH 中。请访问 https://rustup.rs/ 安装"
        log_info "提示: 尝试运行 'source \$HOME/.cargo/env' 或重启终端"
        exit 1
    fi
    RUST_VERSION=$(rustc --version)
    log_success "$RUST_VERSION"

    # 检查 Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装或未在 PATH 中"
        exit 1
    fi
    CARGO_VERSION=$(cargo --version)
    log_success "$CARGO_VERSION"

    log_success "环境检查完成"
}

verify_configuration() {
    log_section "验证配置"

    CONFIG_PATH="src-tauri/tauri.conf.json"
    if [ ! -f "$CONFIG_PATH" ]; then
        log_error "tauri.conf.json 不存在"
        exit 1
    fi

    # 使用简单的 grep 检查，因为不能依赖 jq
    PRODUCT_NAME=$(grep -o '"productName": "[^"]*' "$CONFIG_PATH" | cut -d'"' -f4)
    VERSION=$(grep -o '"version": "[^"]*' "$CONFIG_PATH" | head -1 | cut -d'"' -f4)
    DEVTOOLS=$(grep -o '"devtools": [^,]*' "$CONFIG_PATH" | grep -o 'true\|false')

    log_success "应用名称: $PRODUCT_NAME"
    log_success "版本: $VERSION"
    log_success "开发工具: $DEVTOOLS"

    if [ "$DEVTOOLS" = "false" ]; then
        log_warn "开发工具已禁用。构建后将无法使用 F12 和右键检查元素"
    fi

    log_success "配置验证完成"
}

build_project() {
    local NO_BUNDLE="$1"
    local TARGET_DIR="$(get_target_dir "$NO_BUNDLE")"

    log_section "开始构建"

    local BUILD_CMD="cargo tauri build"
    if [ "$NO_BUNDLE" = "true" ]; then
        BUILD_CMD="$BUILD_CMD --no-bundle"
    fi

    log_info "执行命令: $BUILD_CMD"
    if [ "$(detect_platform)" = "macos" ] && [ "$NO_BUNDLE" != "true" ]; then
        log_info "使用本地 target 目录: $TARGET_DIR"
    fi
    echo ""

    (
        cd src-tauri
        CARGO_TARGET_DIR="$TARGET_DIR" eval "$BUILD_CMD"
    )

    if [ "$(detect_platform)" = "macos" ] && [ "$NO_BUNDLE" != "true" ]; then
        patch_macos_dmg_bundler "$TARGET_DIR"
        build_clean_macos_dmg "$TARGET_DIR"
    fi

    echo ""
    log_success "构建完成"
    echo "$TARGET_DIR"
}

create_distribution_report() {
    local TARGET_DIR="$1"

    log_section "构建产物信息"

    PLATFORM=$(detect_platform)
    BUNDLE_PATH="$TARGET_DIR/release/bundle"

    case $PLATFORM in
        macos)
            if [ -d "$BUNDLE_PATH/dmg" ]; then
                log_success "dmg:"
                ls -lh "$BUNDLE_PATH/dmg" | awk 'NR>1 {print "  └─ " $9 " (" $5 ")"}'
            fi
            if [ -d "$BUNDLE_PATH/macos" ]; then
                log_success "macos:"
                find "$BUNDLE_PATH/macos" -type f -name "Scripted*" -exec ls -lh {} \; | awk '{print "  └─ " $9 " (" $5 ")"}'
            fi
            ;;
        linux)
            if [ -d "$BUNDLE_PATH/appimage" ]; then
                log_success "appimage:"
                ls -lh "$BUNDLE_PATH/appimage" | awk 'NR>1 {print "  └─ " $9 " (" $5 ")"}'
            fi
            if [ -d "$BUNDLE_PATH/deb" ]; then
                log_success "deb:"
                ls -lh "$BUNDLE_PATH/deb" | awk 'NR>1 {print "  └─ " $9 " (" $5 ")"}'
            fi
            ;;
    esac

    echo ""
}

show_next_steps() {
    local TARGET_DIR="$1"

    log_section "完成"

    log_success "分发包已生成！"
    log_info "下一步:"
    echo "  1. 打开文件夹: $TARGET_DIR/release/bundle/"
    echo "  2. 查找您的安装程序文件"
    echo "  3. 分发给用户"
    echo ""
}

# 主程序
main() {
    NO_BUNDLE=false

    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-bundle)
                NO_BUNDLE=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done

    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Scripted Prompt 自动化分发工具 v1.0                  ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    check_prerequisites
    verify_configuration
    TARGET_DIR="$(build_project "$NO_BUNDLE")"
    create_distribution_report "$TARGET_DIR"
    show_next_steps "$TARGET_DIR"
}

main "$@"
