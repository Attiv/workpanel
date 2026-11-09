use crate::model::Settings;
use serde_json::{json, Value};

fn clip(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        chars[..max].iter().collect::<String>() + "…"
    }
}

fn endpoint(base: &str) -> String {
    let b = base.trim().trim_end_matches('/');
    if b.ends_with("/chat/completions") {
        b.to_string()
    } else {
        format!("{}/chat/completions", b)
    }
}

async fn chat(s: &Settings, system: &str, user: &str) -> Result<String, String> {
    if s.ai_key.trim().is_empty() {
        return Err("还没有配置 API Key".into());
    }
    if s.ai_base.trim().is_empty() || s.ai_model.trim().is_empty() {
        return Err("还没有配置接口地址或模型".into());
    }
    let body = json!({
        "model": s.ai_model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ]
    });
    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?
        .post(endpoint(&s.ai_base))
        .bearer_auth(s.ai_key.trim())
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败：{}", e))?;

    let status = resp.status();
    let raw = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("接口返回 {}：{}", status.as_u16(), clip(&raw, 180)));
    }
    let v: Value = serde_json::from_str(&raw).map_err(|_| "接口返回的不是合法 JSON".to_string())?;
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "响应里没有文本内容".to_string())
}

pub async fn translate_batch(s: &Settings, texts: &[String]) -> Result<Vec<String>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let system = "You write the English side of a software engineer's daily report from terse Chinese work-log notes. \
For each numbered line: translate it into English, then polish that English into natural, professional workplace language. \
Keep every fact, technical term, product name and number unchanged; don't add anything the note doesn't imply. \
Use past tense, no trailing period, at most 20 words. \
Reply with a JSON object only: keys are the input numbers as strings, values are the polished English lines.";

    let mut user = String::from("Translate and polish each numbered line:\n");
    for (i, t) in texts.iter().enumerate() {
        user.push_str(&format!("{}. {}\n", i + 1, t));
    }

    let raw = chat(s, system, &user).await?;
    let start = raw.find('{').ok_or("模型没有返回 JSON")?;
    let end = raw.rfind('}').ok_or("模型没有返回 JSON")?;
    let json_str = &raw[start..=end];
    let parsed: Value = serde_json::from_str(json_str).map_err(|_| "模型返回的 JSON 解析失败")?;

    let mut out = Vec::with_capacity(texts.len());
    for i in 0..texts.len() {
        let key = (i + 1).to_string();
        let v = parsed
            .get(&key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().trim_end_matches('.').to_string());
        match v {
            Some(en) if !en.is_empty() => out.push(en),
            _ => out.push(texts[i].clone()),
        }
    }
    Ok(out)
}

pub async fn polish(s: &Settings, text: &str) -> Result<String, String> {
    let system = "你是技术工作日志的编辑。把用户的一句话改写成一条适合日报的描述：\
保留事实、技术名词和数字，去掉口语和情绪，一句话，不超过 40 个汉字，不加引号，不解释。只输出改写后的句子。";
    let out = chat(s, system, text).await?;
    Ok(out.trim_matches('"').trim().to_string())
}
