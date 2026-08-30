use crate::render::{build_player, RenderParams};
use anyhow::Result;
use macroquad::prelude::*;
use prpr::{
    config::{Config, Mods},
    fs,
    scene::{
        set_preview_channels, show_error, GameMode, LoadingScene, NextScene, PreviewAction,
        PreviewStatusReport, Scene,
    },
    time::TimeManager,
    ui::{FontArc, TextPainter, Ui},
    Main,
};
use std::io::{BufRead, Write};
use std::sync::mpsc::channel;

struct BaseScene(Option<NextScene>, bool, bool);
impl Scene for BaseScene {
    fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        show_error(
            result
                .downcast::<anyhow::Error>()
                .unwrap()
                .context("加载谱面失败"),
        );
        self.1 = true;
        if self.2 {
            self.0 = Some(NextScene::Exit);
        }
        Ok(())
    }
    fn enter(&mut self, _tm: &mut TimeManager, _target: Option<RenderTarget>) -> Result<()> {
        if self.0.is_none() && !self.1 {
            self.0 = Some(NextScene::Exit);
        }
        Ok(())
    }
    fn update(&mut self, _tm: &mut TimeManager) -> Result<()> {
        Ok(())
    }
    fn render(&mut self, _tm: &mut TimeManager, _ui: &mut Ui) -> Result<()> {
        Ok(())
    }
    fn next_scene(&mut self, _tm: &mut TimeManager) -> prpr::scene::NextScene {
        self.0.take().unwrap_or_default()
    }
}

pub async fn main() -> Result<()> {
    set_pc_assets_folder(&std::env::args().nth(2).unwrap());

    let (action_tx, action_rx) = channel::<PreviewAction>();
    let (status_tx, status_rx) = channel::<PreviewStatusReport>();
    set_preview_channels(action_rx, status_tx);

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let params: RenderParams = serde_json::from_str(line.trim())?;

    // Spawn background thread to read commands from stdin
    std::thread::spawn(move || {
        crate::ipc::log("[PREVIEW CHILD STDIN READER] thread started");
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut buf = String::new();
        while let Ok(n) = handle.read_line(&mut buf) {
            if n == 0 {
                crate::ipc::log("[PREVIEW CHILD STDIN READER] EOF on stdin");
                break;
            }
            let trimmed = buf.trim();
            if !trimmed.is_empty() {
                crate::ipc::log(&format!("[PREVIEW CHILD STDIN READER] got line: {}", trimmed));
                match serde_json::from_str::<PreviewAction>(trimmed) {
                    Ok(action) => {
                        let is_exit = matches!(action, PreviewAction::Exit);
                        let _ = action_tx.send(action);
                        if is_exit {
                            break;
                        }
                    }
                    Err(e) => {
                        crate::ipc::log(&format!("[PREVIEW CHILD STDIN READER] parse error: {:?}", e));
                    }
                }
            }
            buf.clear();
        }
        crate::ipc::log("[PREVIEW CHILD STDIN READER] thread exit");
    });

    // Spawn background thread to output status to stdout
    std::thread::spawn(move || {
        let mut last_report_time = std::time::Instant::now();
        let mut last_status: Option<PreviewStatusReport> = None;
        let mut stdout = std::io::stdout().lock();
        while let Ok(mut status) = status_rx.recv() {
            while let Ok(newer) = status_rx.try_recv() {
                status = newer;
            }
            let should_send = last_status.as_ref().map_or(true, |last| {
                last.paused != status.paused
                    || (last.speed - status.speed).abs() > 0.001
                    || (last.start - status.start).abs() > 0.01
                    || (last.end - status.end).abs() > 0.01
                    || last_report_time.elapsed().as_millis() >= 35
            });
            if should_send {
                last_report_time = std::time::Instant::now();
                if let Ok(json) = serde_json::to_string(&status) {
                    let _ = writeln!(stdout, "STATUS:{}", json);
                    let _ = stdout.flush();
                }
                last_status = Some(status);
            }
        }
    });

    let fs = fs::fs_from_file(&params.path)?;
    let info = params.info;
    let mut config: Config = params.config.to_config();
    config.mods |= Mods::AUTOPLAY;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font, None);

    let player = build_player(&params.config).await?;

    let tm = TimeManager::default();
    let ctm = TimeManager::from_config(&config);
    let mut main = Main::new(
        Box::new(BaseScene(
            Some(NextScene::Overlay(Box::new(
                LoadingScene::new(
                    GameMode::Exercise,
                    info,
                    config,
                    fs,
                    Some(player),
                    None,
                    Some(Box::new(|_, res, _| {
                        res.config.mods.insert(Mods::AUTOPLAY);
                    })),
                    None,
                    None,
                )
                .await?,
            ))),
            false,
            false,
        )),
        ctm,
        None,
    )
    .await?;

    'app: loop {
        main.update()?;
        main.render(&mut painter)?;
        if main.should_exit() {
            break 'app;
        }

        next_frame().await;
    }

    Ok(())
}

pub async fn frame_cli() -> Result<()> {
    use anyhow::Context;
    use std::ops::DerefMut;

    crate::ipc::log("[FRAME_CLI] entered");
    let args: Vec<String> = std::env::args().collect();
    crate::ipc::log(&format!("[FRAME_CLI] args: {:?}", args));
    if args.len() < 5 {
        crate::ipc::log("[FRAME_CLI] insufficient args");
        eprintln!("Usage: phira-render frame <chart_path> <time_in_seconds> <output_png_path> [width] [height]");
        std::process::exit(1);
    }

    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let asset_path = if exe_dir.join("assets").exists() {
        exe_dir.join("assets")
    } else if std::path::Path::new("src-tauri/assets").exists() {
        std::path::PathBuf::from("src-tauri/assets")
    } else {
        exe_dir.clone()
    };
    crate::ipc::log(&format!("[FRAME_CLI] asset_path: {:?}", asset_path));
    set_pc_assets_folder(&asset_path.display().to_string());

    let chart_path = std::path::PathBuf::from(&args[2]);
    let target_time: f64 = args[3]
        .parse()
        .with_context(|| format!("Invalid frame time: {}", args[3]))?;
    if !target_time.is_finite() {
        anyhow::bail!("Frame time must be finite");
    }
    let output_path = std::path::PathBuf::from(&args[4]);
    crate::ipc::log(&format!("[FRAME_CLI] chart_path: {:?}, time: {}, output: {:?}", chart_path, target_time, output_path));

    let mut fs_box = fs::fs_from_file(&chart_path)
        .with_context(|| format!("Failed to load chart from {:?}", chart_path))?;
    let info = fs::load_info(fs_box.deref_mut())
        .await
        .with_context(|| "Failed to load chart info")?;
    let mut config = Config::default();
    config.mods |= Mods::AUTOPLAY;
    config.sample_count = 1;

    let (action_tx, action_rx) = channel::<PreviewAction>();
    let (status_tx, status_rx) = channel::<PreviewStatusReport>();
    set_preview_channels(action_rx, status_tx);

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font, None);

    let ctm = TimeManager::from_config(&config);
    let mut main = Main::new(
        Box::new(BaseScene(
            Some(NextScene::Overlay(Box::new(
                LoadingScene::new(
                    GameMode::Exercise,
                    info,
                    config,
                    fs_box,
                    None,
                    None,
                    Some(Box::new(|_, res, _| {
                        res.config.mods.insert(Mods::AUTOPLAY);
                    })),
                    None,
                    None,
                )
                .await?,
            ))),
            false,
            true,
        )),
        ctm,
        None,
    )
    .await?;
    crate::ipc::log("[FRAME_CLI] Main created with LoadingScene, starting loop");

    let mut seeked = false;
    let mut captured = false;
    let mut capture_frame = 0;

    'app: loop {
        main.update()?;
        if main.should_exit() {
            anyhow::bail!("Failed to load chart");
        }
        main.render(&mut painter)?;

        while let Ok(status) = status_rx.try_recv() {
            if !seeked {
                crate::ipc::log(&format!("[FRAME_CLI] seeking to {}", target_time));
                let _ = action_tx.send(PreviewAction::Pause);
                let _ = action_tx.send(PreviewAction::Seek { time: target_time });
                seeked = true;
            } else if !captured && (status.time - target_time).abs() < 0.2 {
                capture_frame += 1;
                if capture_frame >= 2 {
                    crate::ipc::log(&format!("[FRAME_CLI] sending capture to {:?}", output_path));
                    let _ = action_tx.send(PreviewAction::CaptureFrame {
                        save_path: Some(output_path.to_string_lossy().to_string()),
                        to_clipboard: false,
                        clipboard_filename: None,
                    });
                    captured = true;
                }
            } else if captured {
                capture_frame += 1;
                if capture_frame >= 4 {
                    crate::ipc::log("[FRAME_CLI] capture complete, breaking");
                    break 'app;
                }
            }
        }

        next_frame().await;
    }

    crate::ipc::log(&format!("[FRAME_CLI] done! saved to {:?}", output_path));
    println!(
        "{{\"status\":\"ok\",\"time\":{},\"output\":{:?}}}",
        target_time,
        output_path.display().to_string()
    );

    Ok(())
}
