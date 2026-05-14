use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineSubmit {
    pub uuid: String,
    pub original: String,
    pub refined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditItem {
    pub record_id: String,
    pub original: String,
    pub refined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSubmit {
    pub uuid: String,
    pub record_id: String,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

pub trait RefineApi: Send + Sync {
    fn submit_refine(&self, req: RefineSubmit) -> Result<ApiResponse>;
    fn get_pending_audit(&self, uuid: &str) -> Result<Option<AuditItem>>;
    fn submit_audit(&self, req: AuditSubmit) -> Result<ApiResponse>;
}

pub struct MockRefineApi {
    counter: std::sync::atomic::AtomicUsize,
}

impl MockRefineApi {
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicUsize::new(1),
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
        Ok(ApiResponse {
            success: true,
            message: Some("提交成功".to_string()),
            data: None,
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
            record_id: format!("mock_{}", n),
            original: originals[idx].to_string(),
            refined: refineds[idx].to_string(),
        }))
    }

    fn submit_audit(&self, req: AuditSubmit) -> Result<ApiResponse> {
        log::info!(
            "MockRefineApi::submit_audit uuid={} record_id={} approved={}",
            req.uuid,
            req.record_id,
            req.approved
        );
        Ok(ApiResponse {
            success: true,
            message: Some("审核已提交".to_string()),
            data: None,
        })
    }
}

pub fn get_user_uuid() -> Option<String> {
    let path = crate::models::refine_db_path();
    let conn = rusqlite::Connection::open(path).ok()?;
    conn.query_row("SELECT uuid FROM user WHERE id = 1", [], |row| row.get(0))
        .ok()
}
