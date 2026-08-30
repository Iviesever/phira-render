# Phira Render

高性能、轻量级的 Phira / Phigros 谱面离线视频渲染、单帧高清导出与交互式练习预览工具。  
基于 **Tauri + Vue 3 + Rust (Macroquad / Miniquad OpenGL) + FFmpeg** 构建。

---

## ✨ 核心特性

### 1. 🎛️ 独立附着式实时预览控制台
- **16:9 纯净无遮挡呈现**：将练习控制区与时间轴完全移至右侧独立的 Win32 附着窗口，谱面主画面保持纯净零遮挡。
- **窗口生命周期深度联动**：控制台窗口与谱面渲染窗口建立原生父子/从属绑定，支持窗口移动、最小化、还原、关闭及 `Alt+Tab` 切屏全流程无缝跟随。
- **高分屏点对点适配**：原生支持 Windows Per-Monitor V2 High-DPI 缩放，跨不同缩放比例多显示器自适应，按键与文字保持点对点清晰锐利。
- **一体化交互时间轴**：
  - 支持直接点击或拖拽绿色播放游标，实现毫秒级即时跳转。
  - 支持拖拽 **A / B 标记点** 自由配置循环练习选区，区间高亮清晰可见。
  - **就地时间戳编辑**：点击当前时间戳可直接就地输入 `mm:ss.ms` 毫秒级时间进行精准定位。
- **变速播放与循环练习**：
  - 支持 `0.5x` 至 `2.0x` 无级滑块与常用档位切换。
  - 支持一键重播循环区间（`R` 键），播放到达 B 点时自动无缝暂停或重新循环。
- **实时单帧图像捕获与导出**：
  - **复制当前帧图片**（快捷键 <kbd>C</kbd>）：一键捕获当前画面的最高清无损渲染帧，直接复制到系统剪贴板（支持在微信、QQ、Discord、图像编辑软件中直接 <kbd>Ctrl</kbd>+<kbd>V</kbd> 粘贴）。
  - **导出当前帧图片**（快捷键 <kbd>S</kbd>）：弹出系统文件保存对话框，将当前画面导出为无损 PNG 图像文件。

### 2. 🎬 高性能离线视频渲染
- **硬件加速与高帧率导出**：采用 Rust 与 FFmpeg 硬件加速（支持 NVIDIA NVENC、Intel QSV、AMD AMF 以及 CPU 编码），导出极速平滑。
- **显示器刷新率自动同步**：自动检测当前物理显示器刷新率（如 144Hz / 240Hz / 360Hz），支持 1:1 导出与游戏内手感一致的高帧率预览视频。
- **完整谱面后处理着色器支持**：深度适配 `extra.json` 特效着色器（如 Arcaea 风格 `arcgros-guide-height.glsl` 引导线高度垂直重构着色器、色差、模糊等），多重采样 MSAA 双缓冲 Ping-Pong 渲染管线，确保特效完美还原。
- **参数预设与路径记忆**：支持多套渲染配置预设保存、覆盖与删除，自定义导出路径与渲染参数自动记忆。
- **RPE 谱面互通**：支持一键绑定 RPE 游戏目录，自动扫描并直接发起渲染。

### 3. ⚡ CLI 命令行单帧渲染模式 (`frame`)
无需启动图形化界面，即可通过命令行极速导出任意谱面在指定时间点的 1:1 游戏画面高清截图：
- **确定性静默快进打击模拟**：准确模拟判定引擎打击状态，精确保留指定时间点的 Combo 连击数、Score 分数、判定线状态与谱面动画。
- **完整着色器效果应用**：自动加载谱面自定义 GLSL 着色器与材质配置，输出真实的游戏最终画面。
- **自定义分辨率输出**：支持缺省 1080P 或自定义任意分辨率（如 2K / 4K）渲染输出。

---

## ⌨️ 快捷键指南

| 快捷键 | 功能说明 |
| :--- | :--- |
| <kbd>Space</kbd> | 暂停 / 继续播放 |
| <kbd>R</kbd> | 重播当前循环选区（若未设置则从起点开始播放） |
| <kbd>C</kbd> | **复制当前帧图片** 到系统剪贴板（可直接粘贴分享） |
| <kbd>S</kbd> | **导出当前帧图片** 为 PNG 文件（打开保存对话框） |
| <kbd>←</kbd> / <kbd>→</kbd> | 快退 / 快进 1 秒 |
| <kbd>Esc</kbd> | 退出预览模式并返回主界面 |

---

## 💻 CLI 命令行使用指南

### 单帧截图导出 (`frame`)

```bash
# 基本语法
phira-render frame <谱面路径.zip> <目标时间秒数> <输出路径.png> [分辨率宽] [分辨率高]

# 示例 1：以默认 1080P 渲染指定谱面第 35.0 秒的高清画面
phira-render.exe frame "D:\charts\CataclysmCry.zip" 35.0 "D:\output\frame_35s.png"

# 示例 2：以 2K 分辨率 (2560x1440) 渲染指定谱面第 10.5 秒的高清画面
phira-render.exe frame "D:\charts\CataclysmCry.zip" 10.5 "D:\output\frame_2k.png" 2560 1440
```

---

## 🛠️ 构建与开发

### 环境要求

1. **Node.js** (v18+) & **pnpm / npm**
2. **Rust** (1.70+ 稳定版) & **Cargo**
3. **FFmpeg**（放置于系统 `PATH` 环境变量中，或放置于应用同级目录）

### 一键构建打包 (Windows)

项目根目录提供了完整自动化构建脚本，双击或在终端运行：

```cmd
build.bat
```

脚本将自动执行：
1. 前端 Vue 3 TypeScript 类型检查与 Vite 生产打包
2. Rust 后端 Release 优化编译与打包
3. 输出最终可执行文件至 `src-tauri/target/release/phira-render.exe`

### 本地开发调试

```bash
# 启动前端热重载与 Tauri 开发调试窗口
npm run dev
cargo tauri dev
```

---

## 🏗️ 技术架构

- **前端界面**：Vue 3 + TypeScript + Vite + Vuetify 3
- **应用宿主**：Tauri v1 + Win32 API 原生窗口与 DPI 管理
- **渲染核心**：Rust + Macroquad + Miniquad OpenGL ES 2.0 / GLSL 着色器管线 + Prpr 判定引擎
- **多媒体处理**：FFmpeg (NVENC / QSV / AMF / CPU) 硬件加速编解码 + Symphonia 音频处理

---

## 📄 开源协议

本项目基于 Apache 2.0 / GPLv3 协议开源。  
原版项目版权归 [TeamFlos/phira-render](https://github.com/TeamFlos/phira-render) 所有。
