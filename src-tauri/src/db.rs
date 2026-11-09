use crate::model::*;
use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};

pub fn open(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         PRAGMA synchronous=NORMAL;",
    )?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    color       TEXT    NOT NULL DEFAULT '#6B7A90',
    aliases     TEXT    NOT NULL DEFAULT '[]',
    archived    INTEGER NOT NULL DEFAULT 0,
    sort        INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id   INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    title        TEXT    NOT NULL,
    note         TEXT    NOT NULL DEFAULT '',
    status       TEXT    NOT NULL DEFAULT 'todo',
    planned_date TEXT,
    due_date     TEXT,
    estimate_min INTEGER,
    done_at      TEXT,
    sort         INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS entries (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id    INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    day        TEXT    NOT NULL,
    minutes    INTEGER NOT NULL,
    note       TEXT    NOT NULL DEFAULT '',
    started_at TEXT,
    ended_at   TEXT,
    created_at TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_day ON entries(day);
CREATE INDEX IF NOT EXISTS idx_tasks_plan ON tasks(planned_date);

CREATE TABLE IF NOT EXISTS timer (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    task_id    INTEGER,
    project_id INTEGER,
    note       TEXT NOT NULL DEFAULT '',
    started_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS translations (
    zh         TEXT PRIMARY KEY,
    en         TEXT NOT NULL,
    created_at TEXT NOT NULL
);
"#,
    )
}

pub fn now_str() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
}

pub fn parse_date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap_or_else(|_| Local::now().date_naive())
}

pub fn week_start_of(date: &str) -> String {
    let d = parse_date(date);
    (d - Duration::days(d.weekday().num_days_from_monday() as i64))
        .format("%Y-%m-%d")
        .to_string()
}

fn aliases_from(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

/// Drag into 待排/今天/进行中/完成 ultimately lands here: move a task to a
/// day. Planning a finished task re-opens it, since it is clearly not done.
pub fn set_plan(conn: &Connection, id: i64, planned: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE tasks SET planned_date = ?1, updated_at = ?2,
                status = CASE WHEN ?1 IS NOT NULL AND status = 'done' THEN 'todo' ELSE status END,
                done_at = CASE WHEN ?1 IS NOT NULL AND status = 'done' THEN NULL ELSE done_at END
         WHERE id = ?3",
        params![planned, now_str(), id],
    )?;
    Ok(())
}

pub fn set_status(conn: &Connection, id: i64, status: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE tasks SET status = ?1, done_at = CASE WHEN ?1 = 'done' THEN ?2 ELSE NULL END,
                updated_at = ?2 WHERE id = ?3",
        params![status, now_str(), id],
    )?;
    Ok(())
}

pub fn load_projects(conn: &Connection, date: &str, week_start: &str) -> rusqlite::Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        r#"
SELECT p.id, p.name, p.color, p.aliases, p.archived,
  (SELECT COUNT(*) FROM tasks t WHERE t.project_id = p.id AND t.status IN ('todo','doing')),
  COALESCE((SELECT SUM(e.minutes) FROM entries e WHERE e.project_id = p.id AND e.day = ?1), 0),
  COALESCE((SELECT SUM(e.minutes) FROM entries e WHERE e.project_id = p.id AND e.day BETWEEN ?2 AND ?1), 0),
  COALESCE((SELECT SUM(e.minutes) FROM entries e WHERE e.project_id = p.id), 0)
FROM projects p
WHERE p.archived = 0
ORDER BY p.sort, p.id
"#,
    )?;
    let rows = stmt.query_map(params![date, week_start], |r| {
        Ok(Project {
            id: r.get(0)?,
            name: r.get(1)?,
            color: r.get(2)?,
            aliases: aliases_from(&r.get::<_, String>(3)?),
            archived: r.get::<_, i64>(4)? != 0,
            open_tasks: r.get(5)?,
            today_minutes: r.get(6)?,
            week_minutes: r.get(7)?,
            total_minutes: r.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn load_tasks(conn: &Connection, date: &str) -> rusqlite::Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        r#"
SELECT t.id, t.project_id, p.name, p.color, t.title, t.note, t.status,
       t.planned_date, t.due_date, t.estimate_min, t.created_at, t.done_at,
       COALESCE((SELECT SUM(e.minutes) FROM entries e WHERE e.task_id = t.id AND e.day = ?1), 0)
FROM tasks t
LEFT JOIN projects p ON p.id = t.project_id
WHERE t.status != 'cancelled'
ORDER BY CASE t.status WHEN 'doing' THEN 0 WHEN 'todo' THEN 1 WHEN 'done' THEN 2 ELSE 3 END,
         t.sort, t.id
"#,
    )?;
    let rows = stmt.query_map(params![date], |r| {
        Ok(Task {
            id: r.get(0)?,
            project_id: r.get(1)?,
            project_name: r.get(2)?,
            color: r.get(3)?,
            title: r.get(4)?,
            note: r.get(5)?,
            status: r.get(6)?,
            planned_date: r.get(7)?,
            due_date: r.get(8)?,
            estimate_min: r.get(9)?,
            created_at: r.get(10)?,
            done_at: r.get(11)?,
            today_minutes: r.get(12)?,
        })
    })?;
    rows.collect()
}

pub fn load_entries(conn: &Connection, date: &str) -> rusqlite::Result<Vec<Entry>> {
    let mut stmt = conn.prepare(
        r#"
SELECT e.id, e.task_id, e.project_id, p.name, p.color, t.title,
       e.day, e.minutes, e.note, e.started_at, e.ended_at
FROM entries e
LEFT JOIN projects p ON p.id = e.project_id
LEFT JOIN tasks t ON t.id = e.task_id
WHERE e.day = ?1
ORDER BY e.started_at IS NULL, e.started_at, e.id
"#,
    )?;
    let rows = stmt.query_map(params![date], |r| {
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
        })
    })?;
    rows.collect()
}

pub fn load_timer(conn: &Connection) -> rusqlite::Result<Option<TimerState>> {
    conn.query_row(
        r#"
SELECT tm.task_id, tm.project_id, p.name, t.title, tm.note, tm.started_at
FROM timer tm
LEFT JOIN projects p ON p.id = tm.project_id
LEFT JOIN tasks t ON t.id = tm.task_id
WHERE tm.id = 1
"#,
        [],
        |r| {
            Ok(TimerState {
                task_id: r.get(0)?,
                project_id: r.get(1)?,
                project_name: r.get(2)?,
                task_title: r.get(3)?,
                note: r.get(4)?,
                started_at: r.get(5)?,
            })
        },
    )
    .optional()
}

pub fn load_settings(conn: &Connection) -> rusqlite::Result<Settings> {
    let mut s = Settings::default();
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (k, v) = row?;
        match k.as_str() {
            "ai_base" => s.ai_base = v,
            "ai_model" => s.ai_model = v,
            "ai_key" => s.ai_key = v,
            "workday_minutes" => s.workday_minutes = v.parse().unwrap_or(480),
            "report_lang" => s.report_lang = v,
            _ => {}
        }
    }
    Ok(s)
}

pub fn save_settings(conn: &Connection, s: &Settings) -> rusqlite::Result<()> {
    let pairs = [
        ("ai_base", s.ai_base.clone()),
        ("ai_model", s.ai_model.clone()),
        ("ai_key", s.ai_key.clone()),
        ("workday_minutes", s.workday_minutes.to_string()),
        ("report_lang", s.report_lang.clone()),
    ];
    for (k, v) in pairs {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        )?;
    }
    Ok(())
}

pub fn load_stats(conn: &Connection, date: &str, week_start: &str) -> rusqlite::Result<Stats> {
    let (today_minutes, today_entries): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(minutes),0), COUNT(*) FROM entries WHERE day = ?1",
        params![date],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let week_minutes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(minutes),0) FROM entries WHERE day BETWEEN ?1 AND ?2",
        params![week_start, date],
        |r| r.get(0),
    )?;
    let open_tasks: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE status IN ('todo','doing')",
        [],
        |r| r.get(0),
    )?;
    let today_tasks: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE planned_date = ?1 AND status != 'done'",
        params![date],
        |r| r.get(0),
    )?;
    Ok(Stats {
        today_minutes,
        week_minutes,
        open_tasks,
        today_entries,
        today_tasks,
    })
}

pub fn load_series(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<SeriesPoint>> {
    let mut stmt = conn.prepare(
        "SELECT day, project_id, SUM(minutes) FROM entries
         WHERE day BETWEEN ?1 AND ?2 GROUP BY day, project_id",
    )?;
    let rows = stmt.query_map(params![from, to], |r| {
        Ok(SeriesPoint {
            day: r.get(0)?,
            project_id: r.get(1)?,
            minutes: r.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn build_snapshot(conn: &Connection, date: &str) -> rusqlite::Result<Snapshot> {
    let week_start = week_start_of(date);
    // 近七天柱状图可以跨周，序列起点取本周一和前 6 天里更早的那个
    let chart_start = (parse_date(date) - Duration::days(6)).format("%Y-%m-%d").to_string();
    let series_from = if chart_start < week_start {
        chart_start
    } else {
        week_start.clone()
    };
    Ok(Snapshot {
        date: date.to_string(),
        projects: load_projects(conn, date, &week_start)?,
        tasks: load_tasks(conn, date)?,
        entries: load_entries(conn, date)?,
        timer: load_timer(conn)?,
        settings: load_settings(conn)?,
        stats: load_stats(conn, date, &week_start)?,
        series: load_series(conn, &series_from, date)?,
    })
}

/// 结算正在运行的计时：写入一条 entry，删除计时行，返回新 entry 的 id。
pub fn settle_timer(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    let Some(t) = load_timer(conn)? else {
        return Ok(None);
    };
    let started = chrono::DateTime::parse_from_rfc3339(&t.started_at)
        .map(|d| d.with_timezone(&Local))
        .unwrap_or_else(|_| Local::now());
    let now = Local::now();
    let minutes = ((now - started).num_seconds() / 60).max(1);
    let day = started.format("%Y-%m-%d").to_string();
    let note = t
        .note
        .clone()
        .trim()
        .to_string();
    conn.execute(
        "INSERT INTO entries(task_id, project_id, day, minutes, note, started_at, ended_at, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            t.task_id,
            t.project_id,
            day,
            minutes,
            note,
            t.started_at,
            now.to_rfc3339(),
            now.to_rfc3339()
        ],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute("DELETE FROM timer WHERE id = 1", [])?;
    Ok(Some(id))
}
