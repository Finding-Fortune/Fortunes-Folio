use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct Note {
    id: i32,
    title: String,
    content: String,
    markdown: bool,
    tags: String, // Tags stored as a comma-separated string
}

// Shared connection for database access
struct AppState {
    conn: Mutex<Connection>,
}

impl AppState {
    /// Initialize the application state and create the `notes` table if it doesn't exist
    fn new() -> Self {
        let conn = Connection::open("notes.db").expect("Failed to connect to the database");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                markdown BOOLEAN NOT NULL,
                tags TEXT
            )",
            [],
        )
        .expect("Failed to create the `notes` table");

        AppState {
            conn: Mutex::new(conn),
        }
    }
}

#[tauri::command]
fn get_notes(state: tauri::State<AppState>) -> Vec<Note> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, title, content, markdown, tags FROM notes")
        .expect("Failed to prepare statement");

    let notes_iter = stmt
        .query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                markdown: row.get(3)?,
                tags: row.get(4)?,
            })
        })
        .expect("Failed to map query results");

    notes_iter.filter_map(|res| res.ok()).collect()
}

#[tauri::command]
fn add_note(
    state: tauri::State<AppState>,
    title: String,
    content: String,
    markdown: bool,
    tags: Vec<String>,
) {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO notes (title, content, markdown, tags) VALUES (?1, ?2, ?3, ?4)",
        params![title, content, markdown, tags.join(",")],
    )
    .expect("Failed to insert note");
}

#[tauri::command]
fn update_note(
    state: tauri::State<AppState>,
    id: i32,
    title: String,
    content: String,
    markdown: bool,
    tags: Vec<String>,
) {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "UPDATE notes SET title = ?1, content = ?2, markdown = ?3, tags = ?4 WHERE id = ?5",
        params![title, content, markdown, tags.join(","), id],
    )
    .expect("Failed to update note");
}

#[tauri::command]
fn delete_note(state: tauri::State<AppState>, id: i32) {
    let conn = state.conn.lock().unwrap();
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .expect("Failed to delete note");
}

#[tauri::command]
fn search_notes_by_tag(state: tauri::State<AppState>, tag: String) -> Vec<Note> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, title, content, markdown, tags FROM notes WHERE tags LIKE ?1")
        .expect("Failed to prepare statement");

    let tag_query = format!("%{}%", tag);
    let notes_iter = stmt
        .query_map([tag_query], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                markdown: row.get(3)?,
                tags: row.get(4)?,
            })
        })
        .expect("Failed to map query results");

    notes_iter.filter_map(|res| res.ok()).collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new()) // Initialize AppState and create the table
        .invoke_handler(tauri::generate_handler![
            get_notes,
            add_note,
            update_note,
            delete_note,
            search_notes_by_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
