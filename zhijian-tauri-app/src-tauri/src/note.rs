use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub color: String,      // cream | blue | pink | green | lavender | yellow
    pub content: String,    // HTML
    pub plain_text: String, // 纯文本（用于搜索）
    pub pinned: bool,
    pub click_through: bool,
    pub rect: Rect,
    pub reminder: Option<String>, // ISO string
    pub created_at: i64,
    pub updated_at: i64,
}

impl Note {
    pub fn new(id: String, color: &str, x: i32, y: i32, w: i32, h: i32) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            color: color.to_string(),
            content: String::new(),
            plain_text: String::new(),
            pinned: false,
            click_through: false,
            rect: Rect { x, y, width: w, height: h },
            reminder: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn window_label(&self) -> String {
        format!("note-{}", self.id)
    }
}

/// 全量便签集合
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NotesState {
    pub notes: Vec<Note>,
}

/// 返回 notes.json 所在路径：{appDataDir}/zhijian/notes.json
pub fn notes_file_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("zhijian");
    let _ = fs::create_dir_all(&path);
    path.push("notes.json");
    path
}

pub fn load_notes() -> NotesState {
    let path = notes_file_path();
    if !path.exists() {
        return NotesState::default();
    }
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str::<NotesState>(&text).unwrap_or_default(),
        Err(_) => NotesState::default(),
    }
}

pub fn save_notes(state: &NotesState) -> anyhow::Result<()> {
    let path = notes_file_path();
    let text = serde_json::to_string_pretty(state)?;
    fs::write(path, text)?;
    Ok(())
}

pub fn upsert_note(state: &mut NotesState, note: Note) {
    if let Some(existing) = state.notes.iter_mut().find(|n| n.id == note.id) {
        *existing = note;
    } else {
        state.notes.push(note);
    }
}

pub fn remove_note(state: &mut NotesState, id: &str) {
    state.notes.retain(|n| n.id != id);
}
