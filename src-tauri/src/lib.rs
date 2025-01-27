use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager}; 
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Note {
    id: i32,
    title: String,
    content: String,
    markdown: bool,
    tags: String, // Tags stored as a comma-separated string
    folderid: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct Folder {
    id: i32,
    name: String,
    parent_id: Option<i32>, // for nesting
}

// Shared connection for database access
struct AppState {
    conn: Mutex<Connection>,
}

impl AppState {
    /// Initialize the application state and create the `notes` table if it doesn't exist
    fn new() -> Self {
        let conn = Connection::open("notes.db").expect("Failed to connect to the database");

        // Create folders table:
        conn.execute(
            "CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER,
                FOREIGN KEY (parent_id) REFERENCES folders(id)
            )",
            [],
        ).expect("Failed to create the `folders` table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                markdown BOOLEAN NOT NULL,
                tags TEXT,
                folderid INTEGER REFERENCES folders(id) DEFAULT NULL
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_preferences (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        )
        .expect("Failed to create the `user_preferences` table");

        AppState {
            conn: Mutex::new(conn),
        }
    }
}

#[tauri::command]
fn get_dark_mode(state: tauri::State<AppState>) -> Result<bool, String> {
    let conn = state.conn.lock().unwrap();
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM user_preferences WHERE key = 'dark_mode'",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(value.unwrap_or("false".to_string()) == "true")
}

#[tauri::command]
fn set_dark_mode(state: tauri::State<AppState>, enabled: bool) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO user_preferences (key, value) VALUES ('dark_mode', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [enabled.to_string()],
    )
    .map_err(|e| format!("Failed to set dark mode: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_notes(state: tauri::State<AppState>) -> Vec<Note> {
    let conn = state.conn.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT
            n.id,
            n.title,
            n.content,
            n.markdown,
            GROUP_CONCAT(t.name, ',') AS tags,
            n.folderid
         FROM notes n
         LEFT JOIN note_tags nt ON n.id = nt.note_id
         LEFT JOIN tags t      ON nt.tag_id = t.id
         GROUP BY n.id
         "
    ).expect("Failed to prepare statement");

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
                    .join(","), // keep as comma-separated for the frontend
                folderid: row.get::<_, Option<i32>>(5)?,
            })
        })
        .expect("Failed to map query results");

    notes_iter.filter_map(|res| res.ok()).collect()
}

#[tauri::command]
fn add_note(
    state: tauri::State<AppState>,
    folderid: i32,
    title: String,
    content: String,
    markdown: bool,
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();

    conn.execute(
        "INSERT INTO notes (title, content, markdown, folderid)
         VALUES (?1, ?2, ?3, ?4)",
        params![title, content, markdown, folderid],
    ).map_err(|e| format!("Failed to insert note: {}", e))?;

    // Then handle tags as before...
    // ...
    Ok(())
}

#[tauri::command]
fn update_note(
    state: tauri::State<AppState>,
    id: i32,
    title: String,
    content: String,
    markdown: bool,
    tags: Vec<String>,
    folderid: Option<i32>, // new
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();

    // update the note itself
    conn.execute(
        "UPDATE notes
         SET title = ?1, content = ?2, markdown = ?3, folderid = ?4
         WHERE id = ?5",
        params![title, content, markdown, folderid, id],
    ).map_err(|e| format!("Failed to update note: {}", e))?;

    // Then handle clearing/adding tag associations
    // ...
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

    // Query to fetch notes with the specified tag
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.title, n.content, n.markdown, GROUP_CONCAT(t.name) AS tags
             FROM notes n
             INNER JOIN note_tags nt ON n.id = nt.note_id
             INNER JOIN tags t ON nt.tag_id = t.id
             WHERE n.id IN (
                 SELECT note_id
                 FROM note_tags
                 INNER JOIN tags ON note_tags.tag_id = tags.id
                 WHERE tags.name LIKE ?
             )
             GROUP BY n.id",
        )
        .expect("Failed to prepare search statement");

    let tag_query = format!("%{}%", tag);
    let notes_iter = stmt
        .query_map([tag_query], |row| {
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
                    .collect::<Vec<String>>()
                    .join(", "), // Convert back to comma-separated string
                folderid: row.get(5)?,
            })
        })
        .expect("Failed to execute query");

    notes_iter.filter_map(|res| res.ok()).collect()
}

#[tauri::command]
fn get_folders(state: tauri::State<AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, parent_id FROM folders").map_err(|e| e.to_string())?;
    let folder_iter = stmt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get::<_, Option<i32>>(2)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder.map_err(|e| e.to_string())?);
    }
    Ok(folders)
}

#[tauri::command]
fn add_folder(
    state: tauri::State<AppState>,
    name: String,
    parent_id: Option<i32>,
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO folders (name, parent_id) VALUES (?1, ?2)",
        params![name, parent_id],
    ).map_err(|e| format!("Failed to create folder: {}", e))?;
    Ok(())
}

#[tauri::command]
fn update_folder(
    state: tauri::State<AppState>,
    folderid: i32,
    new_name: String,
) -> Result<(), String> {
    let conn = state.conn.lock().unwrap();
    conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![new_name, folderid],
    ).map_err(|e| format!("Failed to update folder: {}", e))?;
    Ok(())
}

fn delete_folder_recursive(conn: &Connection, folder_id: i32) -> rusqlite::Result<()> {
    // 1) Find subfolders of this folder.
    //    For each child folder, call this function recursively.
    let mut stmt = conn.prepare("SELECT id FROM folders WHERE parent_id = ?1")?;
    let child_folder_ids = stmt.query_map([folder_id], |row| row.get::<_, i32>(0))?;

    for child_id_result in child_folder_ids {
        let child_id = child_id_result?;
        println!("Recursively deleting subfolder: {}", child_id);
        delete_folder_recursive(conn, child_id)?;
    }

    // 2) Delete all notes in this folder.
    println!("Deleting notes in folder: {}", folder_id);
    conn.execute("DELETE FROM notes WHERE folderid = ?1", [folder_id])?;

    // 3) Finally, delete this folder itself.
    println!("Deleting folder: {}", folder_id);
    conn.execute("DELETE FROM folders WHERE id = ?1", [folder_id])?;

    Ok(())
}

#[tauri::command]
fn delete_folder(
    state: tauri::State<AppState>,
    folderid: i32,
) -> Result<(), String> {
    println!("WFWERW");
    let conn = state.conn.lock().unwrap();

    println!("Starting recursive delete for folder: {}", folderid);

    if let Err(e) = delete_folder_recursive(&conn, folderid) {
        println!("Error while deleting folder {}: {}", folderid, e);
        return Err(format!("Failed to delete folder: {}", e));
    }

    println!("Successfully deleted folder {} and all its contents", folderid);
    Ok(())
}

#[tauri::command]
fn get_tags(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT DISTINCT name FROM tags ORDER BY name")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let tags_iter = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query tags: {}", e))?;

    let mut tags = vec![];
    for tag in tags_iter {
        tags.push(tag.map_err(|e| format!("Failed to get tag: {}", e))?);
    }

    Ok(tags)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new()) // Initialize AppState and create the table
        .invoke_handler(tauri::generate_handler![
            get_notes,
            add_note,
            update_note,
            delete_note,
            search_notes_by_tag,
            get_dark_mode,
            set_dark_mode,
            get_tags,
            get_folders,
            add_folder,
            update_folder,
            delete_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
