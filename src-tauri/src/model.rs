use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub aliases: Vec<String>,
    pub archived: bool,
    pub open_tasks: i64,
    pub today_minutes: i64,
    pub week_minutes: i64,
    pub total_minutes: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub color: Option<String>,
    pub title: String,
    pub note: String,
    pub status: String,
    pub planned_date: Option<String>,
    pub due_date: Option<String>,
    pub estimate_min: Option<i64>,
    pub today_minutes: i64,
    pub attach_count: i64,
    pub done_at: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    pub task_id: Option<i64>,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub color: Option<String>,
    pub task_title: Option<String>,
    pub day: String,
    pub minutes: i64,
    pub note: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub attach_count: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i64,
    pub owner_type: String,
    pub owner_id: i64,
    pub mime: String,
    pub width: i64,
    pub height: i64,
    pub bytes: i64,
    pub data_url: String,
    pub created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInput {
    pub owner_type: String,
    pub owner_id: i64,
    pub data_url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
    pub task_id: Option<i64>,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub task_title: Option<String>,
    pub note: String,
    pub started_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub ai_base: String,
    pub ai_model: String,
    pub ai_key: String,
    pub workday_minutes: i64,
    pub report_lang: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ai_base: "https://api.deepseek.com/v1".into(),
            ai_model: "deepseek-chat".into(),
            ai_key: String::new(),
            workday_minutes: 480,
            report_lang: "both".into(),
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub today_minutes: i64,
    pub week_minutes: i64,
    pub open_tasks: i64,
    pub today_entries: i64,
    pub today_tasks: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SeriesPoint {
    pub day: String,
    pub project_id: Option<i64>,
    pub minutes: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub date: String,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub entries: Vec<Entry>,
    pub timer: Option<TimerState>,
    pub settings: Settings,
    pub stats: Stats,
    pub series: Vec<SeriesPoint>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReportLine {
    pub text: String,
    pub en: Option<String>,
    pub minutes: i64,
    pub task_id: Option<i64>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReportGroup {
    pub project_id: Option<i64>,
    pub name: String,
    pub color: String,
    pub minutes: i64,
    pub lines: Vec<ReportLine>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub markdown: String,
    pub markdown_zh: String,
    pub markdown_en: String,
    pub groups: Vec<ReportGroup>,
    pub total_minutes: i64,
    pub missing_en: Vec<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Translated {
    pub zh: String,
    pub en: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub path: String,
    pub boards: i64,
    pub cards: i64,
    pub created_tasks: i64,
    pub created_projects: i64,
    pub skipped: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub archived: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub id: Option<i64>,
    pub project_id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub status: String,
    pub planned_date: Option<String>,
    pub due_date: Option<String>,
    pub estimate_min: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInput {
    pub id: Option<i64>,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub day: String,
    pub minutes: i64,
    #[serde(default)]
    pub note: String,
}
