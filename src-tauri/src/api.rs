use crate::db::{self, now_str};
use crate::model::*;
use crate::{ai, kanri, report};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rusqlite::params;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

const MAX_ATTACH_PER_OWNER: i64 = 12;
const MAX_ATTACH_BYTES: usize = 6 * 1024 * 1024;

pub struct AppState {
    pub conn: Mutex<rusqlite::Connection>,
    pub db_path: String,
}

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub fn snapshot(state: State<'_, AppState>, date: String) -> Result<Snapshot, String> {
    let conn = state.conn.lock().map_err(err)?;
    db::build_snapshot(&conn, &date).map_err(err)
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> serde_json::Value {
    let view = if cfg!(debug_assertions) {
        std::env::var("WORKPANEL_VIEW").unwrap_or_default()
    } else {
        String::new()
    };
    let zoom = if cfg!(debug_assertions) {
        std::env::var("WORKPANEL_ZOOM")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
    } else {
        None
    };
    let modal = if cfg!(debug_assertions) {
        std::env::var("WORKPANEL_MODAL").unwrap_or_default()
    } else {
        String::new()
    };
    let lang = if cfg!(debug_assertions) {
        std::env::var("WORKPANEL_LANG").unwrap_or_default()
    } else {
        String::new()
    };
    serde_json::json!({
        "dbPath": state.db_path,
        "version": env!("CARGO_PKG_VERSION"),
        "view": view,
        "zoom": zoom,
        "lang": lang,
        "modal": modal
    })
}

#[tauri::command]
pub fn save_project(state: State<'_, AppState>, project: ProjectInput) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(err)?;
    let aliases = serde_json::to_string(&project.aliases).unwrap_or_else(|_| "[]".into());
    match project.id {
        Some(id) => {
            conn.execute(
                "UPDATE projects SET name = ?1, color = ?2, aliases = ?3, archived = ?4 WHERE id = ?5",
                params![
                    project.name.trim(),
                    project.color,
                    aliases,
                    project.archived.unwrap_or(false) as i64,
                    id
                ],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            let sort: i64 = conn
                .query_row("SELECT COALESCE(MAX(sort), 0) + 1 FROM projects", [], |r| r.get(0))
                .map_err(err)?;
            conn.execute(
                "INSERT INTO projects(name, color, aliases, archived, sort, created_at)
                 VALUES(?1, ?2, ?3, 0, ?4, ?5)",
                params![project.name.trim(), project.color, aliases, sort, now_str()],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn save_task(state: State<'_, AppState>, task: TaskInput) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(err)?;
    let now = now_str();
    let status = if task.status.is_empty() {
        "todo".to_string()
    } else {
        task.status.clone()
    };
    match task.id {
        Some(id) => {
            conn.execute(
                "UPDATE tasks SET project_id = ?1, title = ?2, note = ?3, status = ?4,
                        planned_date = ?5, due_date = ?6, estimate_min = ?7, updated_at = ?8,
                        done_at = CASE WHEN ?4 = 'done' THEN COALESCE(done_at, ?8) ELSE NULL END
                 WHERE id = ?9",
                params![
                    task.project_id,
                    task.title.trim(),
                    task.note,
                    status,
                    task.planned_date,
                    task.due_date,
                    task.estimate_min,
                    now,
                    id
                ],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            let sort: i64 = conn
                .query_row("SELECT COALESCE(MAX(sort), 0) + 1 FROM tasks", [], |r| r.get(0))
                .map_err(err)?;
            conn.execute(
                "INSERT INTO tasks(project_id, title, note, status, planned_date, due_date,
                                   estimate_min, done_at, sort, created_at, updated_at)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    task.project_id,
                    task.title.trim(),
                    task.note,
                    status,
                    task.planned_date,
                    task.due_date,
                    task.estimate_min,
                    if status == "done" { Some(now.clone()) } else { None },
                    sort,
                    now
                ],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    db::delete_attachments_of(&conn, "task", id).map_err(err)?;
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
        .map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn set_task_status(state: State<'_, AppState>, id: i64, status: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    db::set_status(&conn, id, &status).map_err(err)
}

#[tauri::command]
pub fn set_task_plan(
    state: State<'_, AppState>,
    id: i64,
    planned_date: Option<String>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    db::set_plan(&conn, id, planned_date.as_deref()).map_err(err)
}

#[tauri::command]
pub fn start_timer(
    state: State<'_, AppState>,
    task_id: Option<i64>,
    note: String,
) -> Result<Option<TimerState>, String> {
    let conn = state.conn.lock().map_err(err)?;
    db::settle_timer(&conn).map_err(err)?;

    let mut project_id: Option<i64> = None;
    if let Some(tid) = task_id {
        project_id = conn
            .query_row("SELECT project_id FROM tasks WHERE id = ?1", params![tid], |r| {
                r.get::<_, Option<i64>>(0)
            })
            .ok()
            .flatten();
        conn.execute(
            "UPDATE tasks SET status = 'doing', updated_at = ?1 WHERE id = ?2 AND status != 'done'",
            params![now_str(), tid],
        )
        .map_err(err)?;
    }

    conn.execute("DELETE FROM timer WHERE id = 1", []).map_err(err)?;
    conn.execute(
        "INSERT INTO timer(id, task_id, project_id, note, started_at) VALUES(1, ?1, ?2, ?3, ?4)",
        params![task_id, project_id, note.trim(), now_str()],
    )
    .map_err(err)?;

    db::load_timer(&conn).map_err(err)
}

#[tauri::command]
pub fn stop_timer(state: State<'_, AppState>, note: Option<String>) -> Result<Option<Entry>, String> {
    let conn = state.conn.lock().map_err(err)?;
    if let Some(n) = note {
        let n = n.trim();
        if !n.is_empty() {
            conn.execute("UPDATE timer SET note = ?1 WHERE id = 1", params![n])
                .map_err(err)?;
        }
    }
    let id = db::settle_timer(&conn).map_err(err)?;
    let Some(id) = id else { return Ok(None) };
    let entry = conn
        .query_row(
            "SELECT e.id, e.task_id, e.project_id, p.name, p.color, t.title, e.day, e.minutes,
                    e.note, e.started_at, e.ended_at
             FROM entries e
             LEFT JOIN projects p ON p.id = e.project_id
             LEFT JOIN tasks t ON t.id = e.task_id
             WHERE e.id = ?1",
            params![id],
            |r| {
                Ok(Entry {
                    id: r.get(0)?,
                    task_id: r.get(1)?,
                    project_id: r.get(2)?,
                    project_name: r.get(3)?,
                    color: r.get(4)?,
                    task_title: r.get(5)?,
                    day: r.get(6)?,
                    minutes: r.get(7)?,
                    note: r.get(8)?,
                    started_at: r.get(9)?,
                    ended_at: r.get(10)?,
                    attach_count: 0,
                })
            },
        )
        .ok();
    Ok(entry)
}

#[tauri::command]
pub fn add_entry(state: State<'_, AppState>, entry: EntryInput) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(err)?;
    if entry.minutes <= 0 {
        return Err("时长必须大于 0".into());
    }
    conn.execute(
        "INSERT INTO entries(task_id, project_id, day, minutes, note, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entry.task_id,
            entry.project_id,
            entry.day,
            entry.minutes,
            entry.note.trim(),
            now_str()
        ],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_entry(state: State<'_, AppState>, entry: EntryInput) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    let id = entry.id.ok_or("缺少记录 id")?;
    if entry.minutes <= 0 {
        return Err("时长必须大于 0".into());
    }
    conn.execute(
        "UPDATE entries SET project_id = ?1, task_id = ?2, day = ?3, minutes = ?4, note = ?5
         WHERE id = ?6",
        params![
            entry.project_id,
            entry.task_id,
            entry.day,
            entry.minutes,
            entry.note.trim(),
            id
        ],
    )
    .map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_entry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    db::delete_attachments_of(&conn, "entry", id).map_err(err)?;
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])
        .map_err(err)?;
    Ok(())
}

pub(crate) fn decode_data_url(url: &str) -> Result<(String, Vec<u8>), String> {
    let rest = url
        .strip_prefix("data:")
        .ok_or_else(|| "unsupported image data".to_string())?;
    let (meta, payload) = rest
        .split_once(',')
        .ok_or_else(|| "unsupported image data".to_string())?;
    if !meta.contains(";base64") {
        return Err("image must be base64 encoded".into());
    }
    let mime = meta
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if !mime.starts_with("image/") {
        return Err("not an image".into());
    }
    let data = STANDARD
        .decode(payload)
        .map_err(|_| "image data is not valid base64".to_string())?;
    if data.is_empty() {
        return Err("image is empty".into());
    }
    if data.len() > MAX_ATTACH_BYTES {
        return Err("image is too large".into());
    }
    Ok((mime, data))
}

#[tauri::command]
pub fn add_attachment(state: State<'_, AppState>, input: AttachmentInput) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(err)?;
    if input.owner_type != "task" && input.owner_type != "entry" {
        return Err("unknown attachment owner".into());
    }
    let count = db::count_attachments(&conn, &input.owner_type, input.owner_id).map_err(err)?;
    if count >= MAX_ATTACH_PER_OWNER {
        return Err(format!("at most {} images per note", MAX_ATTACH_PER_OWNER));
    }
    let (mime, data) = decode_data_url(&input.data_url)?;
    conn.execute(
        "INSERT INTO attachments(owner_type, owner_id, mime, width, height, bytes, data, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.owner_type,
            input.owner_id,
            mime,
            input.width.max(0),
            input.height.max(0),
            data.len() as i64,
            data,
            now_str()
        ],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_attachments(
    state: State<'_, AppState>,
    owner_type: String,
    owner_id: i64,
) -> Result<Vec<Attachment>, String> {
    let conn = state.conn.lock().map_err(err)?;
    db::load_attachments(&conn, &owner_type, owner_id).map_err(err)
}

#[tauri::command]
pub fn delete_attachment(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    conn.execute("DELETE FROM attachments WHERE id = ?1", params![id])
        .map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn build_report(state: State<'_, AppState>,
    d0: String,
    d1: String,
    lang: String,
) -> Result<Report, String> {
    let conn = state.conn.lock().map_err(err)?;
    report::build(&conn, &d0, &d1, &lang).map_err(err)
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err)?;
    db::save_settings(&conn, &settings).map_err(err)
}

#[tauri::command]
pub fn import_kanri(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<ImportResult, String> {
    let conn = state.conn.lock().map_err(err)?;
    kanri::import(&conn, path)
}

#[tauri::command]
pub fn save_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("写入失败：{}", e))
}

#[tauri::command]
pub fn write_temp_html(content: String) -> Result<String, String> {
    let dir = std::env::temp_dir().join("workpanel");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建临时目录失败：{}", e))?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("report-{}.html", stamp));
    std::fs::write(&path, content).map_err(|e| format!("写入失败：{}", e))?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn translate_missing(
    app: AppHandle,
    texts: Vec<String>,
) -> Result<Vec<Translated>, String> {
    let settings = {
        let state = app.state::<AppState>();
        let conn = state.conn.lock().map_err(err)?;
        db::load_settings(&conn).map_err(err)?
    };

    let mut result: Vec<Translated> = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    {
        let state = app.state::<AppState>();
        let conn = state.conn.lock().map_err(err)?;
        for t in &texts {
            let cached: Option<String> = conn
                .query_row(
                    "SELECT en FROM translations WHERE zh = ?1",
                    params![t],
                    |r| r.get(0),
                )
                .ok();
            match cached {
                Some(en) => result.push(Translated { zh: t.clone(), en }),
                None => pending.push(t.clone()),
            }
        }
    }

    for chunk in pending.chunks(20) {
        let translated = ai::translate_batch(&settings, chunk).await?;
        let state = app.state::<AppState>();
        let conn = state.conn.lock().map_err(err)?;
        for (i, zh) in chunk.iter().enumerate() {
            let en = translated.get(i).cloned().unwrap_or_else(|| zh.clone());
            if en != *zh {
                conn.execute(
                    "INSERT INTO translations(zh, en, created_at) VALUES(?1, ?2, ?3)
                     ON CONFLICT(zh) DO UPDATE SET en = excluded.en",
                    params![zh, en, now_str()],
                )
                .map_err(err)?;
            }
            result.push(Translated {
                zh: zh.clone(),
                en: en.clone(),
            });
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn polish_text(app: AppHandle, text: String) -> Result<String, String> {
    let settings = {
        let state = app.state::<AppState>();
        let conn = state.conn.lock().map_err(err)?;
        db::load_settings(&conn).map_err(err)?
    };
    ai::polish(&settings, &text).await
}

#[tauri::command]
pub async fn test_ai(app: AppHandle) -> Result<String, String> {
    let settings = {
        let state = app.state::<AppState>();
        let conn = state.conn.lock().map_err(err)?;
        db::load_settings(&conn).map_err(err)?
    };
    let out = ai::polish(&settings, "今天修了登录问题").await?;
    Ok(out)
}
