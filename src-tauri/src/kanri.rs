use crate::db::now_str;
use crate::model::{ImportResult, Project};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const PALETTE: [&str; 8] = [
    "#6C5CE7", "#12967A", "#D2603A", "#2F7BD6", "#B07714", "#C94F7C", "#5E8C1F", "#6B7A90",
];

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| {
            !c.is_whitespace()
                && !matches!(
                    c,
                    '-' | '_' | '·' | '/' | '、' | '，' | ',' | '。' | '.' | '（' | '）' | '(' | ')' | '：' | ':'
                )
        })
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn is_cjk(c: char) -> bool {
    matches!(c, '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}')
}

fn is_sep(c: char) -> bool {
    matches!(c, ' ' | '-' | '_' | '.' | '+' | '·' | '|' | '/')
}

/// Kanri stores descriptions as a small HTML blob. Keep the text, drop the markup.
fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => {
                in_tag = true;
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "…"
    }
}

fn due_of(v: &Value) -> Option<String> {
    let s = v.as_str()?;
    let d = s.get(0..10)?;
    if d.len() == 10 && d.as_bytes().get(4) == Some(&b'-') {
        Some(d.to_string())
    } else {
        None
    }
}

fn status_of(column: &str, card: &Value) -> String {
    let c = column.to_lowercase();
    if c.contains("done") || c.contains("完成") || c.contains("finished") || c.contains("finish") {
        return "done".into();
    }
    // abandoned work never belongs in 待排 either
    if c.contains("cancel") || c.contains("取消") || c.contains("放弃") || c.contains("archive") {
        return "done".into();
    }
    if c.contains("doing") || c.contains("progress") || c.contains("进行") {
        return "doing".into();
    }
    if card["isDueDateCompleted"].as_bool().unwrap_or(false) {
        return "done".into();
    }
    "todo".into()
}

/// The project prefix people put in front of a card title, e.g. `pokemon 更新`,
/// `career matching 调整`, `贩卖机 prod 准备`. Titles that start with a verb
/// (`使用 AI 整理仓库`) look the same, so a prefix only becomes a project when at
/// least two cards share it.
fn leading_token(title: &str) -> Option<String> {
    let chars: Vec<char> = title.trim().chars().collect();
    if chars.is_empty() {
        return None;
    }

    let mut end = 0;
    if is_cjk(chars[0]) {
        while end < chars.len() && (is_cjk(chars[end]) || chars[end].is_ascii_digit()) {
            end += 1;
        }
    } else {
        while end < chars.len()
            && !is_cjk(chars[end])
            && (chars[end].is_alphanumeric() || is_sep(chars[end]))
        {
            end += 1;
        }
    }

    let raw: String = chars[..end].iter().collect();
    let raw = raw.trim_end_matches(is_sep).trim();
    if raw.chars().count() < 2 {
        None
    } else {
        Some(raw.to_string())
    }
}

fn match_project(hay: &str, cands: &[(i64, Vec<String>)]) -> Option<i64> {
    let mut best: Option<(i64, usize)> = None;
    for (id, keys) in cands {
        for k in keys {
            let kk = normalize(k);
            if kk.chars().count() >= 2 && hay.contains(&kk) {
                let len = kk.chars().count();
                if best.map(|(_, l)| len > l).unwrap_or(true) {
                    best = Some((*id, len));
                }
            }
        }
    }
    best.map(|(id, _)| id)
}

struct Card {
    title: String,
    note: String,
    status: String,
    due: Option<String>,
    hay: String,
    token: Option<String>,
    project_id: Option<i64>,
}

pub fn default_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        PathBuf::from(appdata).join("tech.trobonox.kanri\\.kanri.dat")
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Application Support/tech.trobonox.kanri/.kanri.dat")
    }
}

pub fn import(conn: &Connection, path: Option<String>) -> Result<ImportResult, String> {
    let p = path.map(PathBuf::from).unwrap_or_else(default_path);
    let raw = fs::read_to_string(&p).map_err(|e| format!("读不到 Kanri 数据：{}", e))?;
    let data: Value = serde_json::from_str(&raw).map_err(|e| format!("解析失败：{}", e))?;

    let boards = data["boards"].as_array().cloned().unwrap_or_default();
    if boards.is_empty() {
        return Err("Kanri 文件里没有看板".into());
    }

    let mut existing: Vec<Project> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, name, color, aliases, archived FROM projects")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                    aliases: serde_json::from_str::<Vec<String>>(&r.get::<_, String>(3)?)
                        .unwrap_or_default(),
                    archived: false,
                    open_tasks: 0,
                    today_minutes: 0,
                    week_minutes: 0,
                    total_minutes: 0,
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            existing.push(row.map_err(|e| e.to_string())?);
        }
    }

    // pass 1: read every card, match what we can against existing projects
    let mut cards: Vec<Card> = Vec::new();
    let mut cards_total = 0i64;
    let mut skipped = 0i64;
    let mut cands: Vec<(i64, Vec<String>)> = existing
        .iter()
        .map(|p| {
            let mut keys = vec![p.name.clone()];
            keys.extend(p.aliases.iter().cloned());
            (p.id, keys)
        })
        .collect();

    for board in &boards {
        let columns = board["columns"].as_array().cloned().unwrap_or_default();
        for col in &columns {
            let col_name = col["title"].as_str().unwrap_or("");
            let items = col["cards"].as_array().cloned().unwrap_or_default();
            for card in &items {
                cards_total += 1;
                let title = strip_html(card["name"].as_str().unwrap_or(""))
                    .trim()
                    .to_string();
                if title.is_empty() {
                    skipped += 1;
                    continue;
                }
                let dup: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM tasks WHERE title = ?1",
                        params![title],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if dup > 0 {
                    skipped += 1;
                    continue;
                }

                let desc = strip_html(card["description"].as_str().unwrap_or(""));
                let hay = normalize(&format!("{} {}", title, desc));

                cards.push(Card {
                    note: clip(&desc, 200),
                    status: status_of(col_name, card),
                    due: due_of(&card["dueDate"]),
                    token: leading_token(&title),
                    project_id: match_project(&hay, &cands),
                    hay,
                    title,
                });
            }
        }
    }

    // pass 2: group the leftovers by title prefix; only repeated prefixes become projects
    let mut clusters: Vec<(String, i64)> = Vec::new();
    for c in cards.iter().filter(|c| c.project_id.is_none()) {
        let Some(tok) = &c.token else { continue };
        let key = normalize(tok);
        match clusters.iter_mut().find(|(name, _)| {
            let k = normalize(name);
            k == key || (k.chars().count() >= 2 && (k.starts_with(&key) || key.starts_with(&k)))
        }) {
            Some(entry) => {
                entry.1 += 1;
                if tok.chars().count() < entry.0.chars().count() {
                    entry.0 = tok.clone();
                }
            }
            None => clusters.push((tok.clone(), 1)),
        }
    }

    let mut projects_created = 0i64;
    let mut created: Vec<(String, i64)> = Vec::new();
    for (name, count) in clusters {
        if count < 2 {
            continue;
        }
        if existing.iter().any(|p| normalize(&p.name) == normalize(&name)) {
            continue;
        }
        let color = PALETTE[(projects_created as usize) % PALETTE.len()];
        conn.execute(
            "INSERT INTO projects(name, color, aliases, sort, created_at) VALUES(?1, ?2, '[]', ?3, ?4)",
            params![name, color, 100 + projects_created, now_str()],
        )
        .map_err(|e| e.to_string())?;
        created.push((name, conn.last_insert_rowid()));
        projects_created += 1;
    }

    // pass 3: cards the new projects obviously belong to (the project name shows
    // up in the title or description) get attached before we write anything
    if !created.is_empty() {
        for (name, id) in &created {
            cands.push((*id, vec![name.clone()]));
        }
        for c in cards.iter_mut().filter(|c| c.project_id.is_none()) {
            c.project_id = match_project(&c.hay, &cands);
        }
    }

    // pass 4: write the tasks
    let mut created_tasks = 0i64;
    let now = now_str();
    for (i, c) in cards.iter().enumerate() {
        let project_id = c.project_id.or_else(|| {
            let key = normalize(c.token.as_deref()?);
            created
                .iter()
                .find(|(name, _)| {
                    let k = normalize(name);
                    k == key || (k.chars().count() >= 2 && (k.starts_with(&key) || key.starts_with(&k)))
                })
                .map(|(_, id)| *id)
        });
        conn.execute(
            "INSERT INTO tasks(project_id, title, note, status, due_date, sort, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![project_id, c.title, c.note, c.status, c.due, i as i64 + 1, now],
        )
        .map_err(|e| e.to_string())?;
        created_tasks += 1;
    }

    Ok(ImportResult {
        path: p.display().to_string(),
        boards: boards.len() as i64,
        cards: cards_total,
        created_tasks,
        created_projects: projects_created,
        skipped,
    })
}
