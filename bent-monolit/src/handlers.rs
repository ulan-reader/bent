use crate::AppState;
use crate::error::AppError;
use crate::extractors::{OptionalClaims, RequireAdmin};
use crate::models::{
    AdminLoginRequest, AuthResponse, Claims, CreateSubmissionRequest, InspectorAuthRequest,
    ListSubmissionsQuery, Role, SubmissionResponse, SubmissionStatus, UpdateStatusRequest,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json,
    extract::{Path, Query, State},
};

// ============== Submissions ==============

/// POST /api/submissions
/// Employee: req.token = Some(token), без заголовка Authorization
/// Inspector: req.token = None, Authorization: Bearer <jwt>
pub async fn create_submission(
    State(state): State<AppState>,
    OptionalClaims(maybe_claims): OptionalClaims,
    Json(req): Json<CreateSubmissionRequest>,
) -> Result<Json<SubmissionResponse>, AppError> {
    match (&req.token, &maybe_claims) {
        (Some(token), _) => create_from_employee_token(&state, token, req).await,
        (None, Some(claims)) => create_from_inspector(&state, claims, req).await,
        (None, None) => Err(AppError::BadRequest(
            "either token or a valid inspector session is required".into(),
        )),
    }
}

async fn create_from_employee_token(
    state: &AppState,
    token: &str,
    req: CreateSubmissionRequest,
) -> Result<Json<SubmissionResponse>, AppError> {
    let mut tx = state.pool.begin().await?;

    // for update — лочим строку, чтобы два параллельных запроса с одним
    // токеном не оба прошли проверку used = false до того, как кто-то
    // из них успеет проставить used = true (race condition на сабмите).
    let token_row = sqlx::query!(
        r#"
        select telegram_user_id, used, expires_at
        from one_time_tokens
        where token = $1
        for update
        "#,
        token
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::TokenNotFound)?;

    if token_row.used {
        return Err(AppError::TokenAlreadyUsed);
    }
    if token_row.expires_at < chrono::Utc::now() {
        return Err(AppError::TokenExpired);
    }

    let row = sqlx::query!(
        r#"
        insert into submissions (title, description, file_url, status, telegram_user_id)
        values ($1, $2, $3, 'pending', $4)
        returning id, title, description, file_url,
                  status as "status: SubmissionStatus", created_at
        "#,
        req.title,
        req.description,
        req.file_url,
        token_row.telegram_user_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "update one_time_tokens set used = true where token = $1",
        token
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(SubmissionResponse {
        id: row.id,
        title: row.title,
        description: row.description,
        file_url: row.file_url,
        status: row.status,
        created_at: row.created_at,
        submitted_by_tabel: None,
    }))
}

async fn create_from_inspector(
    state: &AppState,
    claims: &Claims,
    req: CreateSubmissionRequest,
) -> Result<Json<SubmissionResponse>, AppError> {
    let row = sqlx::query!(
        r#"
        insert into submissions (title, description, file_url, status, submitted_by_tabel)
        values ($1, $2, $3, 'pending', $4)
        returning id, title, description, file_url,
                  status as "status: SubmissionStatus", created_at
        "#,
        req.title,
        req.description,
        req.file_url,
        claims.sub,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(SubmissionResponse {
        id: row.id,
        title: row.title,
        description: row.description,
        file_url: row.file_url,
        status: row.status,
        created_at: row.created_at,
        submitted_by_tabel: Some(claims.sub.clone()),
    }))
}

/// GET /api/admin/submissions?status=pending&page=1&per_page=20
pub async fn list_submissions(
    State(state): State<AppState>,
    RequireAdmin(_claims): RequireAdmin,
    Query(query): Query<ListSubmissionsQuery>,
) -> Result<Json<Vec<SubmissionResponse>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query!(
        r#"
        select id, title, description, file_url,
               status as "status: SubmissionStatus", created_at, submitted_by_tabel
        from submissions
        where ($1::submission_status is null or status = $1)
        order by created_at desc
        limit $2 offset $3
        "#,
        query.status as Option<SubmissionStatus>,
        per_page,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;

    let result = rows
        .into_iter()
        .map(|r| SubmissionResponse {
            id: r.id,
            title: r.title,
            description: r.description,
            file_url: r.file_url,
            status: r.status,
            created_at: r.created_at,
            submitted_by_tabel: r.submitted_by_tabel,
        })
        .collect();

    Ok(Json(result))
}

/// PATCH /api/admin/submissions/{id}/status
pub async fn update_submission_status(
    State(state): State<AppState>,
    RequireAdmin(_claims): RequireAdmin,
    Path(id): Path<i64>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<Json<SubmissionResponse>, AppError> {
    if req.status == SubmissionStatus::Rejected && req.reject_reason.is_none() {
        return Err(AppError::BadRequest(
            "reject_reason is required when rejecting a submission".into(),
        ));
    }

    let row = sqlx::query!(
        r#"
        update submissions
        set status = $1, reject_reason = $2
        where id = $3
        returning id, title, description, file_url,
                  status as "status: SubmissionStatus", created_at, submitted_by_tabel
        "#,
        req.status as SubmissionStatus,
        req.reject_reason,
        id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::SubmissionNotFound)?;

    Ok(Json(SubmissionResponse {
        id: row.id,
        title: row.title,
        description: row.description,
        file_url: row.file_url,
        status: row.status,
        created_at: row.created_at,
        submitted_by_tabel: row.submitted_by_tabel,
    }))
}

// ============== Inspector auth ==============

/// POST /api/inspector/auth
pub async fn inspector_auth(
    State(state): State<AppState>,
    Json(req): Json<InspectorAuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let exists = sqlx::query_scalar!(
        "select exists(select 1 from inspectors where tabel_number = $1 and is_active = true)",
        req.tabel_number
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::InvalidTabelNumber);
    }

    // 12 часов — рабочая смена; подбери под реальный режим
    let token = state.jwt.issue(&req.tabel_number, Role::Inspector, 12)?;

    Ok(Json(AuthResponse { token }))
}

// ============== Admin auth ==============

/// POST /api/admin/login
pub async fn admin_login(
    State(state): State<AppState>,
    Json(req): Json<AdminLoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row = sqlx::query!(
        "select email, password_hash from admins where email = $1",
        req.email
    )
    .fetch_optional(&state.pool)
    .await?;

    // Early-return здесь технически открывает тайминг-атаку (отличить
    // существующий email от несуществующего по времени ответа).
    // Для MVP — ок, для продакшен-аудита замени на сверку с dummy-хешем.
    let row = row.ok_or(AppError::InvalidCredentials)?;

    let parsed_hash = PasswordHash::new(&row.password_hash).map_err(|_| AppError::PasswordHash)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    // 8 часов на рабочий день; ставь короче, если нет refresh-механизма
    let token = state.jwt.issue(&row.email, Role::Admin, 8)?;

    Ok(Json(AuthResponse { token }))
}
