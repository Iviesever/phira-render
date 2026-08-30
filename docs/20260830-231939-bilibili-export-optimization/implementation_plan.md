# B站视频导出优化技术蓝图

## 范围

在不改变渲染帧、时间轴、PBO读回与音频混音的前提下，优化默认预设和 FFmpeg 编码参数。保留旧版 7M 预设作为回退。

## 配置与预设

- 新增内置“B站高质量”预设并设为首次进入时的默认选择。
- 参数：1920×1080、60fps、硬件加速、4×MSAA、12M 视频码率。
- 保留“旧版 7M”内置预设。
- 用户自行保存的预设继续从后端加载，不改变存储格式。

## FFmpeg 参数

- rawvideo 输入继续使用配置中的尺寸、帧率和 RGBA。
- 删除 `-hwaccel_output_format cuda`；它属于解码路径，对 rawvideo 输入无效。
- 输出显式添加：
  - `-r <fps> -fps_mode cfr`
  - `-g <fps×2>`
  - `-profile:v high`
  - `-pix_fmt yuv420p`
  - `vflip,scale=in_range=pc:out_range=tv:out_color_matrix=bt709,format=yuv420p`
  - `-color_primaries bt709 -color_trc bt709 -colorspace bt709 -color_range tv`
  - `-movflags +faststart`
- NVENC 使用 `p5`；x264 保持 `medium`。
- 固定码率模式继续使用用户填写的 `-b:v`；码率为 `0` 时继续使用 CQ18/CRF18。

## 代码结构

- 从 `render.rs` 中抽离纯参数生成函数，输入编码器、尺寸、帧率、码率，输出 FFmpeg 参数列表。
- 渲染主流程只负责追加输入文件与输出路径并启动 FFmpeg。
- 参数函数单元测试覆盖 NVENC、x264、固定码率、质量模式和公共平台兼容参数。

## 错误处理

- 沿用现有码率非空校验与 FFmpeg 进程错误传播。
- 不新增自动降级、重试或隐藏式参数修正。

## 验证

1. 先写参数测试并确认当前实现失败。
2. `npm run build`。
3. `cargo test --release --manifest-path src-tauri/Cargo.toml`。
4. 实际导出短规格测试视频并检查：H.264 High、BT.709、CFR、目标帧数、2秒GOP、faststart。
5. 重新运行 DEINOS、Riven 开场帧回归和单帧截图回归。

## 明确不做

- 不尝试绕过 B站普通1080P的30fps限制。
- 不加入补帧、运动模糊、帧混合或音频重采样。
- 不改变现有任务协议或预设后端存储格式。
