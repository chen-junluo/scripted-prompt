# Scripted Prompt

[English README](README.md)

Prompt 更适合做成可组合的积木。

Scripted Prompt 是一个本地桌面应用，用来收藏、整理和组合可复用的 prompt 片段。它适合经常写 prompt、希望复用结构、变量和流程的人。

## 从这里开始

普通用户请直接前往 **GitHub Releases** 下载应用。

推荐下载：
- macOS：`.dmg`
- Windows：`.msi`

## macOS 说明

当前 macOS 版本还没有做 Apple 签名。

如果 macOS 提示应用已损坏，或者不允许打开，请这样做：

1. 打开 `.dmg`
2. 把 `Scripted Prompt.app` 拖到 `Applications`
3. 运行：

```bash
xattr -dr com.apple.quarantine "/Applications/Scripted Prompt.app"
open "/Applications/Scripted Prompt.app"
```

这个发布方式适合开发者用户和小范围测试。它还不是面向普通用户的无提示安装路径。

如果你只是想使用应用，不必本地构建。

## 它能做什么

Scripted Prompt 有两个核心单元：

- **Script**：一个可复用的 prompt 片段
- **Template**：多个 Script 的有序组合

加上 AI compression 之后，这个 workflow 变成了双向循环：

- 你可以把多个 Script 组合成 Template
- 也可以把一个 Template 再压缩成新的 Script

这样更适合持续整理 prompt 库。你不用只会越堆越长，也可以把成熟流程重新收敛成更干净的积木。

## 适合用来做什么

- 收藏可复用的角色、任务和输出格式片段
- 用多个小片段拼出完整 prompt 流程
- 把已经变长的 Template 压缩成一个更干净的 Script
- 随着工作方式变化，持续整理 prompt 库
- 导入和导出本地 prompt 库

## 首次使用

1. 打开 GitHub 仓库
2. 进入 **Releases** 页面
3. 下载对应平台的安装包
4. 安装应用
5. 先创建一个 Script，再把多个 Script 组合成 Template
6. 当某个 Template 已经稳定下来后，用 AI 把它压缩成新的 Script

这样就形成一个正向循环：Script → Template → Script。

## 工作方式

一个 **Script** 包含：
- 名称
- 层级标签，例如 `writing/academic/outline`
- prompt 内容
- 可选变量，例如 `{{topic}}` 或 `{{tone:formal}}`

一个 **Template** 可以：
- 按顺序组合多个 Script
- 共享变量只填一次
- 预览最终 prompt
- 保存组合结果，方便复用

加上 AI compression 之后，一个 Template 在审阅结果后，也可以再变回新的 Script。

## 从源码构建

这一部分面向贡献者和本地开发。

环境要求：
- Node.js
- Rust toolchain
- Cargo
- Tauri 所需的平台构建工具

本地运行：

```bash
npm install
cargo tauri dev
```

测试：

```bash
npm test
```

发布构建：

```bash
npm run build:release
```

按平台构建：

```bash
npm run build:mac
npm run build:windows
npm run build:linux
```

## 发布文件

常见输出：
- macOS：`.dmg`
- Windows：`.msi` 和 `.exe`

普通使用请直接从 **GitHub Releases** 下载。

## 边界

Scripted Prompt 用于本地 prompt 管理、组合和整理。

它不是：
- 云同步服务
- 在线 prompt 市场
- 多人协作编辑器
- 内置托管模型的 AI 平台

## 设计取舍

- 本地优先存储
- 用可复用 prompt 片段代替长 prompt 文档
- 让 Script 和 Template 之间可以形成循环，而不是单向堆叠
- 用桌面安装包分发，而不是优先走网页使用

## 数据存储

运行时数据保存在用户本机。

常见路径：
- macOS：`~/Library/Application Support/scripted-prompt/`
- Linux：`~/.local/share/scripted-prompt/`
- Windows：`%APPDATA%\scripted-prompt\`

如果数据文件不存在，应用会自动初始化默认数据。

AI settings 会保存在本地 `settings.json`。导出数据时不会包含 API key。

## 说明

- 正式版默认关闭 Tauri devtools。
- 默认应用数据在运行时生成，公开仓库不需要提交本地 `data/` 文件。

## 许可证

AGPL-3.0-or-later。见 [LICENSE](LICENSE)。
