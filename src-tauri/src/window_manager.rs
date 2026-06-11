use crate::note::{Note, NotesState};
use std::sync::Mutex;
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};

// 全局便签状态（由 main.rs 初始化）
pub struct AppState {
    pub notes: Mutex<NotesState>,
}

const DEFAULT_W: i32 = 280;
const DEFAULT_H: i32 = 280;
const OFFSET: i32 = 40; // 新窗口相对偏移量

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
    .min_inner_size(180.0, 140.0)
    .inner_size(note.rect.width as f64, note.rect.height as f64)
    .position(note.rect.x as f64, note.rect.y as f64)
    .always_on_top(note.pinned)
    .skip_taskbar(true)
    .focused(true)
    .build()?;

    // 确保新窗口获得焦点
    let _ = win.set_focus();

    Ok(win)
}

/// 为指定窗口把自己的 note id 暴露出去（前端通过 invoke("get_my_note_id") 读取）
/// 这里我们简单地约定：窗口 label 就是 "note-{id}"，前端解析即可。

/// 获取当前焦点窗口的位置，用作新建便利贴的基准位置
fn get_focused_window_pos<R: tauri::Runtime>(app: &impl Manager<R>) -> Option<(i32, i32)> {
    for win in app.webview_windows().values() {
        if win.is_focused().unwrap_or(false) {
            if let Ok(pos) = win.outer_position() {
                return Some((pos.x, pos.y));
            }
        }
    }
    None
}

/// 计算新窗口位置：基于焦点窗口偏移，带屏幕边界检测
fn calc_new_position<R: tauri::Runtime>(app: &impl Manager<R>) -> (i32, i32) {
    // 尝试基于焦点窗口偏移
    if let Some((fx, fy)) = get_focused_window_pos(app) {
        let nx = fx + OFFSET;
        let ny = fy + OFFSET;
        // 简单边界检测：若超出常见屏幕范围，回退到焦点窗口位置
        if nx < 0 || ny < 0 || nx > 3000 || ny > 2000 {
            return (fx, fy);
        }
        return (nx, ny);
    }

    // 无焦点窗口时，基于已有便签数量偏移
    let state: tauri::State<AppState> = app.state();
    let count = state.notes.lock().unwrap().notes.len() as i32;
    let x = 120 + count * OFFSET;
    let y = 120 + count * OFFSET;
    (x, y)
}

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

    // 计算位置：优先使用传入坐标，否则智能计算
    let (calc_x, calc_y) = if x.is_some() && y.is_some() {
        (x.unwrap(), y.unwrap())
    } else {
        calc_new_position(app)
    };

    let mut note = Note::new(
        id.clone(),
        &color,
        calc_x,
        calc_y,
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

    // 确保新窗口可见并获得焦点
    let _ = win.show();
    let _ = win.set_focus();

    // 保存到全局状态并落盘
    let state: tauri::State<AppState> = app.state();
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
