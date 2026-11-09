use crate::model::*;
use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub fn fmt_minutes(m: i64) -> String {
    if m < 60 {
        format!("{}m", m)
    } else if m % 60 == 0 {
        format!("{}h", m / 60)
    } else {
        format!("{}h{}m", m / 60, m % 60)
    }
}

fn weekday_zh(d: NaiveDate) -> &'static str {
    match d.weekday().num_days_from_monday() {
        0 => "周一",
        1 => "周二",
        2 => "周三",
        3 => "周四",
        4 => "周五",
        5 => "周六",
        _ => "周日",
    }
}

fn weekday_en(d: NaiveDate) -> &'static str {
    match d.weekday().num_days_from_monday() {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        _ => "Sun",
    }
}

fn clip(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn merge_notes(notes: &[String], task_title: &str, max: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    for raw in notes {
        for piece in raw.split(['；', ';', '。', '\n', '|']) {
            let p = piece.trim().trim_matches(',').trim_matches('，').trim();
            if p.is_empty() {
                continue;
            }
            if !parts.iter().any(|x| x == p) {
                parts.push(p.to_string());
            }
        }
    }
    let joined = if parts.is_empty() {
        task_title.trim().to_string()
    } else {
        parts.join("；")
    };
    clip(&joined, max)
}

struct Bucket {
    project_id: Option<i64>,
    name: String,
    color: String,
    minutes: i64,
    order: i64,
    tasks: Vec<BucketLine>,
    index: HashMap<String, usize>,
}

struct BucketLine {
    task_id: Option<i64>,
    title: String,
    minutes: i64,
    notes: Vec<String>,
}

pub fn build(conn: &Connection, d0: &str, d1: &str, lang: &str) -> rusqlite::Result<Report> {
    let mut stmt = conn.prepare(
        r#"
SELECT e.project_id, p.name, p.color, e.task_id, t.title, e.note, e.minutes
FROM entries e
LEFT JOIN projects p ON p.id = e.project_id
LEFT JOIN tasks t ON t.id = e.task_id
WHERE e.day BETWEEN ?1 AND ?2
ORDER BY e.day, e.id
"#,
    )?;

    let rows = stmt.query_map(params![d0, d1], |r| {
        Ok((
            r.get::<_, Option<i64>>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, i64>(6)?,
        ))
    })?;

    let mut buckets: Vec<Bucket> = Vec::new();
    let mut total: i64 = 0;

    for row in rows {
        let (project_id, pname, pcolor, task_id, ttitle, note, minutes) = row?;
        total += minutes;
        let name = pname.unwrap_or_else(|| "未归类".into());
        let color = pcolor.unwrap_or_else(|| "#6B7A90".into());

        let bi = match buckets.iter().position(|b| b.project_id == project_id) {
            Some(i) => i,
            None => {
                buckets.push(Bucket {
                    project_id,
                    name,
                    color,
                    minutes: 0,
                    order: buckets.len() as i64,
                    tasks: Vec::new(),
                    index: HashMap::new(),
                });
                buckets.len() - 1
            }
        };

        let b = &mut buckets[bi];
        b.minutes += minutes;
        let key = match task_id {
            Some(id) => format!("t{}", id),
            None => format!("n{}", note.trim()),
        };
        let li = match b.index.get(&key) {
            Some(i) => *i,
            None => {
                b.tasks.push(BucketLine {
                    task_id,
                    title: ttitle.clone().unwrap_or_default(),
                    minutes: 0,
                    notes: Vec::new(),
                });
                let i = b.tasks.len() - 1;
                b.index.insert(key, i);
                i
            }
        };
        b.tasks[li].minutes += minutes;
        if !note.trim().is_empty() {
            b.tasks[li].notes.push(note);
        }
    }

    buckets.sort_by(|a, b| b.minutes.cmp(&a.minutes).then(a.order.cmp(&b.order)));

    let mut missing: Vec<String> = Vec::new();
    let mut groups: Vec<ReportGroup> = Vec::new();

    for b in &buckets {
        let mut lines: Vec<ReportLine> = Vec::new();
        for t in &b.tasks {
            let zh = merge_notes(&t.notes, &t.title, 72);
            let en: Option<String> = conn
                .query_row(
                    "SELECT en FROM translations WHERE zh = ?1",
                    params![zh],
                    |r| r.get::<_, String>(0),
                )
                .ok();
            if en.is_none() && lang != "zh" && !missing.iter().any(|m| m == &zh) {
                missing.push(zh.clone());
            }
            lines.push(ReportLine {
                text: zh,
                en,
                minutes: t.minutes,
                task_id: t.task_id,
            });
        }
        groups.push(ReportGroup {
            project_id: b.project_id,
            name: b.name.clone(),
            color: b.color.clone(),
            minutes: b.minutes,
            lines,
        });
    }

    let markdown = render_markdown(&groups, d0, d1, lang, total);
    // 复制按钮要能各自拿到纯中/纯英版本，所以两种都渲染好一起返回
    let markdown_zh = render_markdown(&groups, d0, d1, "zh", total);
    let markdown_en = render_markdown(&groups, d0, d1, "en", total);
    Ok(Report {
        markdown,
        markdown_zh,
        markdown_en,
        groups,
        total_minutes: total,
        missing_en: missing,
    })
}

fn render_markdown(groups: &[ReportGroup], d0: &str, d1: &str, lang: &str, total: i64) -> String {
    let start = NaiveDate::parse_from_str(d0, "%Y-%m-%d").unwrap_or_else(|_| {
        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
    });
    let end = NaiveDate::parse_from_str(d1, "%Y-%m-%d").unwrap_or(start);

    let mut out = String::new();
    if lang == "en" {
        let range = if d0 == d1 {
            format!("{} ({})", d0, weekday_en(end))
        } else {
            format!("{} to {}", d0, d1)
        };
        out.push_str(&format!("# Daily report · {}\n\n", range));
    } else {
        let range = if d0 == d1 {
            format!("{}（{}）", d0, weekday_zh(end))
        } else {
            format!("{} 至 {}", d0, d1)
        };
        out.push_str(&format!("# 工作日报 · {}\n\n", range));
    }

    for g in groups {
        out.push_str(&format!("**{}** · {}\n", g.name, fmt_minutes(g.minutes)));
        for l in &g.lines {
            match lang {
                "zh" => out.push_str(&format!("- {}（{}）\n", l.text, fmt_minutes(l.minutes))),
                "en" => {
                    let en = l.en.clone().unwrap_or_else(|| l.text.clone());
                    out.push_str(&format!("- {} ({})\n", en, fmt_minutes(l.minutes)));
                }
                _ => {
                    out.push_str(&format!("- {}（{}）\n", l.text, fmt_minutes(l.minutes)));
                    if let Some(en) = &l.en {
                        out.push_str(&format!("  {}\n", en));
                    }
                }
            }
        }
        out.push('\n');
    }

    if lang == "en" {
        out.push_str(&format!("Total: {}\n", fmt_minutes(total)));
    } else {
        out.push_str(&format!("合计：{}\n", fmt_minutes(total)));
    }
    out
}
