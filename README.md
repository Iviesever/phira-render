# phira-render (Iviesever的增强分支)

> **注:** 这是一个基于 TeamFlos 开发的原版 [phira-render](https://github.com/TeamFlos/phira-render) 的修改版。所有原始版权归原作者所有。本分支包含了一些专为高级使用场景量身定制的工作流和渲染行为改进。

一个帮助您渲染 Phigros 谱面的实用工具。基于 Tauri + Vuetify 构建。

## Iviesever 的改进与区别

- **1:1 原生导出预览**: 新增“导出预览”功能，确保渲染出的视频与实时预览完全一致。它通过自动获取物理显示器的原生刷新率（例如 240Hz，且能精准绕过远程桌面虚拟适配器的干扰）并采用高质量的 CRF 编码 (`-crf 18`) 来实现所见即所得。
- **自定义导出路径**: 您现在可以在参数配置界面直接定义自定义导出文件夹或具体的文件路径。
- **支持快速粘贴路径**: 完美支持直接从 Windows 资源管理器粘贴带引号的路径 (`Ctrl+Shift+C`)，程序会自动去除两端的双引号，让操作更加顺畅。
- **预设记忆功能**: 自定义的导出路径已完全集成到预设配置系统中，重启应用后依然能够记忆并恢复您的配置。
- **默认开启硬件加速**: 默认开启硬件加速选项，优先利用 NVENC/QSV 大幅提升渲染效率。

## 原始特性

- 现代美观的 UI 界面；
- 渲染速度极快（与 prpr-render 相比提升了 3-4 倍）；
- 只需几次点击即可绑定 RPE；
- 任务列表管理；
- 添加和管理多个参数预设；
- ...

## 截图展示

![Render Options](arts/render-options.png)

![Task List](arts/task-list.png)

## 安装方法

请查看 [Releases](https://github.com/TeamFlos/phira-render/releases) 页面。

## 手动构建

开发环境

```bash
cargo tauri dev
```

构建

```bash
cargo tauri build
```
