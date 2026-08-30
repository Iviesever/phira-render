// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr_l10n::tl_file!("render" tl);

use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{Config, Mods},
    core::{internal_id, MSRenderTarget, NoteKind},
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, GameMode, GameScene, LoadingScene, NextScene, Scene},
    time::TimeManager,
    ui::{FontArc, TextPainter, Ui},
    Main,
};
use sasa::AudioClip;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use std::ffi::OsStr;
use tempfile::NamedTempFile;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderConfig {
    resolution: (u32, u32),
    ending_length: f64,
    fps: u32,
    hardware_accel: bool,
    bitrate: String,

    aggressive: bool,
    disable_effect: bool,
    double_hint: bool,
    fxaa: bool,
    note_scale: f32,
    particle: bool,
    player_avatar: Option<String>,
    player_name: String,
    player_rks: f32,
    sample_count: u32,
    res_pack_path: Option<String>,
    speed: f32,
    volume_music: f32,
    volume_sfx: f32,
    #[serde(default)]
    show_player: bool,
    #[serde(default)]
    export_preview: bool,
    #[serde(default)]
    pub export_path: Option<String>,
}

impl RenderConfig {
    pub fn to_config(&self) -> Config {
        let mut config = Config::default();
        config.aggressive = self.aggressive;
        config.disable_effect = self.disable_effect;
        config.double_hint = self.double_hint;
        config.fxaa = self.fxaa;
        config.note_scale = self.note_scale;
        config.particle = self.particle;
        config.player_name = self.player_name.clone();
        config.player_rks = self.player_rks;
        config.sample_count = self.sample_count;
        config.res_pack_path = self.res_pack_path.clone();
        config.speed = self.speed;
        config.volume_music = self.volume_music;
        config.volume_sfx = self.volume_sfx;
        config
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParams {
    pub path: PathBuf,
    pub info: ChartInfo,
    pub config: RenderConfig,
}

#[derive(Serialize, Deserialize)]
pub enum IPCEvent {
    StartMixing,
    StartRender(u64),
    Frame,
    Done(f64),
}

pub async fn build_player(config: &RenderConfig) -> Result<BasicPlayer> {
    Ok(BasicPlayer {
        avatar: if let Some(path) = &config.player_avatar {
            Some(
                Texture2D::from_file_with_format(
                    &tokio::fs::read(path)
                        .await
                        .with_context(|| tl!("load-avatar-failed"))?,
                    None,
                )
                .into(),
            )
        } else {
            None
        },
        id: 0,
        rks: config.player_rks,
        historic_best: 0,
    })
}

fn cmd_hidden(program: impl AsRef<OsStr>) -> Command {
    let cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = cmd;
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    cmd
}

fn video_input_args(width: u32, height: u32, fps: u32) -> Vec<String> {
    [
        "-y".to_owned(),
        "-f".to_owned(),
        "rawvideo".to_owned(),
        "-c:v".to_owned(),
        "rawvideo".to_owned(),
        "-s".to_owned(),
        format!("{width}x{height}"),
        "-r".to_owned(),
        fps.to_string(),
        "-pix_fmt".to_owned(),
        "rgba".to_owned(),
        "-i".to_owned(),
        "-".to_owned(),
        "-i".to_owned(),
    ]
    .into()
}

fn video_output_args(codec: &str, bitrate: &str, fps: u32) -> Vec<String> {
    let mut args = vec![
        "-c:a".to_owned(),
        "aac".to_owned(),
        "-b:a".to_owned(),
        "320k".to_owned(),
        "-c:v".to_owned(),
        codec.to_owned(),
    ];
    match (codec, bitrate) {
        ("h264_nvenc", "0") => args.extend(["-cq", "18", "-preset", "p5"].map(str::to_owned)),
        ("h264_qsv", "0") => args.extend(["-global_quality", "18", "-preset", "medium"].map(str::to_owned)),
        (_, "0") => args.extend(["-crf", "18", "-preset", "medium"].map(str::to_owned)),
        ("h264_nvenc", bitrate) => args.extend(["-b:v", bitrate, "-preset", "p5"].map(str::to_owned)),
        (_, bitrate) => args.extend(["-b:v", bitrate, "-preset", "medium"].map(str::to_owned)),
    }
    args.extend(
        [
            "-profile:v".to_owned(),
            "high".to_owned(),
            "-pix_fmt".to_owned(),
            "yuv420p".to_owned(),
            "-r".to_owned(),
            fps.to_string(),
            "-fps_mode".to_owned(),
            "cfr".to_owned(),
            "-g".to_owned(),
            (fps * 2).to_string(),
            "-color_primaries".to_owned(),
            "bt709".to_owned(),
            "-color_trc".to_owned(),
            "bt709".to_owned(),
            "-colorspace".to_owned(),
            "bt709".to_owned(),
            "-color_range".to_owned(),
            "tv".to_owned(),
            "-map".to_owned(),
            "0:v:0".to_owned(),
            "-map".to_owned(),
            "1:a:0".to_owned(),
            "-vf".to_owned(),
            "vflip,scale=in_range=pc:out_range=tv:out_color_matrix=bt709,format=yuv420p".to_owned(),
            "-movflags".to_owned(),
            "+faststart".to_owned(),
            "-f".to_owned(),
            "mp4".to_owned(),
        ]
        .into_iter(),
    );
    args
}

#[cfg(test)]
mod tests {
    use super::{video_input_args, video_output_args};

    fn contains(args: &[String], expected: &[&str]) -> bool {
        args.windows(expected.len())
            .any(|window| window.iter().map(String::as_str).eq(expected.iter().copied()))
    }

    #[test]
    fn rawvideo_input_is_cfr_and_does_not_request_cuda_decoding() {
        let args = video_input_args(1920, 1080, 60);
        assert!(contains(&args, &["-s", "1920x1080", "-r", "60"]));
        assert!(!args.iter().any(|arg| arg == "-hwaccel_output_format"));
    }

    #[test]
    fn nvenc_fixed_bitrate_uses_bilibili_compatible_output() {
        let args = video_output_args("h264_nvenc", "12M", 60);
        assert!(contains(&args, &["-b:v", "12M", "-preset", "p5"]));
        assert!(contains(&args, &["-profile:v", "high", "-pix_fmt", "yuv420p"]));
        assert!(contains(&args, &["-r", "60", "-fps_mode", "cfr", "-g", "120"]));
        assert!(contains(&args, &["-color_primaries", "bt709", "-color_trc", "bt709"]));
        assert!(contains(&args, &["-colorspace", "bt709", "-color_range", "tv"]));
        assert!(contains(
            &args,
            &["-vf", "vflip,scale=in_range=pc:out_range=tv:out_color_matrix=bt709,format=yuv420p"]
        ));
        assert!(contains(&args, &["-movflags", "+faststart"]));
    }

    #[test]
    fn quality_modes_match_each_encoder() {
        assert!(contains(&video_output_args("h264_nvenc", "0", 60), &["-cq", "18", "-preset", "p5"]));
        assert!(contains(
            &video_output_args("h264_qsv", "0", 60),
            &["-global_quality", "18", "-preset", "medium"]
        ));
        assert!(contains(&video_output_args("libx264", "0", 60), &["-crf", "18", "-preset", "medium"]));
    }
}

pub fn find_ffmpeg() -> Result<Option<String>> {
    fn test(path: impl AsRef<OsStr>) -> bool {
        matches!(cmd_hidden(path).arg("-version").output(), Ok(_))
    }
    
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let ffmpeg_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let local_ffmpeg = exe_dir.join(ffmpeg_name);
    
    // Prioritize local ffmpeg over global PATH ffmpeg
    if test(&local_ffmpeg) {
        return Ok(Some(local_ffmpeg.display().to_string()));
    }
    
    if test("ffmpeg") {
        return Ok(Some("ffmpeg".to_owned()));
    }
    
    eprintln!("Failed to find local and global ffmpeg.");
    Ok(None)
}

pub async fn main() -> Result<()> {
    use crate::ipc::client::*;

    set_pc_assets_folder(&std::env::args().nth(2).unwrap());

    let mut stdin = std::io::stdin().lock();
    let stdin = &mut stdin;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let params: RenderParams = serde_json::from_str(line.trim())?;
    let path = params.path;

    line.clear();
    stdin.read_line(&mut line)?;
    let output_path: PathBuf = serde_json::from_str(line.trim())?;

    let mut fs = fs::fs_from_file(&path)?;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;

    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };
    dbg!(&ffmpeg);

    let mut painter = TextPainter::new(font, None);

    let mut config = params.config.to_config();
    config.mods = Mods::AUTOPLAY;

    let info = params.info;

    let (chart, ..) = GameScene::load_chart(fs.deref_mut(), &info)
        .await
        .with_context(|| tl!("load-chart-failed"))?;
    let mut res_pack_fs = if let Some(path) = &params.config.res_pack_path {
        prpr::fs::fs_from_file(path.as_ref()).ok()
    } else {
        None
    };

    macro_rules! ld {
            ($path:literal) => {
                async {
                    let bytes = if let Some(fs) = &mut res_pack_fs {
                        if let Ok(b) = fs.load_file($path).await {
                            b
                        } else {
                            load_file($path).await.unwrap()
                        }
                    } else {
                        load_file($path).await.unwrap()
                    };
                    AudioClip::new(bytes).with_context(|| tl!("load-sfx-failed", "name" => $path))
                }.await?
            };
        }
    let music: Result<_> = async { AudioClip::new(fs.load_file(&info.music).await?) }.await;
    let music = music.with_context(|| tl!("load-music-failed"))?;
    let ending = ld!("ending.ogg");
    let track_length = music.length() as f64;
    let sfx_click = ld!("click.ogg");
    let sfx_drag = ld!("drag.ogg");
    let sfx_flick = ld!("flick.ogg");

    let mut gl = unsafe { get_internal_gl() };

    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);

    let fps = params.config.fps;
    let frame_delta = 1. / fps as f32;

    // Loading scene: BEFORE_TIME(1.0) + transition_time(1.4) + wait_time(0.4) = 2.8
    // +1 frame for the strict '>' comparison in loading.rs next_scene
    // GameScene BEFORE_TIME: 0.7
    let O: f64 = 2.8 + frame_delta as f64 + 0.7;
    const A: f64 = 0.7 + 0.3 + 0.4;

    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = if params.config.show_player {
        O + length + A + params.config.ending_length
    } else {
        // End exactly when the game scene fades to black.
        // start_time = 2.8 + frame_delta
        // game time for fade to black = track_length + 0.5 (WAIT_TIME) + 0.7 (AFTER_TIME) = track_length + 1.2
        4.0 + frame_delta as f64 + track_length - chart.offset.min(0.) as f64
    };
    let offset = chart.offset.max(0.);

    let render_start_time = Instant::now();

    send(IPCEvent::StartMixing);
    let mixing_output = NamedTempFile::new()?;
    let sample_rate = 44100;
    assert_eq!(sample_rate, ending.sample_rate());
    assert_eq!(sample_rate, sfx_click.sample_rate());
    assert_eq!(sample_rate, sfx_drag.sample_rate());
    assert_eq!(sample_rate, sfx_flick.sample_rate());
    let mut output = vec![0.0_f32; (video_length * sample_rate as f64).ceil() as usize * 2];
    {
        let pos = O - chart.offset.min(0.) as f64;
        let count = (music.length() as f64 * sample_rate as f64) as usize;
        let mut it = output[((pos * sample_rate as f64).round() as usize * 2)..].iter_mut();
        let ratio = 1. / sample_rate as f64;
        for frame in 0..count {
            let position = frame as f64 * ratio;
            let frame = music.sample(position as f64).unwrap_or_default();
            *it.next().unwrap() += frame.0 * volume_music;
            *it.next().unwrap() += frame.1 * volume_music;
        }
    }
    let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
        let position = (pos * sample_rate as f64).round() as usize * 2;
        if position >= output.len() {
            return 0;
        }
        let slice = &mut output[position..];
        let len = (slice.len() / 2).min(clip.frame_count());
        let mut it = slice.iter_mut();
        // TODO optimize?
        for frame in clip.frames()[..len].iter() {
            let dst = it.next().unwrap();
            *dst += frame.0 * volume;
            let dst = it.next().unwrap();
            *dst += frame.1 * volume;
        }
        return len;
    };
    for note in chart
        .lines
        .iter()
        .flat_map(|it| it.notes.iter())
        .filter(|it| !it.fake)
    {
        place(
            O + note.time as f64 + offset as f64,
            match note.kind {
                NoteKind::Click | NoteKind::Hold { .. } => &sfx_click,
                NoteKind::Drag => &sfx_drag,
                NoteKind::Flick => &sfx_flick,
            },
            volume_sfx,
        );
    }
    if params.config.show_player {
        let mut pos = O + length + A;
        while place(pos, &ending, volume_music) != 0 {
            pos += ending.frame_count() as f64 / sample_rate as f64;
        }
    }
    let mut proc = cmd_hidden(&ffmpeg)
        .args("-y -f f32le -ar 44100 -ac 2 -i - -f wav".split_whitespace())
        .arg(mixing_output.path())
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let input = proc.stdin.as_mut().unwrap();
    let mut writer = BufWriter::new(input);
    for sample in output.into_iter() {
        writer.write_all(&sample.to_le_bytes())?;
    }
    drop(writer);
    proc.wait()?;

    let (vw, vh) = params.config.resolution;
    let mst = Rc::new(MSRenderTarget::new((vw, vh), config.sample_count));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    static MSAA: AtomicBool = AtomicBool::new(false);
    let player = if params.config.show_player {
        Some(build_player(&params.config).await?)
    } else {
        None
    };
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(GameMode::Normal, info, config, fs, player, None, None, None, None).await?,
        ),
        tm,
        {
            let mut cnt = 0;
            let mst = Rc::clone(&mst);
            move || {
                cnt += 1;
                if cnt == 1 || cnt == 3 {
                    MSAA.store(true, Ordering::SeqCst);
                    Some(mst.input())
                } else {
                    MSAA.store(false, Ordering::SeqCst);
                    Some(mst.output())
                }
            }
        },
    )
    .await?;
    main.top_level = false;
    main.viewport = Some((0, 0, vw as _, vh as _));


    let codecs = String::from_utf8(
        cmd_hidden(&ffmpeg)
            .arg("-codecs")
            .output()
            .with_context(|| tl!("run-ffmpeg-failed"))?
            .stdout,
    )?;
    let use_cuda = params.config.hardware_accel && codecs.contains("h264_nvenc");
    let has_qsv = params.config.hardware_accel && codecs.contains("h264_qsv");

    let codec = if use_cuda {
        "h264_nvenc"
    } else if has_qsv {
        "h264_qsv"
    } else if params.config.hardware_accel {
        bail!(tl!("no-hwacc"));
    } else {
        "libx264"
    };
    let input_args = video_input_args(vw, vh, fps);
    let output_args = video_output_args(codec, &params.config.bitrate, fps);

    let mut proc = cmd_hidden(&ffmpeg)
        .args(input_args)
        .arg(mixing_output.path())
        .args(output_args)
        .arg(output_path)
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let mut input = proc.stdin.take().unwrap();

    let byte_size = vw as usize * vh as usize * 4;

    let frames = (video_length * fps as f64).ceil() as u64;

    const N: usize = 3;
    let pipeline_depth = (N - 1).min(frames as usize);
    let mut pbos: [GLuint; N] = [0; N];
    unsafe {
        use miniquad::gl::*;
        glGenBuffers(N as _, pbos.as_mut_ptr());
        for pbo in pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo);
            glBufferData(
                GL_PIXEL_PACK_BUFFER,
                (vw as u64 * vh as u64 * 4) as _,
                std::ptr::null(),
                GL_STREAM_READ,
            );
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    }

    send(IPCEvent::StartRender(frames));

    for frame in 0..frames {
        // Fix #3: use f64 arithmetic to eliminate f32 quantization jitter
        *my_time.borrow_mut() = frame as f64 / fps as f64;
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        clear_background(BLACK);
        main.viewport = Some((0, 0, vw as _, vh as _));
        main.update()?;
        main.render(&mut painter)?;
        // TODO magic. can't remove this line.
        draw_rectangle(0., 0., 0., 0., Color::default());
        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }
        unsafe {
            use miniquad::gl::*;
            let tex = mst.output().texture.raw_miniquad_texture_handle();
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));

            // Async read current frame into its PBO slot
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame as usize % N]);
            glReadPixels(
                0,
                0,
                tex.width as _,
                tex.height as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );

            // Fix #1: Only read from PBO after pipeline is primed (N-1 frames rendered)
            // This eliminates the 2 garbage frames at the start
            if frame >= pipeline_depth as u64 {
                let read_idx = (frame as usize - pipeline_depth) % N;
                glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[read_idx]);
                let mut src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
                // Fix #2: If MapBuffer fails, force GPU sync and retry instead of silently dropping
                if src.is_null() {
                    glFinish();
                    src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
                }
                if !src.is_null() {
                    input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
                    glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
                }
            }
        }
        send(IPCEvent::Frame);
    }

    // Fix #1 (cont.): Flush remaining frames from the PBO pipeline
    // Without this, the last N-1 frames would be lost
    unsafe {
        use miniquad::gl::*;
        for i in 0..pipeline_depth {
            let read_idx = (frames as usize - pipeline_depth + i) % N;
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[read_idx]);
            let mut src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            if src.is_null() {
                glFinish();
                src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            }
            if !src.is_null() {
                input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
            }
        }
    }
    drop(input);
    proc.wait()?;

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}
