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

struct BaseScene(Option<NextScene>, bool);
impl Scene for BaseScene {
    fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        show_error(
            result
                .downcast::<anyhow::Error>()
                .unwrap()
                .context("加载谱面失败"),
        );
        self.1 = true;
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
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut buf = String::new();
        while let Ok(n) = handle.read_line(&mut buf) {
            if n == 0 {
                break;
            }
            let trimmed = buf.trim();
            if !trimmed.is_empty() {
                if let Ok(action) = serde_json::from_str::<PreviewAction>(trimmed) {
                    let is_exit = matches!(action, PreviewAction::Exit);
                    let _ = action_tx.send(action);
                    if is_exit {
                        break;
                    }
                }
            }
            buf.clear();
        }
    });

    // Spawn background thread to output status to stdout
    std::thread::spawn(move || {
        let mut last_report_time = std::time::Instant::now();
        let mut last_status: Option<PreviewStatusReport> = None;
        let mut stdout = std::io::stdout().lock();
        while let Ok(status) = status_rx.recv() {
            let should_send = last_status.as_ref().map_or(true, |last| {
                last.paused != status.paused
                    || (last.speed - status.speed).abs() > 0.001
                    || (last.start - status.start).abs() > 0.01
                    || (last.end - status.end).abs() > 0.01
                    || last_report_time.elapsed().as_millis() >= 50
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
