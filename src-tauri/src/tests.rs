use crate::{db, kanri, report};
use rusqlite::{params, Connection};
use std::path::PathBuf;

fn tmp_db(name: &str) -> (Connection, PathBuf) {
    let path = std::env::temp_dir().join(format!("wp-test-{}-{}.db", name, std::process::id()));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{}", path.display(), suffix));
    }
    let conn = db::open(&path).unwrap();
    (conn, path)
}

#[test]
fn kanri_import_maps_projects_and_skips_duplicates() {
    let (conn, path) = tmp_db("import");
    conn.execute(
        "INSERT INTO projects(name, color, aliases, sort, created_at)
         VALUES('pokemon', '#111111', '[\"pok\"]', 1, 'x')",
        [],
    )
    .unwrap();

    let fixture = std::env::temp_dir().join(format!("wp-kanri-{}.json", std::process::id()));
    std::fs::write(
        &fixture,
        r#"{"boards":[{"title":"b","columns":[
            {"title":"Todo","cards":[
                {"name":"pokemon 灰度发布","description":"","dueDate":"2026-11-12T00:00:00.000Z"},
                {"name":"htf 首页改版","description":""},
                {"name":"htf 缓存优化","description":""}
            ]},
            {"title":"Done","cards":[{"name":"pokemon 登录修复","description":""}]}
        ]}]}"#,
    )
    .unwrap();

    let r = kanri::import(&conn, Some(fixture.display().to_string())).unwrap();
    assert_eq!(r.cards, 4);
    assert_eq!(r.created_tasks, 4);
    assert_eq!(r.created_projects, 1);

    let done: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks WHERE status = 'done'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(done, 1);

    let htf: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = (SELECT id FROM projects WHERE name = 'htf')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(htf, 2);

    let due: Option<String> = conn
        .query_row("SELECT due_date FROM tasks WHERE title LIKE 'pokemon 灰度%'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(due.as_deref(), Some("2026-11-12"));

    let again = kanri::import(&conn, Some(fixture.display().to_string())).unwrap();
    assert_eq!(again.created_tasks, 0);
    assert_eq!(again.skipped, 4);

    drop(conn);
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(fixture);
}

#[test]
fn report_merges_by_task_and_totals_minutes() {
    let (conn, path) = tmp_db("report");
    conn.execute(
        "INSERT INTO projects(name, color, aliases, sort, created_at)
         VALUES('pokemon', '#111111', '[]', 1, 'x')",
        [],
    )
    .unwrap();
    let pid = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO tasks(project_id, title, note, status, sort, created_at, updated_at)
         VALUES(?1, '修登录', '', 'done', 1, 'x', 'x')",
        params![pid],
    )
    .unwrap();
    let tid = conn.last_insert_rowid();

    for (min, note) in [(90, "修复时区偏移；补测试"), (30, "修复时区偏移")] {
        conn.execute(
            "INSERT INTO entries(task_id, project_id, day, minutes, note, created_at)
             VALUES(?1, ?2, '2026-11-09', ?3, ?4, 'x')",
            params![tid, pid, min, note],
        )
        .unwrap();
    }

    let r = report::build(&conn, "2026-11-09", "2026-11-09", "both").unwrap();
    assert_eq!(r.total_minutes, 120);
    assert_eq!(r.groups.len(), 1);
    assert_eq!(r.groups[0].minutes, 120);
    assert_eq!(r.groups[0].lines.len(), 1);
    assert_eq!(r.groups[0].lines[0].text, "修复时区偏移；补测试");
    assert_eq!(r.groups[0].lines[0].minutes, 120);
    assert!(r.markdown.contains("2h"));

    let en = report::build(&conn, "2026-11-09", "2026-11-09", "en").unwrap();
    assert_eq!(en.missing_en.len(), 1);

    drop(conn);
    let _ = std::fs::remove_file(path);
}

#[test]
fn kanri_import_maps_columns_prefixes_and_html() {
    let (conn, path) = tmp_db("import2");
    let fixture = std::env::temp_dir().join(format!("wp-kanri2-{}.json", std::process::id()));
    std::fs::write(
        &fixture,
        r#"{"boards":[{"title":"1","columns":[
            {"title":"Todo","cards":[
                {"name":"贩卖机 prod 准备","description":""},
                {"name":"贩卖机加首页","description":""}
            ]},
            {"title":"Doing","cards":[
                {"name":"career-matching","description":""},
                {"name":"career matching 调整","description":""},
                {"name":"REACH Attendence","description":"<p>签到系统</p>"}
            ]},
            {"title":"Cancel","cards":[{"name":"pokemon pwa","description":""}]}
        ]}]}"#,
    )
    .unwrap();

    let r = kanri::import(&conn, Some(fixture.display().to_string())).unwrap();
    assert_eq!(r.cards, 6);
    assert_eq!(r.created_tasks, 6);
    // 贩卖机 + career-matching repeat, REACH and pokemon appear once
    assert_eq!(r.created_projects, 2);

    let doing: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks WHERE status = 'doing'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(doing, 3);

    let cancelled_status: String = conn
        .query_row("SELECT status FROM tasks WHERE title = 'pokemon pwa'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(cancelled_status, "done");

    let career: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = (SELECT id FROM projects WHERE name LIKE 'career%')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(career, 2);

    let vending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = (SELECT id FROM projects WHERE name = '贩卖机')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(vending, 2);

    let note: String = conn
        .query_row(
            "SELECT note FROM tasks WHERE title = 'REACH Attendence'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(note, "签到系统");

    drop(conn);
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(fixture);
}

#[test]
fn planning_a_done_task_reopens_it() {
    let (conn, path) = tmp_db("plan");
    conn.execute(
        "INSERT INTO projects(name, color, aliases, sort, created_at)
         VALUES('pokemon', '#111111', '[]', 1, 'x')",
        [],
    )
    .unwrap();
    let pid = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO tasks(project_id, title, note, status, sort, created_at, updated_at)
         VALUES(?1, '修登录', '', 'todo', 1, 'x', 'x')",
        params![pid],
    )
    .unwrap();
    let tid = conn.last_insert_rowid();

    // drag 待排 -> 今天
    db::set_plan(&conn, tid, Some("2026-11-10")).unwrap();
    let (status, planned): (String, Option<String>) = conn
        .query_row(
            "SELECT status, planned_date FROM tasks WHERE id = ?1",
            params![tid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "todo");
    assert_eq!(planned.as_deref(), Some("2026-11-10"));

    // drag 今天 -> 完成
    db::set_status(&conn, tid, "done").unwrap();
    let (status, done_at): (String, Option<String>) = conn
        .query_row(
            "SELECT status, done_at FROM tasks WHERE id = ?1",
            params![tid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "done");
    assert!(done_at.is_some());

    // dragging a finished card back onto 今天 re-opens it
    db::set_plan(&conn, tid, Some("2026-11-11")).unwrap();
    let (status, done_at): (String, Option<String>) = conn
        .query_row(
            "SELECT status, done_at FROM tasks WHERE id = ?1",
            params![tid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "todo");
    assert!(done_at.is_none());

    // dragging back to 待排 clears the plan but keeps the status
    db::set_plan(&conn, tid, None).unwrap();
    let (status, planned): (String, Option<String>) = conn
        .query_row(
            "SELECT status, planned_date FROM tasks WHERE id = ?1",
            params![tid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "todo");
    assert!(planned.is_none());

    drop(conn);
    let _ = std::fs::remove_file(path);
}

#[test]
fn report_en_uses_cached_translation() {
    let (conn, path) = tmp_db("report-en");
    conn.execute(
        "INSERT INTO projects(name, color, aliases, sort, created_at)
         VALUES('pokemon', '#111111', '[]', 1, 'x')",
        [],
    )
    .unwrap();
    let pid = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO tasks(project_id, title, note, status, sort, created_at, updated_at)
         VALUES(?1, '修登录', '', 'done', 1, 'x', 'x')",
        params![pid],
    )
    .unwrap();
    let tid = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO entries(task_id, project_id, day, minutes, note, created_at)
         VALUES(?1, ?2, '2026-11-09', 45, '修登录', 'x')",
        params![tid, pid],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO translations(zh, en, created_at) VALUES('修登录', 'Fixed login', 'x')",
        [],
    )
    .unwrap();

    let r = report::build(&conn, "2026-11-09", "2026-11-09", "en").unwrap();
    assert!(r.missing_en.is_empty());
    assert_eq!(r.groups[0].lines[0].en.as_deref(), Some("Fixed login"));
    assert!(r.markdown.contains("# Daily report"));
    assert!(r.markdown.contains("- Fixed login (45m)"));
    assert!(r.markdown.contains("Total: 45m"));

    let both = report::build(&conn, "2026-11-09", "2026-11-09", "both").unwrap();
    assert!(both.markdown.contains("修登录"));
    assert!(both.markdown.contains("Fixed login"));

    drop(conn);
    let _ = std::fs::remove_file(path);
}

#[test]
fn timer_settles_into_entry() {
    let (conn, path) = tmp_db("timer");
    conn.execute(
        "INSERT INTO projects(name, color, aliases, sort, created_at)
         VALUES('pokemon', '#111111', '[]', 1, 'x')",
        [],
    )
    .unwrap();
    let pid = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO tasks(project_id, title, note, status, sort, created_at, updated_at)
         VALUES(?1, '修登录', '', 'todo', 1, 'x', 'x')",
        params![pid],
    )
    .unwrap();
    let tid = conn.last_insert_rowid();

    let past = (chrono::Local::now() - chrono::Duration::minutes(25)).to_rfc3339();
    conn.execute(
        "INSERT INTO timer(id, task_id, project_id, note, started_at) VALUES(1, ?1, ?2, '', ?3)",
        params![tid, pid, past],
    )
    .unwrap();

    let id = db::settle_timer(&conn).unwrap().unwrap();
    let (minutes, note_len): (i64, i64) = conn
        .query_row(
            "SELECT minutes, length(note) FROM entries WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!((25..=26).contains(&minutes));
    assert_eq!(note_len, 0);

    let remaining: i64 = conn
        .query_row("SELECT COUNT(*) FROM timer", [], |r| r.get(0))
        .unwrap();
    assert_eq!(remaining, 0);

    drop(conn);
    let _ = std::fs::remove_file(path);
}

#[test]
fn attachments_roundtrip_and_cascade() {
    let (conn, path) = tmp_db("attach");
    conn.execute(
        "INSERT INTO tasks(title, created_at, updated_at) VALUES('截图任务', 'x', 'x')",
        [],
    )
    .unwrap();
    let task_id = conn.last_insert_rowid();

    let (mime, data) = crate::api::decode_data_url("data:image/png;base64,iVBORw0KGgo=").unwrap();
    assert_eq!(mime, "image/png");
    assert_eq!(data.len(), 8);

    conn.execute(
        "INSERT INTO attachments(owner_type, owner_id, mime, width, height, bytes, data, created_at)
         VALUES('task', ?1, 'image/png', 320, 200, 8, ?2, 'x')",
        params![task_id, data],
    )
    .unwrap();

    assert_eq!(db::count_attachments(&conn, "task", task_id).unwrap(), 1);

    let list = db::load_attachments(&conn, "task", task_id).unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].data_url.starts_with("data:image/png;base64,"));
    assert_eq!(list[0].width, 320);
    assert_eq!(list[0].height, 200);

    // 任务列表要带出附件数量，卡片上才能显示角标
    let tasks = db::load_tasks(&conn, "2026-11-09").unwrap();
    assert_eq!(tasks[0].attach_count, 1);

    db::delete_attachments_of(&conn, "task", task_id).unwrap();
    assert_eq!(db::count_attachments(&conn, "task", task_id).unwrap(), 0);

    drop(conn);
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{}", path.display(), suffix));
    }
}

#[test]
fn decode_data_url_rejects_non_images() {
    assert!(crate::api::decode_data_url("data:text/plain;base64,aGk=").is_err());
    assert!(crate::api::decode_data_url("not-a-data-url").is_err());
    assert!(crate::api::decode_data_url("data:image/png,raw").is_err());
    assert!(crate::api::decode_data_url("data:image/png;base64,").is_err());
}
