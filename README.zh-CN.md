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

这让你可以：
- 保存小的 prompt 单元，而不是维护一整段长文本
- 在更大的 prompt 中只填写一次共享变量
- 反复使用固定的 prompt 流程
- 把所有内容保留在本机

## 适合用来做什么

- 收藏可复用的角色、任务和输出格式片段
- 用多个小片段拼出完整 prompt 流程
- 保存常用的写作、评审、编码 prompt 组合
- 导入和导出本地 prompt 库

## 首次使用

1. 打开 GitHub 仓库
2. 进入 **Releases** 页面
3. 下载对应平台的安装包
4. 安装应用
5. 先创建一个 Script，再把多个 Script 组合成 Template

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

## 你会得到什么

- 三栏桌面界面
- Scripts 和 Templates 分离的树形结构
- 收藏与最近使用
- 带默认值的变量解析
- 本地 JSON 存储
- 导入与导出
- 基于 Tauri 的桌面打包

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

如果你要自己构建，再看 [BUILD_GUIDE.md](BUILD_GUIDE.md)。

## 发布文件

常见输出：
- macOS：`.dmg`
- Windows：`.msi` 和 `.exe`
- Linux：`.AppImage` 和 `.deb`

普通使用请直接从 **GitHub Releases** 下载。

## 边界

Scripted Prompt 用于本地 prompt 管理和组合。

它不是：
- 云同步服务
- 在线 prompt 市场
- 多人协作编辑器

## 设计取舍

- 本地优先存储
- 用可复用 prompt 片段代替长 prompt 文档
- 用桌面安装包分发，而不是优先走网页使用

## 数据存储

运行时数据保存在用户本机。

常见路径：
- macOS：`~/Library/Application Support/scripted-prompt/`
- Linux：`~/.local/share/scripted-prompt/`
- Windows：`%APPDATA%\scripted-prompt\`

如果数据文件不存在，应用会自动初始化默认数据。

## 说明

- UI 改动后，正式构建前建议清理 Tauri 构建缓存。见 [DEPLOY.md](DEPLOY.md)。
- 正式版默认关闭 Tauri devtools。
- 默认应用数据在运行时生成，公开仓库不需要提交本地 `data/` 文件。

## 许可证

AGPL-3.0-or-later。见 [LICENSE](LICENSE)。
