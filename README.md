# Phira Render

高性能、轻量级的 Phira / Phigros 谱面离线视频渲染与练习预览工具。基于 Tauri + Vue 3 + Rust 构建。

---

## 主要特性

- **高性能渲染**：采用 Rust 与 FFmpeg 硬件加速（NVENC / QSV），音画合成极速高效。
- **1:1 导出预览**：自动获取物理显示器刷新率（如 144Hz / 240Hz），采用高质量 CRF 编码导出与游戏内完全一致的高帧率预览视频。
- **游戏内练习模式**：内置原生时间轴交互，支持点击字符就地编辑时间、左右方向键导航与毫秒级精准跳转。
- **高分屏点对点渲染**：原生支持 Windows Per-Monitor V2 High-DPI，界面在各类缩放下保持清晰锐利。
- **参数预设与路径记忆**：支持多套渲染配置预设保存、覆盖与删除，自定义导出路径自动记忆。
- **RPE 谱面互通**：支持一键绑定 RPE 游戏目录，自动扫描并直接发起渲染。

---

## 构建与运行

### 环境准备

1. **Node.js** (v18+)
2. **Rust** (1.70+)
3. **FFmpeg**（放置于系统 PATH 或应用所在目录）

### 一键构建 (Windows)

直接双击运行根目录下的脚本即可完成前端打包与 Release 二进制编译：

```cmd
build.bat
```

编译产物将输出至 `src-tauri/target/release/phira-render.exe`。

### 开发调试

```bash
# 启动前端开发服务器与 Tauri 调试窗口
npm run dev
cargo tauri dev
```

---

## 开源协议

本项目基于 Apache 2.0 / GPLv3 协议开源。
原版项目版权归 [TeamFlos/phira-render](https://github.com/TeamFlos/phira-render) 所有。
