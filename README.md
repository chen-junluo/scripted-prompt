# Scripted Prompt

[中文说明](README.zh-CN.md)

Prompts work better as building blocks.

Scripted Prompt is a local desktop app for collecting, organizing, and composing reusable prompt blocks. It is for people who write prompts often and want a faster way to reuse structure, variables, and sequences.

## Start here

Use the app from **GitHub Releases**.

Recommended downloads:
- macOS: `.dmg`
- Windows: `.msi`

## macOS note

The current macOS build is unsigned.

If macOS says the app is damaged or refuses to open it:

1. Open the `.dmg`
2. Drag `Scripted Prompt.app` to `Applications`
3. Run:

```bash
xattr -dr com.apple.quarantine "/Applications/Scripted Prompt.app"
open "/Applications/Scripted Prompt.app"
```

This release path is fine for developer users and small tests. It is not yet the no-warning macOS install path.

If you only want to use the app, start from Releases, not from local build steps.

## What this does

Scripted Prompt gives you two core units:

- **Script**: one reusable prompt block
- **Template**: an ordered combination of Scripts

This lets you:
- keep small prompt units instead of one long note
- reuse shared variables once across a larger prompt
- assemble repeatable prompt workflows
- keep everything local on your own machine

## Use it for

- keeping reusable role, task, and output-format blocks
- building prompt workflows from smaller pieces
- saving common review, writing, and coding prompt sequences
- exporting and importing local prompt libraries

## First use

1. Open the repository on GitHub
2. Go to **Releases**
3. Download the installer for your platform
4. Install the app
5. Create a Script, then combine several Scripts into a Template

## How it works

A **Script** has:
- a name
- hierarchical tags, such as `writing/academic/outline`
- prompt content
- optional variables, such as `{{topic}}` or `{{tone:formal}}`

A **Template** lets you:
- combine Scripts in order
- fill shared variables once
- preview the final prompt
- save the composition for reuse

## What you get

- three-panel desktop interface
- separate trees for Scripts and Templates
- favorites and recent items
- variable parsing with defaults
- local JSON storage
- import and export
- desktop packaging through Tauri

## Build from source

This section is for contributors and local development.

Requirements:
- Node.js
- Rust toolchain
- Cargo
- platform build tools required by Tauri

Run locally:

```bash
npm install
cargo tauri dev
```

Test:

```bash
npm test
```

Release build:

```bash
npm run build:release
```

Platform-specific builds:

```bash
npm run build:mac
npm run build:windows
npm run build:linux
```

See [BUILD_GUIDE.md](BUILD_GUIDE.md) if you want to build from source.

## Release files

Typical outputs:
- macOS: `.dmg`
- Windows: `.msi` and `.exe`
- Linux: `.AppImage` and `.deb`

For normal use, download these files from **GitHub Releases**.

## Scope

Scripted Prompt is for local prompt management and composition.

It is not:
- a cloud sync service
- a hosted prompt marketplace
- a collaborative prompt editor

## Design choices

- local-first storage
- reusable prompt blocks instead of long prompt documents
- desktop distribution through installers, not browser-first usage

## Data storage

Runtime data is stored on the user machine.

Typical locations:
- macOS: `~/Library/Application Support/scripted-prompt/`
- Linux: `~/.local/share/scripted-prompt/`
- Windows: `%APPDATA%\\scripted-prompt\\`

The app initializes default data files if they do not exist.

## Notes

- After UI changes, clear the Tauri build cache before a fresh release build. See [DEPLOY.md](DEPLOY.md).
- Production builds disable Tauri devtools by default.
- Default app data is generated at runtime. Local `data/` files are not required for public release.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
