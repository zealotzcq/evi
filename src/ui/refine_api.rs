use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

const SECRET: &str = "evi_default_secret";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineSubmit {
    pub uuid: String,
    pub original: String,
    pub refined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditItem {
    pub task_id: i64,
    pub original: String,
    pub refined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSubmit {
    pub uuid: String,
    pub task_id: i64,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub contribution: i32,
    #[serde(default)]
    pub submit_count: i32,
    #[serde(default)]
    pub submit_completed: i32,
    #[serde(default)]
    pub submit_approved: i32,
    #[serde(default)]
    pub submit_reliability: f64,
    #[serde(default)]
    pub review_count: i32,
    #[serde(default)]
    pub review_completed: i32,
    #[serde(default)]
    pub review_majority: i32,
    #[serde(default)]
    pub review_reliability: f64,
    #[serde(default)]
    pub blocked: bool,
}

pub trait RefineApi: Send + Sync {
    fn submit_refine(&self, req: RefineSubmit) -> Result<ApiResponse>;
    fn get_pending_audit(&self, uuid: &str) -> Result<Option<AuditItem>>;
    fn submit_audit(&self, req: AuditSubmit) -> Result<ApiResponse>;
    fn get_user_profile(&self, uuid: &str) -> Result<UserProfile>;
}

fn code(interval: i64) -> String {
    use md5::Digest;
    let mut h = md5::Md5::new();
    h.update(SECRET.as_bytes());
    h.update(interval.to_string().as_bytes());
    format!("{:x}", h.finalize())[..6].to_string()
}

fn token6(c: &str, uuid: &str) -> String {
    use md5::Digest;
    let mut h = md5::Md5::new();
    h.update(c.as_bytes());
    h.update(uuid.as_bytes());
    format!("{:x}", h.finalize())[..6].to_string()
}

fn build_token(uuid: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let i = ts / 300;
    let c_prev = code(i - 1);
    let c_cur = code(i);
    let c_next = code(i + 1);
    token6(&c_prev, uuid) + &token6(&c_cur, uuid) + &token6(&c_next, uuid)
}

pub struct HttpRefineApi {
    base_url: String,
    agent: ureq::Agent,
}

impl HttpRefineApi {
    pub fn new(base_url: &str) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(15)))
            .build()
            .into();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            agent,
        }
    }

    fn post<T: Serialize>(&self, path: &str, uuid: &str, body: &T) -> Result<ApiResponse> {
        let token = build_token(uuid);
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .agent
            .post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .send(serde_json::to_string(body)?.as_str())?;
        let status = resp.status();
        let body_text = resp.into_body().read_to_string().context("read body")?;
        let api_resp: ApiResponse = serde_json::from_str(&body_text).context("parse response")?;
        if !api_resp.success {
            let code = api_resp.code.as_deref().unwrap_or("UNKNOWN");
            bail!(
                "API error {} ({}): {}",
                status,
                code,
                api_resp.message.as_deref().unwrap_or("unknown")
            );
        }
        Ok(api_resp)
    }
}

impl RefineApi for HttpRefineApi {
    fn submit_refine(&self, req: RefineSubmit) -> Result<ApiResponse> {
        self.post("/api/v1/refine/submit", &req.uuid, &req)
    }

    fn get_pending_audit(&self, uuid: &str) -> Result<Option<AuditItem>> {
        let body = serde_json::json!({ "uuid": uuid });
        let resp = self.post("/api/v1/refine/audit/next", uuid, &body)?;
        match resp.data {
            Some(v) if !v.is_null() => {
                let item: AuditItem = serde_json::from_value(v).context("parse audit item")?;
                Ok(Some(item))
            }
            _ => Ok(None),
        }
    }

    fn submit_audit(&self, req: AuditSubmit) -> Result<ApiResponse> {
        self.post("/api/v1/refine/audit/submit", &req.uuid, &req)
    }

    fn get_user_profile(&self, uuid: &str) -> Result<UserProfile> {
        let body = serde_json::json!({ "uuid": uuid });
        let resp = self.post("/api/v1/user/profile", uuid, &body)?;
        match resp.data {
            Some(v) => {
                let profile: UserProfile =
                    serde_json::from_value(v).context("parse user profile")?;
                Ok(profile)
            }
            None => bail!("no data in profile response"),
        }
    }
}

pub struct MockRefineApi {
    counter: std::sync::atomic::AtomicUsize,
    contribution: std::sync::atomic::AtomicI32,
}

impl MockRefineApi {
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicUsize::new(1),
            contribution: std::sync::atomic::AtomicI32::new(0),
        }
    }
}

impl RefineApi for MockRefineApi {
    fn submit_refine(&self, req: RefineSubmit) -> Result<ApiResponse> {
        log::info!(
            "MockRefineApi::submit_refine uuid={} original='{}' refined='{}'",
            req.uuid,
            req.original,
            req.refined
        );
        let c = self
            .contribution
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        Ok(ApiResponse {
            success: true,
            message: Some("提交成功".to_string()),
            code: None,
            data: Some(serde_json::json!({ "contribution": c })),
        })
    }

    fn get_pending_audit(&self, _uuid: &str) -> Result<Option<AuditItem>> {
        let n = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let originals = [
            "我今天去了一个很好玩的地方",
            "这个产品的质量非常好值得推荐",
            "昨天晚上看了一部特别精彩的动作电影",
            "北京的天气最近变化很大忽冷忽热",
            "我们公司正在开发一款新的语音识别软件",
            "人工智能技术的发展速度超出了很多人的预期",
            "春节是中国最重要的传统节日之一",
        ];
        let refineds = [
            "今天我去了一个非常有趣的地方。",
            "这款产品质量上乘，值得推荐。",
            "昨晚看了一部非常精彩的动作电影。",
            "最近北京天气变化很大，忽冷忽热。",
            "我们公司正在开发一款全新的语音识别软件。",
            "人工智能技术的发展速度远超许多人的预期。",
            "春节是中国最重要的传统节日之一。",
        ];
        let idx = n % originals.len();
        Ok(Some(AuditItem {
            task_id: n as i64,
            original: originals[idx].to_string(),
            refined: refineds[idx].to_string(),
        }))
    }

    fn submit_audit(&self, req: AuditSubmit) -> Result<ApiResponse> {
        log::info!(
            "MockRefineApi::submit_audit uuid={} task_id={} approved={}",
            req.uuid,
            req.task_id,
            req.approved
        );
        let c = self
            .contribution
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        Ok(ApiResponse {
            success: true,
            message: Some("审核已提交".to_string()),
            code: None,
            data: Some(serde_json::json!({ "contribution": c })),
        })
    }

    fn get_user_profile(&self, _uuid: &str) -> Result<UserProfile> {
        let c = self.contribution.load(std::sync::atomic::Ordering::Relaxed);
        Ok(UserProfile {
            contribution: c,
            submit_count: 0,
            submit_completed: 0,
            submit_approved: 0,
            submit_reliability: 1.0,
            review_count: 0,
            review_completed: 0,
            review_majority: 0,
            review_reliability: 1.0,
            blocked: false,
        })
    }
}

pub fn get_user_uuid() -> Option<String> {
    let path = crate::models::refine_db_path();
    let conn = rusqlite::Connection::open(path).ok()?;
    conn.query_row("SELECT uuid FROM user WHERE id = 1", [], |row| row.get(0))
        .ok()
}

pub fn create_api() -> Box<dyn RefineApi> {
    let url = crate::Config::load()
        .ok()
        .and_then(|c| c.server_url)
        .unwrap_or_else(|| "http://localhost:8080".to_string());
    match url.as_str() {
        "mock" => Box::new(MockRefineApi::new()),
        _ => Box::new(HttpRefineApi::new(&url)),
    }
}
