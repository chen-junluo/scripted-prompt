#!/usr/bin/env node

/**
 * Scripted Prompt 自动化分发脚本
 *
 * 使用方法:
 *   node scripts/build-release.js              # 为当前平台构建
 *   node scripts/build-release.js --all        # 为所有支持的平台构建
 *   node scripts/build-release.js --help       # 显示帮助
 *
 * 要求:
 *   - Node.js 14+
 *   - Rust 工具链（用于 Tauri）
 *   - 在 macOS 上：Xcode Command Line Tools
 *   - 在 Windows 上：Visual Studio Build Tools（可选，Tauri 会指导安装）
 */

const fs = require('fs');
const path = require('path');
const { execSync, spawnSync } = require('child_process');
const os = require('os');

// 颜色输出
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  blue: '\x1b[34m',
};

function log(type, message) {
  const timestamp = new Date().toLocaleTimeString();
  const prefix = {
    info: `${colors.blue}[INFO]${colors.reset}`,
    success: `${colors.green}[✓]${colors.reset}`,
    warn: `${colors.yellow}[⚠]${colors.reset}`,
    error: `${colors.red}[✗]${colors.reset}`,
  }[type] || `[${type}]`;

  console.log(`${prefix} [${timestamp}] ${message}`);
}

function logSection(title) {
  console.log(`\n${colors.bright}${colors.blue}${'='.repeat(60)}${colors.reset}`);
  console.log(`${colors.bright}${colors.blue}  ${title}${colors.reset}`);
  console.log(`${colors.bright}${colors.blue}${'='.repeat(60)}${colors.reset}\n`);
}

function showHelp() {
  console.log(`
${colors.bright}Scripted Prompt 自动化分发脚本${colors.reset}

${colors.bright}用法:${colors.reset}
  node scripts/build-release.js [选项]

${colors.bright}选项:${colors.reset}
  --all                构建所有支持的平台（需要分别在各平台运行）
  --mac                构建 macOS 版本（仅 macOS）
  --windows            构建 Windows 版本（仅 Windows）
  --linux              构建 Linux 版本（仅 Linux）
  --no-bundle          仅构建可执行文件，不生成安装程序
  --version            显示版本号
  --help               显示此帮助信息

${colors.bright}示例:${colors.reset}
  # 构建当前平台
  node scripts/build-release.js

  # 仅构建可执行文件（调试）
  node scripts/build-release.js --no-bundle

  # 在 macOS 上构建
  node scripts/build-release.js --mac

${colors.bright}注意事项:${colors.reset}
  • 跨平台构建需要在对应的操作系统上运行
  • 构建产物位于 src-tauri/target/release/bundle 目录下
  • 分发版本会保留开发者工具（F12 / 右键检查元素）
  `);
}

function getPlatform() {
  const platform = os.platform();
  if (platform === 'darwin') return 'macos';
  if (platform === 'win32') return 'windows';
  if (platform === 'linux') return 'linux';
  return 'unknown';
}

function getInstallationInstructions(platform, targetDir) {
  const bundlePath = path.join(targetDir, 'release/bundle');
  const instructions = {
    macos: `
${colors.bright}macOS 安装说明:${colors.reset}
  1. 生成的文件位置：
     - .dmg 文件：${path.join(bundlePath, 'dmg')}
     - .app 文件：${path.join(bundlePath, 'macos')}

  2. 分发方式：
     方式1 (推荐): 分发 .dmg 文件
       用户双击 .dmg 文件，选择拖拽 Scripted Prompt.app 到 Applications 文件夹

     方式2: 分发 .app 文件
       用户可直接从文件中运行，或复制到 Applications 文件夹

  3. 签名和公证 (可选):
     若要分发给其他用户，建议对应用进行代码签名和公证。
     参考: https://tauri.app/en/v1/guides/distribution/sign-macos
    `,
    windows: `
${colors.bright}Windows 安装说明:${colors.reset}
  1. 生成的文件位置：
     - .msi 文件：${path.join(bundlePath, 'msi')}
     - .exe 文件：${path.join(bundlePath, 'nsis')}

  2. 分发方式：
     方式1 (推荐): 分发 .msi 文件
       用户双击 .msi 文件，按照向导完成安装

     方式2: 分发 NSIS 安装程序
       用户运行 .exe 安装程序，可自定义安装目录

  3. 数字签名 (可选):
     若要分发给其他用户，建议对应用进行数字签名。
     参考: https://tauri.app/en/v1/guides/distribution/sign-windows
    `,
    linux: `
${colors.bright}Linux 安装说明:${colors.reset}
  1. 生成的文件位置：
     - AppImage 文件：${path.join(bundlePath, 'appimage')}
     - .deb 文件：${path.join(bundlePath, 'deb')}

  2. 分发方式：
     方式1: AppImage (推荐)
       chmod +x Scripted\ Prompt.AppImage
       ./Scripted\ Prompt.AppImage

     方式2: .deb 包
       sudo apt install ./scripted-prompt_*.deb
    `,
  };

  return instructions[platform] || '';
}

function setupRustEnvironment() {
  // 添加 Rust 路径到 PATH
  const cargoPath = path.join(os.homedir(), '.cargo', 'bin');
  if (fs.existsSync(cargoPath)) {
    process.env.PATH = `${cargoPath}${path.delimiter}${process.env.PATH}`;
  }
}

function checkPrerequisites() {
  logSection('检查环境');

  const platform = getPlatform();
  log('info', `当前平台: ${platform.toUpperCase()}`);

  // 设置 Rust 环境
  setupRustEnvironment();

  // 检查 Node.js
  try {
    const nodeVersion = execSync('node --version', { encoding: 'utf8' }).trim();
    log('success', `Node.js: ${nodeVersion}`);
  } catch (e) {
    log('error', 'Node.js 未安装');
    process.exit(1);
  }

  // 检查 Rust
  try {
    const rustVersion = execSync('rustc --version', { encoding: 'utf8' }).trim();
    log('success', `Rust: ${rustVersion}`);
  } catch (e) {
    log('error', 'Rust 未安装或未在 PATH 中。请访问 https://rustup.rs/ 安装');
    log('info', `提示: 尝试运行 'source $HOME/.cargo/env' 或重启终端`);
    process.exit(1);
  }

  // 检查 Cargo
  try {
    const cargoVersion = execSync('cargo --version', { encoding: 'utf8' }).trim();
    log('success', `Cargo: ${cargoVersion}`);
  } catch (e) {
    log('error', 'Cargo 未安装或未在 PATH 中');
    process.exit(1);
  }

  // 检查 Tauri CLI
  try {
    execSync('npm list @tauri-apps/cli', { encoding: 'utf8', stdio: 'ignore' });
    log('success', '@tauri-apps/cli: 已安装');
  } catch (e) {
    log('warn', '@tauri-apps/cli 未安装，将使用 cargo tauri 命令');
  }

  log('success', '环境检查完成\n');
}

function verifyConfiguration() {
  logSection('验证配置');

  // 检查 tauri.conf.json
  const configPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
  if (!fs.existsSync(configPath)) {
    log('error', 'tauri.conf.json 不存在');
    process.exit(1);
  }

  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

  log('success', `应用名称: ${config.productName}`);
  log('success', `版本: ${config.version}`);
  log('success', `开发工具: ${config.app.windows[0].devtools ? '启用' : '禁用'}`);

  if (!config.app.windows[0].devtools) {
    log('warn', '开发工具已禁用。构建后将无法使用 F12 和右键检查元素');
  }

  const targets = config.bundle.targets;
  log('success', `构建目标: ${targets.join(', ')}`);

  log('success', '配置验证完成\n');
}

function getMacosTargetDir() {
  return path.join(os.tmpdir(), 'scripted-prompt-target');
}

function getTargetDir(options = {}) {
  const { platform = getPlatform(), noBundle = false } = options;

  if (platform === 'macos' && !noBundle) {
    return getMacosTargetDir();
  }

  return path.join(__dirname, '../src-tauri/target');
}

function patchMacosDmgBundler(targetDir) {
  const bundleScriptPath = path.join(targetDir, 'release/bundle/dmg/bundle_dmg.sh');
  if (!fs.existsSync(bundleScriptPath)) {
    throw new Error(`未找到 DMG 打包脚本: ${bundleScriptPath}`);
  }

  const script = fs.readFileSync(bundleScriptPath, 'utf8');
  const oldBlock = [
    'if [[ -n "$VOLUME_ICON_FILE" ]]; then',
    '\techo "Copying volume icon file \'$VOLUME_ICON_FILE\'..."',
    '\tcp "$VOLUME_ICON_FILE" "$MOUNT_DIR/.VolumeIcon.icns"',
    '\tSetFile -c icnC "$MOUNT_DIR/.VolumeIcon.icns"',
    'fi'
  ].join('\n');
  const newBlock = [
    'if [[ -n "$VOLUME_ICON_FILE" ]]; then',
    '\techo "Skipping volume icon copy to keep DMG root clean..."',
    'fi'
  ].join('\n');

  if (!script.includes(oldBlock)) {
    if (script.includes(newBlock)) {
      log('info', 'DMG 打包脚本已是干净版本');
      return;
    }
    throw new Error('未找到可替换的 volume icon 复制逻辑');
  }

  fs.writeFileSync(bundleScriptPath, script.replace(oldBlock, newBlock));
  log('success', '已修补 DMG 打包脚本，跳过 .VolumeIcon.icns');
}

function buildCleanMacosDmg(targetDir) {
  const bundleRoot = path.join(targetDir, 'release/bundle');
  const dmgDir = path.join(bundleRoot, 'dmg');
  const macosDir = path.join(bundleRoot, 'macos');
  const appName = 'Scripted Prompt.app';
  const appSource = path.join(macosDir, appName);
  const dmgScriptPath = path.join(dmgDir, 'bundle_dmg.sh');
  const dmgBackgroundPath = path.join(__dirname, '../src-tauri/icons/dmg-background-clean.png');
  const tempSourceDir = path.join(os.tmpdir(), 'scripted-prompt-dmg-clean');
  const appTarget = path.join(tempSourceDir, appName);

  if (!fs.existsSync(appSource)) {
    throw new Error(`未找到应用包: ${appSource}`);
  }
  if (!fs.existsSync(dmgScriptPath)) {
    throw new Error(`未找到 DMG 打包脚本: ${dmgScriptPath}`);
  }

  logSection('重建 macOS Clean DMG');

  fs.rmSync(tempSourceDir, { recursive: true, force: true });
  fs.mkdirSync(tempSourceDir, { recursive: true });
  fs.cpSync(appSource, appTarget, { recursive: true });

  const existingDmgs = fs.readdirSync(dmgDir).filter(f => f.endsWith('.dmg'));
  existingDmgs.forEach(file => fs.rmSync(path.join(dmgDir, file), { force: true }));

  const configPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  const dmgConfig = config.bundle?.macOS?.dmg || {};
  const width = dmgConfig.windowSize?.width || 660;
  const height = dmgConfig.windowSize?.height || 400;
  const appX = dmgConfig.appPosition?.x || 180;
  const appY = dmgConfig.appPosition?.y || 170;
  const applicationsX = dmgConfig.applicationFolderPosition?.x || 480;
  const applicationsY = dmgConfig.applicationFolderPosition?.y || 170;
  const arch = os.arch() === 'arm64' ? 'aarch64' : os.arch();
  const dmgName = `${config.productName}_${config.version}_${arch}.dmg`;
  const dmgOutput = path.join(dmgDir, dmgName);

  log('info', `执行命令: ${path.basename(dmgScriptPath)} --background ${path.basename(dmgBackgroundPath)} ...`);

  const result = spawnSync('bash', [dmgScriptPath,
    '--volname', config.productName,
    '--background', dmgBackgroundPath,
    '--window-size', String(width), String(height),
    '--icon-size', '128',
    '--icon', appName, String(appX), String(appY),
    '--app-drop-link', String(applicationsX), String(applicationsY),
    dmgOutput,
    tempSourceDir,
  ], {
    cwd: dmgDir,
    stdio: 'inherit',
  });

  fs.rmSync(tempSourceDir, { recursive: true, force: true });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    throw new Error(`重建 clean DMG 失败，退出码: ${result.status}`);
  }

  log('success', `macOS clean DMG 已生成: ${dmgOutput}`);
}

function ensureMacosBundleFromExistingArtifacts(projectRoot, targetDir) {
  const sourceApp = path.join(projectRoot, 'src-tauri/target/release/bundle/macos/Scripted Prompt.app');
  const fallbackBinary = path.join(targetDir, 'release/scripted-prompt');
  const targetApp = path.join(targetDir, 'release/bundle/macos/Scripted Prompt.app');
  const targetMacosDir = path.dirname(targetApp);

  fs.mkdirSync(targetMacosDir, { recursive: true });

  if (fs.existsSync(sourceApp)) {
    fs.rmSync(targetApp, { recursive: true, force: true });
    fs.cpSync(sourceApp, targetApp, { recursive: true });
    log('success', '已复用现有 .app 产物');
    return;
  }

  if (!fs.existsSync(fallbackBinary)) {
    throw new Error(`未找到 release 可执行文件: ${fallbackBinary}`);
  }

  throw new Error('未找到可复用的 macOS .app 产物，请先在本机成功生成一次 .app');
}

function buildProject(options = {}) {
  const {
    noBundle = false,
    platform = getPlatform(),
  } = options;

  logSection(`开始构建 ${platform.toUpperCase()}`);

  const projectRoot = path.join(__dirname, '..');
  const targetDir = getTargetDir({ platform, noBundle });
  const env = { ...process.env, CARGO_TARGET_DIR: targetDir };

  try {
    const args = ['tauri', 'build'];

    if (noBundle) {
      args.push('--no-bundle');
    }

    log('info', `执行命令: cargo ${args.join(' ')}`);
    if (platform === 'macos' && !noBundle) {
      log('info', `使用本地 target 目录: ${targetDir}`);
    }
    console.log('');

    const result = spawnSync('cargo', args, {
      cwd: path.join(projectRoot, 'src-tauri'),
      stdio: 'inherit',
      shell: true,
      env,
    });

    if (platform === 'macos' && !noBundle && result.status !== 0) {
      log('warn', '标准 Tauri 打包失败，改为保留 release 可执行文件并手动重建 macOS 产物');
      const fallbackResult = spawnSync('cargo', ['build', '--release'], {
        cwd: path.join(projectRoot, 'src-tauri'),
        stdio: 'inherit',
        shell: true,
        env,
      });

      if (fallbackResult.error) {
        log('error', `构建失败: ${fallbackResult.error.message}`);
        process.exit(1);
      }

      if (fallbackResult.status !== 0) {
        log('error', `构建失败，退出码: ${fallbackResult.status}`);
        process.exit(1);
      }

      ensureMacosBundleFromExistingArtifacts(projectRoot, targetDir);
      patchMacosDmgBundler(targetDir);
      buildCleanMacosDmg(targetDir);

      console.log('');
      log('success', `${platform.toUpperCase()} 构建完成`);
      console.log(getInstallationInstructions(platform, targetDir));

      return targetDir;
    }

    if (result.error) {
      log('error', `构建失败: ${result.error.message}`);
      process.exit(1);
    }

    if (result.status !== 0) {
      log('error', `构建失败，退出码: ${result.status}`);
      process.exit(1);
    }

    if (platform === 'macos' && !noBundle) {
      patchMacosDmgBundler(targetDir);
      buildCleanMacosDmg(targetDir);
    }

    console.log('');
    log('success', `${platform.toUpperCase()} 构建完成`);
    console.log(getInstallationInstructions(platform, targetDir));

    return targetDir;
  } catch (error) {
    log('error', `构建过程出错: ${error.message}`);
    process.exit(1);
  }
}

function createDistributionReport(platform, targetDir) {
  logSection('构建产物信息');

  const bundlePath = path.join(targetDir, 'release/bundle');

  const platformPaths = {
    macos: ['dmg', 'macos'],
    windows: ['msi', 'nsis'],
    linux: ['appimage', 'deb'],
  };

  const paths = platformPaths[platform] || [];

  paths.forEach(subdir => {
    const fullPath = path.join(bundlePath, subdir);
    if (fs.existsSync(fullPath)) {
      const files = fs.readdirSync(fullPath);
      if (files.length > 0) {
        log('success', `${subdir}:`);
        files.forEach(file => {
          if (!file.startsWith('.')) {
            const filePath = path.join(fullPath, file);
            const stat = fs.statSync(filePath);
            const size = (stat.size / 1024 / 1024).toFixed(2);
            console.log(`  └─ ${file} (${size} MB)`);
          }
        });
      }
    }
  });

  console.log('');
}

// 主程序
function main() {
  const args = process.argv.slice(2);

  if (args.includes('--help') || args.includes('-h')) {
    showHelp();
    process.exit(0);
  }

  if (args.includes('--version')) {
    const configPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
    const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
    console.log(`Scripted Prompt v${config.version}`);
    process.exit(0);
  }

  const noBundle = args.includes('--no-bundle');
  const platform = getPlatform();
  const targetDir = getTargetDir({ platform, noBundle });

  console.log(`
${colors.bright}╔════════════════════════════════════════════════════════════╗${colors.reset}
${colors.bright}║     Scripted Prompt 自动化分发工具 v1.0                  ║${colors.reset}
${colors.bright}╚════════════════════════════════════════════════════════════╝${colors.reset}
  `);

  checkPrerequisites();
  verifyConfiguration();
  const buildTargetDir = buildProject({ noBundle, platform });
  createDistributionReport(platform, buildTargetDir);

  logSection('完成');
  log('success', '分发包已生成！');
  log('info', '下一步:');
  console.log(`  1. 打开文件夹: ${path.join(buildTargetDir, 'release/bundle')}`);
  console.log(`  2. 查找您的安装程序文件`);
  console.log(`  3. 分发给用户\n`);
}

main();
