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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT UNIQUE NOT NULL
                )",
            [],
        )
        .expect("Failed to create the `tags` table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS note_tags (
                    note_id INTEGER NOT NULL,
                    tag_id INTEGER NOT NULL,
                    FOREIGN KEY (note_id) REFERENCES notes (id) ON DELETE CASCADE,
                    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE,
                    PRIMARY KEY (note_id, tag_id)
                )",
            [],
        )
        .expect("Failed to create the `note_tags` table");

        AppState {
            conn: Mutex::new(conn),
        }
    }
}

#[tauri::command]
fn get_notes(state: tauri::State<AppState>) -> Vec<Note> {
    let conn = state.conn.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.title, n.content, n.markdown, 
                    GROUP_CONCAT(t.name, ',') AS tags
             FROM notes n
             LEFT JOIN note_tags nt ON n.id = nt.note_id
             LEFT JOIN tags t ON nt.tag_id = t.id
             GROUP BY n.id",
        )
        .expect("Failed to prepare statement");

    let notes_iter = stmt
        .query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                markdown: row.get(3)?,
                tags: row
                    .get::<_, Option<String>>(4)?
                    .unwrap_or_default()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
                    .join(","), // Return as a comma-separated string
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
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();

    // Insert the new note
    conn.execute(
        "INSERT INTO notes (title, content, markdown) VALUES (?1, ?2, ?3)",
        params![title, content, markdown],
    )
    .map_err(|e| format!("Failed to insert note: {}", e))?;

    // Get the newly created note's ID
    let note_id: i64 = conn
        .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
        .map_err(|e| format!("Failed to retrieve new note ID: {}", e))?;

    // Process and associate tags
    for tag in tags {
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", [&tag])
            .map_err(|e| format!("Failed to insert tag: {}", e))?;

        let tag_id: i32 = conn
            .query_row(
                "SELECT id FROM tags WHERE name = ?",
                [&tag],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to retrieve tag ID: {}", e))?;

        conn.execute(
            "INSERT INTO note_tags (note_id, tag_id) VALUES (?, ?)",
            params![note_id, tag_id],
        )
        .map_err(|e| format!("Failed to associate tag with note: {}", e))?;
    }

    Ok(())
}


#[tauri::command]
fn update_note(
    state: tauri::State<AppState>,
    id: i32,
    title: String,
    content: String,
    markdown: bool,
    tags: Vec<String>, // Ensure this is passed as a Vec<String>
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();

    // Update the note
    conn.execute(
        "UPDATE notes SET title = ?, content = ?, markdown = ? WHERE id = ?",
        params![title, content, markdown, id],
    )
    .map_err(|e| format!("Failed to update note: {}", e))?;

    // Remove old tag associations
    conn.execute("DELETE FROM note_tags WHERE note_id = ?", [id])
        .map_err(|e| format!("Failed to clear old tags: {}", e))?;

    // Process each tag
    for tag in tags {
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", [&tag])
            .map_err(|e| format!("Failed to insert tag: {}", e))?;

        let tag_id: i32 = conn
            .query_row(
                "SELECT id FROM tags WHERE name = ?",
                [&tag],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to retrieve tag ID: {}", e))?;

        conn.execute(
            "INSERT INTO note_tags (note_id, tag_id) VALUES (?, ?)",
            params![id, tag_id],
        )
        .map_err(|e| format!("Failed to associate tag with note: {}", e))?;
    }

    Ok(())
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
