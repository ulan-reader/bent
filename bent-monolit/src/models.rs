use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------- Submissions ----------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "submission_type", rename_all = "UPPERCASE")]
pub enum SubmissionType {
    Employee,
    Inspector,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "submission_status", rename_all = "UPPERCASE")]
pub enum SubmissionStatus {
    New,
    InProgress,
    Rejected,
    Completed,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub text: String,
    pub file_url: Option<String>,
    pub department_id: Option<i64>,
    pub category_id: Option<i64>,
    pub channel: Option<String>,
    // Заполняется employee-флоу через body, если отправляет инспектор — игнорируется
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: SubmissionType,
    pub department_id: Option<i64>,
    pub category_id: Option<i64>,
    pub text: String,
    pub file_url: Option<String>,
    pub status: SubmissionStatus,
    pub reject_reason: Option<String>,
    pub channel: Option<String>,
    pub telegram_user_id: Option<i64>,
    pub created_by_inspector: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListSubmissionsQuery {
    pub status: Option<SubmissionStatus>,
    pub department_id: Option<i64>,
    pub category_id: Option<i64>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: SubmissionStatus,
    pub reject_reason: Option<String>,
}

// ---------- Token (Telegram one-time link) ----------

#[derive(Debug, Deserialize)]
pub struct GenerateTokenRequest {
    pub telegram_user_id: i64,
}

#[derive(Debug, Serialize)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
}

// ---------- Inspector auth ----------

#[derive(Debug, Deserialize)]
pub struct InspectorAuthRequest {
    pub tabel_number: i64,
}

// ---------- Admin / user auth (RBAC) ----------

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

// ---------- JWT claims ----------
// sub = user_id (UUID, для admin/RBAC) ИЛИ tabel_number (для inspector) — как строка.
// sid = session_id (UUID) — позволяет инвалидировать сессию через revoked-флаг в `sessions`.
// role различает inspector-токены (которые не имеют записи в `sessions`) от user-токенов.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub sid: String,
    pub role: TokenRole,
    pub exp: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenRole {
    User,
    Inspector,
}
