use crate::note::{Note, NotesState};
use std::sync::Mutex;
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewWindow, WebviewWindowBuilder};

// 全局便签状态（由 main.rs 初始化）
pub struct AppState {
    pub notes: Mutex<NotesState>,
}

const DEFAULT_W: i32 = 280;
const DEFAULT_H: i32 = 280;

/// 生成一个简单的唯一 id
pub fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("n{:x}", ms)
}

/// 为一张便利贴创建一个 Tauri 窗口；若 note.id 已存在窗口，则不重复创建
pub fn build_note_window<R: tauri::Runtime>(
    app: &impl Manager<R>,
    note: &Note,
) -> anyhow::Result<WebviewWindow<R>> {
    let label = note.window_label();
    // 如果已经有这个 label 的窗口，直接返回
    if let Some(existing) = app.get_webview_window(&label) {
        existing.show()?;
        existing.set_focus()?;
        return Ok(existing);
    }

    let win = WebviewWindowBuilder::new(
        app,
        label.clone(),
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("纸间")
    .decorations(false)
    .transparent(true)
    .resizable(true)
    .min_inner_size(LogicalSize::new(180.0, 140.0))
    .inner_size(LogicalSize::new(note.rect.width as f64, note.rect.height as f64))
    .position(LogicalPosition::new(note.rect.x as f64, note.rect.y as f64))
    .always_on_top(note.pinned)
    .skip_taskbar(true)
    .focused(true)
    .build()?;

    Ok(win)
}

/// 为指定窗口把自己的 note id 暴露出去（前端通过 invoke("get_my_note_id") 读取）
/// 这里我们简单地约定：窗口 label 就是 "note-{id}"，前端解析即可。

/// 创建一张全新的便利贴（新建窗口 + 写入 state）
pub fn create_note<R: tauri::Runtime>(
    app: &impl Manager<R>,
    color: Option<String>,
    content: Option<String>,
    x: Option<i32>,
    y: Option<i32>,
) -> anyhow::Result<String> {
    let id = new_id();
    let color = color.unwrap_or_else(|| "cream".to_string());

    let state: tauri::State<AppState> = app.state();

    // 如果没指定位置，给一个稍微偏移的默认位置（避免堆叠）
    let count = state.notes.lock().unwrap().notes.len() as i32;
    let default_x = 120 + count * 28;
    let default_y = 120 + count * 28;

    let mut note = Note::new(
        id.clone(),
        &color,
        x.unwrap_or(default_x),
        y.unwrap_or(default_y),
        DEFAULT_W,
        DEFAULT_H,
    );
    if let Some(text) = content {
        note.plain_text = text.clone();
        note.content = text;
    }

    let win = build_note_window(app, &note)?;

    // 设置鼠标穿透（如果标记为 true）
    if note.click_through {
        let _ = win.set_ignore_cursor_events(true);
    }

    // 保存到全局状态并落盘
    {
        let mut guard = state.notes.lock().unwrap();
        super::note::upsert_note(&mut guard, note);
        let _ = super::note::save_notes(&guard);
    }

    Ok(id)
}

/// 从当前便签状态恢复所有窗口（启动时调用）
pub fn restore_all_windows<R: tauri::Runtime>(app: &impl Manager<R>) -> anyhow::Result<()> {
    let state: tauri::State<AppState> = app.state();
    let notes = state.notes.lock().unwrap().notes.clone();
    if notes.is_empty() {
        return Ok(());
    }
    for note in notes {
        let _ = build_note_window(app, &note);
    }
    Ok(())
}
