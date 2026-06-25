use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------- Submissions ----------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "submission_status", rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    InReview,
    Approved,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub title: String,
    pub description: String,
    pub file_url: Option<String>,
    // Заполняется employee-флоу через query-параметр `token`,
    // если отправляет инспектор — игнорируется (берём из JWT)
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub file_url: Option<String>,
    pub status: SubmissionStatus,
    pub created_at: DateTime<Utc>,
    // None, если подано анонимно сотрудником
    pub submitted_by_tabel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListSubmissionsQuery {
    pub status: Option<SubmissionStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: SubmissionStatus,
    pub reject_reason: Option<String>,
}

// ---------- Inspector auth ----------

#[derive(Debug, Deserialize)]
pub struct InspectorAuthRequest {
    pub tabel_number: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}

// ---------- Admin auth ----------

#[derive(Debug, Deserialize)]
pub struct AdminLoginRequest {
    pub email: String,
    pub password: String,
}

// ---------- JWT claims ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // tabel_number или email
    pub role: Role,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Inspector,
    Admin,
}
