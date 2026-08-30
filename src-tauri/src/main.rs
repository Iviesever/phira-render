// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

prpr_l10n::tl_file!("main" mtl);

mod common;
mod ipc;
mod preview;
mod render;
mod task;

use anyhow::{bail, Context, Result};
use common::{ensure_dir, respack_dir, CONFIG_DIR, DATA_DIR};
use fs4::tokio::AsyncFileExt;
use macroquad::prelude::set_pc_assets_folder;
use prpr::{
    fs::{self, FileSystem},
    info::ChartInfo,
};
use render::{find_ffmpeg, RenderConfig, RenderParams};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    future::Future,
    io::{BufRead, BufReader, BufWriter},
    ops::DerefMut,
    path::{Path, PathBuf},
    process::Stdio,
    sync::OnceLock,
    time::SystemTime,
};
use task::{TaskQueue, TaskView};
use tauri::{
    CustomMenuItem, InvokeError, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, WindowEvent,
};
use tokio::{io::AsyncWriteExt, process::Command};

static ASSET_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOCK_FILE: OnceLock<tokio::fs::File> = OnceLock::new();

#[cfg(target_os = "windows")]
fn enable_hidpi() {
    use std::ffi::c_void;
    extern "system" {
        fn LoadLibraryA(lpLibFileName: *const u8) -> *mut c_void;
        fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
    }
    unsafe {
        let user32 = LoadLibraryA(b"user32.dll\0".as_ptr());
        if !user32.is_null() {
            let func_ptr = GetProcAddress(user32, b"SetProcessDpiAwarenessContext\0".as_ptr());
            if !func_ptr.is_null() {
                let set_dpi: unsafe extern "system" fn(isize) -> i32 = std::mem::transmute(func_ptr);
                // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
                let _ = set_dpi(-4);
                return;
            }
        }
        let shcore = LoadLibraryA(b"shcore.dll\0".as_ptr());
        if !shcore.is_null() {
            let func_ptr = GetProcAddress(shcore, b"SetProcessDpiAwareness\0".as_ptr());
            if !func_ptr.is_null() {
                let set_dpi: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(func_ptr);
                // PROCESS_PER_MONITOR_DPI_AWARE = 2
                let _ = set_dpi(2);
            }
        }
    }
}

#[inline]
async fn wrap_async<R>(f: impl Future<Output = Result<R>>) -> Result<R, InvokeError> {
    f.await.map_err(|e| {
        eprintln!("{e:?}");
        InvokeError::from_anyhow(e)
    })
}

pub fn build_conf() -> macroquad::window::Conf {
    #[cfg(target_os = "windows")]
    enable_hidpi();

    let mut width = 1280;
    let mut height = 720;

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 5 && args.get(1).map(|s| s.as_str()) == Some("preview") {
        if let (Ok(w), Ok(h)) = (args[3].parse::<i32>(), args[4].parse::<i32>()) {
            if w > 0 && h > 0 {
                width = w;
                height = h;
            }
        }
    } else {
        #[cfg(target_os = "windows")]
        {
            extern "system" {
                fn GetSystemMetrics(nIndex: i32) -> i32;
            }
            let screen_w = unsafe { GetSystemMetrics(0) }; // SM_CXSCREEN
            if screen_w > 0 {
                width = ((screen_w as f32) * 0.75).min(1920.0).max(1280.0) as i32;
                height = ((width as f32) * 9.0 / 16.0) as i32;
            }
        }
    }

    macroquad::window::Conf {
        window_title: "Phira".to_string(),
        window_width: width,
        window_height: height,
        window_resizable: true,
        high_dpi: false,
        headless: std::env::args().skip(1).next().as_deref() != Some("preview"),
        ..Default::default()
    }
}

async fn run_wrapped(f: impl Future<Output = Result<()>>) -> ! {
    if let Err(err) = f.await {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
    std::process::exit(0);
}

#[macroquad::main(build_conf)]
async fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    if std::env::args().len() > 1 {
        match std::env::args().skip(1).next().as_deref() {
            Some("render") => {
                run_wrapped(render::main()).await;
            }
            Some("preview") => {
                run_wrapped(preview::main()).await;
            }
            cmd => {
                eprintln!("Unknown subcommand: {cmd:?}");
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "windows")]
    enable_hidpi();

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("toggle".to_owned(), mtl!("tray-hide")))
        .add_item(CustomMenuItem::new("tasks".to_owned(), mtl!("tray-tasks")))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_owned(), mtl!("tray-quit")));
    let app = tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .manage(TaskQueue::new())
        .invoke_handler(tauri::generate_handler![
            is_the_only_instance,
            exit_program,
            show_in_folder,
            preview_chart,
            parse_chart,
            post_render,
            get_tasks,
            cancel_task,
            get_respacks,
            open_respack_folder,
            get_presets,
            add_preset,
            remove_preset,
            set_rpe_dir,
            unset_rpe_dir,
            get_rpe_charts,
            test_ffmpeg,
            open_app_folder,
            get_refresh_rate,
            send_preview_command,
        ])
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let window = app.get_window("main").unwrap();
                let visible = window.is_visible().unwrap();
                match id.as_str() {
                    "toggle" => {
                        app.tray_handle()
                            .get_item(&id)
                            .set_title(if visible {
                                mtl!("tray-show")
                            } else {
                                mtl!("tray-hide")
                            })
                            .unwrap();
                        if visible {
                            window.hide().unwrap();
                        } else {
                            window.show().unwrap();
                        }
                    }
                    "tasks" => {
                        if !visible {
                            window.show().unwrap();
                        }
                        window.eval("window.goto('tasks')").unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event
                    .window()
                    .app_handle()
                    .tray_handle()
                    .get_item("toggle")
                    .set_title(mtl!("tray-show"))
                    .unwrap();
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let resolver = app.path_resolver();
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap();

    let cache_dir = ensure_dir(
        resolver
            .app_cache_dir()
            .unwrap_or_else(|| exe_dir.to_owned()),
    );
    let lock_file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(cache_dir.join("app.lock"))
        .await?;
    if lock_file.try_lock_exclusive().is_ok() {
        LOCK_FILE.set(lock_file).unwrap();
    } else {
        eprintln!("Lock failed");
    }

    CONFIG_DIR
        .set(ensure_dir(
            resolver
                .app_config_dir()
                .unwrap_or_else(|| exe_dir.to_owned()),
        ))
        .unwrap();
    DATA_DIR
        .set(ensure_dir(
            resolver
                .app_data_dir()
                .unwrap_or_else(|| exe_dir.to_owned()),
        ))
        .unwrap();

    let asset_dir = resolver.resolve_resource("assets").unwrap();
    ASSET_PATH.set(asset_dir.clone()).unwrap();
    set_pc_assets_folder(&asset_dir.display().to_string());

    app.run(|_, _| {});

    Ok(())
}

#[tauri::command]
fn is_the_only_instance() -> bool {
    LOCK_FILE.get().is_some()
}

#[tauri::command]
fn exit_program() {
    std::process::exit(0);
}

#[tauri::command]
fn show_in_folder(path: &Path) -> Result<(), InvokeError> {
    (move || {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .args(["/select,", &path.display().to_string()]) // The comma after select is not a typo
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("gdbus")
                .args([
                    "call",
                    "--session",
                    "--dest",
                    "org.freedesktop.FileManager1",
                    "--object-path",
                    "/org/freedesktop/FileManager1",
                    "--method",
                    "org.freedesktop.FileManager1.ShowItems",
                    &format!("['file://{}']", path.canonicalize()?.display()),
                    "",
                ])
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(["-R", &path.display().to_string()])
                .spawn()?;
        }

        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn parse_chart(path: &Path) -> Result<ChartInfo, InvokeError> {
    wrap_async(async move {
        let mut fs: Box<dyn FileSystem + Send + Sync + 'static> =
            fs::fs_from_file(path).with_context(|| mtl!("read-chart-failed"))?;
        let info = fs::load_info(fs.deref_mut())
            .await
            .with_context(|| mtl!("load-info-failed"))?;
        Ok(info)
    })
    .await
}

#[cfg(target_os = "windows")]
mod win32 {
    #[repr(C)]
    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    pub const GWLP_HWNDPARENT: i32 = -8;
    pub const SW_HIDE: i32 = 0;
    pub const SW_SHOWNOACTIVATE: i32 = 4;
    pub const SWP_NOACTIVATE: u32 = 0x0010;
    pub const SWP_NOZORDER: u32 = 0x0004;

    extern "system" {
        pub fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
        pub fn GetWindowRect(hWnd: isize, lpRect: *mut RECT) -> i32;
        pub fn IsWindow(hWnd: isize) -> i32;
        pub fn IsIconic(hWnd: isize) -> i32;
        pub fn IsWindowVisible(hWnd: isize) -> i32;
        pub fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
        pub fn SetWindowLongPtrW(hWnd: isize, nIndex: i32, dwNewLong: isize) -> isize;
        pub fn SetWindowPos(
            hWnd: isize,
            hWndInsertAfter: isize,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: u32,
        ) -> i32;
    }
}

static PREVIEW_CHILD_STDIN: std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>> =
    std::sync::Mutex::new(None);

#[tauri::command]
fn send_preview_command(cmd: String) -> Result<(), InvokeError> {
    if let Ok(guard) = PREVIEW_CHILD_STDIN.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(cmd);
        }
    }
    Ok(())
}

#[tauri::command]
async fn preview_chart(window: tauri::Window, params: RenderParams) -> Result<(), InvokeError> {
    wrap_async(async move {
        let (pw, ph) = (1280, 720);

        let mut child = Command::new(std::env::current_exe()?)
            .arg("preview")
            .arg(ASSET_PATH.get().unwrap())
            .arg(pw.to_string())
            .arg(ph.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&params)?).as_bytes())
            .await?;

        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        if let Ok(mut guard) = PREVIEW_CHILD_STDIN.lock() {
            *guard = Some(cmd_tx);
        }

        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let _ = stdin.write_all(format!("{}\n", cmd).as_bytes()).await;
                let _ = stdin.flush().await;
            }
        });

        let app_handle = window.app_handle();
        if let Some(existing) = app_handle.get_window("preview_control") {
            let _ = existing.close();
        }

        let ctrl_win = tauri::WindowBuilder::new(
            &app_handle,
            "preview_control",
            tauri::WindowUrl::App("/preview-control".into()),
        )
        .title("Phira 预览控制")
        .inner_size(360.0, 720.0)
        .resizable(true)
        .always_on_top(false)
        .skip_taskbar(true)
        .build()?;

        let child_stdout = child.stdout.take().unwrap();
        let ctrl_win_events = ctrl_win.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(child_stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Some(json_str) = line.strip_prefix("STATUS:") {
                    let _ = ctrl_win_events.emit("preview-status", json_str);
                }
            }
            let _ = ctrl_win_events.close();
        });

        #[cfg(target_os = "windows")]
        {
            let ctrl_win_for_dock = ctrl_win.clone();
            tokio::spawn(async move {
                let ctrl_title: Vec<u16> = "Phira 预览控制\0".encode_utf16().collect();
                let phira_title: Vec<u16> = "Phira\0".encode_utf16().collect();
                let mut phira_hwnd = 0;
                let mut ctrl_hwnd = 0;

                for _ in 0..80 {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    if phira_hwnd == 0 {
                        phira_hwnd = unsafe { win32::FindWindowW(std::ptr::null(), phira_title.as_ptr()) };
                    }
                    if ctrl_hwnd == 0 {
                        ctrl_hwnd = unsafe { win32::FindWindowW(std::ptr::null(), ctrl_title.as_ptr()) };
                    }
                    if phira_hwnd != 0 && ctrl_hwnd != 0 {
                        break;
                    }
                }

                if phira_hwnd != 0 && ctrl_hwnd != 0 {
                    // Set Phira window as the OWNER of the Control window
                    // This ties its z-order, taskbar grouping, and alt-tab behavior directly to Phira!
                    unsafe {
                        win32::SetWindowLongPtrW(ctrl_hwnd, win32::GWLP_HWNDPARENT, phira_hwnd);
                    }

                    let mut last_rect: Option<win32::RECT> = None;
                    let mut was_minimized = false;

                    while unsafe { win32::IsWindow(phira_hwnd) } != 0 {
                        let is_iconic = unsafe { win32::IsIconic(phira_hwnd) } != 0;
                        let is_vis = unsafe { win32::IsWindowVisible(phira_hwnd) } != 0;

                        if is_iconic || !is_vis {
                            if !was_minimized {
                                unsafe { win32::ShowWindow(ctrl_hwnd, win32::SW_HIDE) };
                                was_minimized = true;
                            }
                        } else {
                            if was_minimized {
                                unsafe { win32::ShowWindow(ctrl_hwnd, win32::SW_SHOWNOACTIVATE) };
                                was_minimized = false;
                            }

                            let mut rect: win32::RECT = unsafe { std::mem::zeroed() };
                            if unsafe { win32::GetWindowRect(phira_hwnd, &mut rect) } != 0 {
                                let changed = last_rect.map_or(true, |lr| {
                                    lr.left != rect.left
                                        || lr.top != rect.top
                                        || lr.right != rect.right
                                        || lr.bottom != rect.bottom
                                });
                                if changed {
                                    last_rect = Some(rect);
                                    let target_x = rect.right;
                                    let target_y = rect.top;
                                    let target_h = (rect.bottom - rect.top).max(400);

                                    unsafe {
                                        win32::SetWindowPos(
                                            ctrl_hwnd,
                                            0,
                                            target_x,
                                            target_y,
                                            360,
                                            target_h,
                                            win32::SWP_NOACTIVATE | win32::SWP_NOZORDER,
                                        );
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                    }
                    let _ = ctrl_win_for_dock.close();
                }
            });
        }

        tokio::spawn(async move {
            let _ = child.wait().await;
            let _ = ctrl_win.close();
        });

        Ok(())
    })
    .await
}

#[tauri::command]
async fn post_render(queue: State<'_, TaskQueue>, params: RenderParams) -> Result<(), InvokeError> {
    wrap_async(async move {
        queue.post(params).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn get_tasks(queue: State<'_, TaskQueue>) -> Result<Vec<TaskView>, InvokeError> {
    wrap_async(async move { Ok(queue.tasks().await) }).await
}

#[tauri::command]
async fn cancel_task(queue: State<'_, TaskQueue>, id: u32) -> Result<(), InvokeError> {
    queue.cancel(id).await;
    Ok(())
}

#[derive(Serialize)]
struct RespackInfo {
    name: String,
    path: String,
}
#[tauri::command]
fn get_respacks() -> Result<Vec<RespackInfo>, InvokeError> {
    (|| {
        let dir = respack_dir()?;
        let mut names: Vec<RespackInfo> = dir
            .read_dir()?
            .filter_map(|it| {
                it.ok()
                    .filter(|it| it.path().is_file())
                    .map(|it| RespackInfo {
                        name: it.file_name().to_str().unwrap().to_owned(),
                        path: it.path().canonicalize().unwrap().display().to_string(),
                    })
            })
            .collect();
        names.sort_by(|x, y| x.name.cmp(&y.name));
        Ok(names)
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_respack_folder() -> Result<(), InvokeError> {
    (|| {
        open::that_detached(respack_dir()?)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

fn get_presets_file() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("presets.json");
    if file.exists() && !file.is_file() {
        bail!("presets.json is not a file");
    }
    Ok(file)
}

#[tauri::command]
async fn get_presets() -> Result<HashMap<String, RenderConfig>, InvokeError> {
    (|| {
        let file = get_presets_file()?;
        Ok(if !file.exists() {
            HashMap::new()
        } else {
            serde_json::from_reader(BufReader::new(File::open(file)?))?
        })
    })()
    .map_err(InvokeError::from_anyhow)
}

async fn save_presets(presets: &HashMap<String, RenderConfig>) -> Result<()> {
    serde_json::to_writer(BufWriter::new(File::create(get_presets_file()?)?), presets)?;
    Ok(())
}

#[tauri::command]
async fn add_preset(name: String, config: RenderConfig) -> Result<(), InvokeError> {
    let mut presets = get_presets().await?;
    wrap_async(async move {
        if presets.insert(name, config).is_some() {
            bail!(mtl!("preset-exists"));
        }
        save_presets(&presets).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn remove_preset(name: String) -> Result<(), InvokeError> {
    let mut presets = get_presets().await?;
    wrap_async(async move {
        if presets.remove(&name).is_none() {
            bail!(mtl!("preset-not-found"));
        }
        save_presets(&presets).await?;
        Ok(())
    })
    .await
}

fn rpe_dir() -> Result<Option<PathBuf>> {
    let file = CONFIG_DIR.get().unwrap().join("rpe_path.txt");
    if file.exists() {
        if !file.is_file() {
            bail!("rpe_path.txt is not a file");
        }
    } else {
        return Ok(None);
    }
    let dir: PathBuf = std::fs::read_to_string(file)?.into();
    Ok(if dir.exists() { Some(dir) } else { None })
}

#[derive(Serialize)]
pub struct RPEChartInfo {
    name: String,
    id: String,
    path: String,
    illustration: String,
    charter: String,
    #[serde(skip)]
    modified: SystemTime,
}

#[tauri::command]
fn set_rpe_dir(path: PathBuf) -> Result<(), InvokeError> {
    (|| {
        if !path.is_dir()
            || ["Chartlist.txt", "Resources"]
                .iter()
                .any(|it| !path.join(*it).exists())
        {
            bail!(mtl!("not-valid-rpe"));
        }
        std::fs::write(
            CONFIG_DIR.get().unwrap().join("rpe_path.txt"),
            path.canonicalize()?.display().to_string().as_bytes(),
        )?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn unset_rpe_dir() -> Result<(), InvokeError> {
    (|| {
        std::fs::remove_file(CONFIG_DIR.get().unwrap().join("rpe_path.txt"))?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn get_rpe_charts() -> Result<Option<Vec<RPEChartInfo>>, InvokeError> {
    (|| {
        let Some(dir) = rpe_dir()? else { return Ok(None) };
        let mut results = Vec::new();
        let mut name = None;
        let mut id = None;
        let mut chart = None;
        let mut illustration = None;
        let mut charter = None;
        macro_rules! commit {
            () => {
                (|| {
                    let id = id.take()?;
                    let path = dir.join("Resources").join(&id);
                    let metadata = path.join(chart.take()?).metadata();
                    results.push(RPEChartInfo {
                        name: name.take()?,
                        id,
                        path: path.display().to_string(),
                        illustration: path.join(illustration.take()?).display().to_string(),
                        charter: charter.take()?,
                        modified: metadata
                            .and_then(|it| it.modified())
                            .unwrap_or(SystemTime::UNIX_EPOCH),
                    });
                    Some(())
                })()
            };
        }
        for line in BufReader::new(File::open(dir.join("Chartlist.txt"))?).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line == "#" {
                commit!();
                continue;
            }
            let Some((key, value)) = line.split_once(':') else { continue };
            *(match key {
                "Name" => &mut name,
                "Path" => &mut id,
                "Chart" => &mut chart,
                "Picture" => &mut illustration,
                "Charter" => &mut charter,
                _ => continue,
            }) = Some(value.trim().to_owned());
        }
        commit!();

        results.sort_by_key(|it| it.modified);
        results.reverse();

        Ok(Some(results))
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn test_ffmpeg() -> Result<bool, InvokeError> {
    (|| Ok(find_ffmpeg()?.is_some()))().map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_app_folder() -> Result<(), InvokeError> {
    (|| {
        open::that_detached(std::env::current_exe()?.parent().unwrap())?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn get_refresh_rate() -> u32 {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // Use WMI via PowerShell to reliably query the physical display's refresh rate.
        // FFI approaches (GetDeviceCaps, EnumDisplaySettingsW) fail in Remote Desktop
        // sessions because they only see the remote display adapter (e.g., 32 Hz).
        // WMI's Win32_VideoController reports ALL adapters including physical GPUs.
        let result = std::process::Command::new("powershell")
            .args([
                "-NoProfile", "-Command",
                "(Get-CimInstance Win32_VideoController | Where-Object { $_.CurrentRefreshRate -gt 1 -and $_.Name -notmatch 'Remote|Virtual|Basic' } | Sort-Object CurrentRefreshRate -Descending | Select-Object -First 1).CurrentRefreshRate"
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        if let Ok(output) = result {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(rate) = s.trim().parse::<u32>() {
                    if rate > 1 {
                        return rate;
                    }
                }
            }
        }
        60
    }
    #[cfg(not(target_os = "windows"))]
    { 60 }
}
