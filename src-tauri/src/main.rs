mod note;
mod window_manager;

use crate::note::{load_notes, remove_note, save_notes, upsert_note, Item, Mode, Note, Rect};
use crate::window_manager::{
    build_note_window, create_note, new_id, restore_all_windows, AppState,
};

use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

// ========== IPC Commands ==========

/// 新建一张便利贴（创建窗口 + 写入 state）
#[tauri::command]
fn cmd_create_note(
    app: tauri::AppHandle,
    color: Option<String>,
    content: Option<String>,
    x: Option<i32>,
    y: Option<i32>,
) -> Result<String, String> {
    create_note(&app, color, content, x, y).map_err(|e| e.to_string())
}

/// 关闭当前便利贴窗口（同时从 state 中删除）
#[tauri::command]
fn cmd_close_note(window: tauri::Window, state: tauri::State<AppState>) -> Result<bool, String> {
    let label = window.label();
    let id = label.trim_start_matches("note-").to_string();
    {
        let mut guard = state.notes.lock().unwrap();
        remove_note(&mut guard, &id);
        let _ = save_notes(&guard);
    }
    let _ = window.close();
    Ok(true)
}

/// 置顶 / 取消置顶
#[tauri::command]
fn cmd_pin_note(
    window: tauri::Window,
    state: tauri::State<AppState>,
    pinned: bool,
) -> Result<bool, String> {
    let id = window.label().trim_start_matches("note-").to_string();
    let _ = window.set_always_on_top(pinned);
    {
        let mut guard = state.notes.lock().unwrap();
        if let Some(n) = guard.notes.iter_mut().find(|n| n.id == id) {
            n.pinned = pinned;
            let _ = save_notes(&guard);
        }
    }
    Ok(true)
}

/// 最小化当前窗口
#[tauri::command]
fn cmd_minimize_note(window: tauri::Window) -> Result<bool, String> {
    let _ = window.minimize();
    Ok(true)
}

/// 设置鼠标穿透
#[tauri::command]
fn cmd_set_click_through(
    window: tauri::Window,
    state: tauri::State<AppState>,
    flag: bool,
) -> Result<bool, String> {
    let id = window.label().trim_start_matches("note-").to_string();
    let _ = window.set_ignore_cursor_events(flag);
    {
        let mut guard = state.notes.lock().unwrap();
        if let Some(n) = guard.notes.iter_mut().find(|n| n.id == id) {
            n.click_through = flag;
            let _ = save_notes(&guard);
        }
    }
    Ok(true)
}

/// 获取当前窗口 rect
#[tauri::command]
fn cmd_get_window_rect(window: tauri::Window) -> Result<(i32, i32, i32, i32), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y, size.width as i32, size.height as i32))
}

/// 设置当前窗口位置 / 大小
#[tauri::command]
fn cmd_set_window_rect(
    window: tauri::Window,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<i32>,
    height: Option<i32>,
) -> Result<bool, String> {
    if let (Some(x), Some(y)) = (x, y) {
        let _ = window.set_position(x as f64, y as f64);
    }
    if let (Some(w), Some(h)) = (width, height) {
        let _ = window.set_size(w as f64, h as f64);
    }
    Ok(true)
}

/// 保存当前窗口的便签内容（支持 text / items 两种模式）
#[tauri::command]
fn cmd_save_note(
    window: tauri::Window,
    state: tauri::State<AppState>,
    content: String,
    plain_text: String,
    color: String,
    mode: Option<String>,       // "text" | "items"
    items_json: Option<String>, // JSON: [{id, text, done}]
) -> Result<bool, String> {
    let id = window.label().trim_start_matches("note-").to_string();
    let pos = window
        .outer_position()
        .map(|p| (p.x, p.y))
        .unwrap_or((120, 120));
    let size = window
        .outer_size()
        .map(|s| (s.width, s.height))
        .unwrap_or((280, 280));

    // 解析 mode
    let parsed_mode = match mode.as_deref() {
        Some("items") => Mode::Items,
        _ => Mode::Text,
    };

    // 解析 items
    let parsed_items: Vec<Item> = if parsed_mode == Mode::Items {
        items_json
            .and_then(|j| serde_json::from_str::<Vec<Item>>(&j).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    {
        let mut guard = state.notes.lock().unwrap();
        if let Some(n) = guard.notes.iter_mut().find(|n| n.id == id) {
            n.content = content;
            n.plain_text = plain_text;
            n.color = color;
            n.mode = parsed_mode.clone();
            n.items = parsed_items.clone();
            n.rect = Rect {
                x: pos.0,
                y: pos.1,
                width: size.0 as i32,
                height: size.1 as i32,
            };
            n.updated_at = chrono::Utc::now().timestamp_millis();
        } else {
            guard.notes.push(Note {
                id: id.clone(),
                color: color.clone(),
                mode: parsed_mode.clone(),
                content: content.clone(),
                plain_text: plain_text.clone(),
                items: parsed_items.clone(),
                pinned: false,
                click_through: false,
                rect: Rect {
                    x: pos.0,
                    y: pos.1,
                    width: size.0 as i32,
                    height: size.1 as i32,
                },
                reminder: None,
                created_at: chrono::Utc::now().timestamp_millis(),
                updated_at: chrono::Utc::now().timestamp_millis(),
            });
        }
        let _ = save_notes(&guard);
    }
    Ok(true)
}

/// 前端加载自己这张便利贴（返回 mode / content / items）
#[tauri::command]
fn cmd_get_my_note(
    window: tauri::Window,
    state: tauri::State<AppState>,
) -> Result<(String, String, String), String> {
    let id = window.label().trim_start_matches("note-").to_string();
    let guard = state.notes.lock().unwrap();
    if let Some(n) = guard.notes.iter().find(|n| n.id == id) {
        let mode = match n.mode {
            Mode::Items => "items".to_string(),
            _ => "text".to_string(),
        };
        let items = serde_json::to_string(&n.items).unwrap_or_else(|_| "[]".into());
        Ok((mode, n.content.clone(), items))
    } else {
        Ok(("text".to_string(), String::new(), "[]".to_string()))
    }
}

/// 加载所有便签（仅用于调试/搜索场景）
#[tauri::command]
fn cmd_load_all_notes(state: tauri::State<AppState>) -> Result<Vec<Note>, String> {
    Ok(state.notes.lock().unwrap().notes.clone())
}

/// 显示所有便签（托盘触发）
#[tauri::command]
fn cmd_show_all(app: tauri::AppHandle) -> Result<bool, String> {
    for win in app.webview_windows().values() {
        let _ = win.show();
        let _ = win.unminimize();
    }
    Ok(true)
}

/// 隐藏所有便签
#[tauri::command]
fn cmd_hide_all(app: tauri::AppHandle) -> Result<bool, String> {
    for win in app.webview_windows().values() {
        let _ = win.minimize();
    }
    Ok(true)
}

/// 退出
#[tauri::command]
fn cmd_quit_app(app: tauri::AppHandle) -> Result<bool, String> {
    app.exit(0);
    Ok(true)
}

// ========== 主入口 ==========

fn main() {
    let initial_state = load_notes();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 再次启动应用时：新建一张便利贴
            let _ = create_note(app, Some("cream".into()), None, None, None);
        }))
        .manage(AppState {
            notes: Mutex::new(initial_state),
        })
        .invoke_handler(tauri::generate_handler![
            cmd_create_note,
            cmd_close_note,
            cmd_pin_note,
            cmd_minimize_note,
            cmd_set_click_through,
            cmd_get_window_rect,
            cmd_set_window_rect,
            cmd_save_note,
            cmd_get_my_note,
            cmd_load_all_notes,
            cmd_show_all,
            cmd_hide_all,
            cmd_quit_app,
        ])
        .setup(|app| {
            // ========== 系统托盘 ==========
            // 使用 tauri::tray::MenuBuilder 构建菜单
            let new_item = MenuItemBuilder::with_id("new", "新建便利贴").build(app)?;
            let show_item = MenuItemBuilder::with_id("show", "显示全部").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "最小化全部").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出 纸间").build(app)?;
            let separator = PredefinedMenuItem::separator(app)?;

            let menu = MenuBuilder::new(app)
                .item(&new_item)
                .item(&separator)
                .item(&show_item)
                .item(&hide_item)
                .item(&separator)
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .tooltip("纸间便利贴")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().0.as_str() {
                    "new" => {
                        let _ = create_note(app, Some("cream".into()), None, None, None);
                    }
                    "show" => {
                        for win in app.webview_windows().values() {
                            let _ = win.show();
                            let _ = win.unminimize();
                        }
                    }
                    "hide" => {
                        for win in app.webview_windows().values() {
                            let _ = win.minimize();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击：切换显示/隐藏所有
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        let wins = app.webview_windows();
                        let any_visible = wins.values().any(|w| {
                            w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false)
                        });
                        if any_visible {
                            for win in wins.values() {
                                let _ = win.minimize();
                            }
                        } else {
                            for win in wins.values() {
                                let _ = win.show();
                                let _ = win.unminimize();
                            }
                        }
                    }
                })
                .build(app)?;

            // ========== 恢复窗口 ==========
            let state: tauri::State<AppState> = app.state();
            let has_notes = !state.notes.lock().unwrap().notes.is_empty();

            if has_notes {
                let _ = restore_all_windows(app);
            } else {
                // 首次启动：创建欢迎便签
                let welcome_html = r#"<h3>欢迎使用 纸间</h3><p>这是一张便利贴。</p><p>• 拖动<b>顶部</b>可移动窗口</p><p>• 拖动<b>右下角</b>可调整大小</p><p>• 点击<b>彩色圆点</b>切换颜色</p><p>• 点击<b>📌</b>置顶 / <b>+</b>新建 / <b>✕</b>关闭</p><p style="color:#8A8279;font-size:12px;margin-top:8px;">支持 <b>#</b> 标题 / <b>**粗体**</b> / <b>*斜体*</b> / <b>`代码`</b> / <b>&gt; 引用</b></p>"#.to_string();

                let id = new_id();
                let mut note = Note::new(id.clone(), "cream", 160, 140, 320, 340);
                note.content = welcome_html.clone();
                note.plain_text = "欢迎使用 纸间。拖动顶部可移动窗口...".to_string();
                {
                    let mut guard = state.notes.lock().unwrap();
                    upsert_note(&mut guard, note.clone());
                    let _ = save_notes(&guard);
                }
                let _ = build_note_window(app, &note);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
