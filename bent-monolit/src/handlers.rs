// // use crate::AppState;
// // use crate::error::AppError;
// // use crate::extractors::{OptionalClaims, RequireAdmin};
// // use crate::models::{
// //     AdminLoginRequest, AuthResponse, Claims, CreateSubmissionRequest, InspectorAuthRequest,
// //     ListSubmissionsQuery, Role, SubmissionResponse, SubmissionStatus, UpdateStatusRequest,
// // };
// // use argon2::{Argon2, PasswordHash, PasswordVerifier};
// // use axum::{
// //     Json,
// //     extract::{Path, Query, State},
// // };

// // // ============== Submissions ==============

// // /// POST /api/submissions
// // /// Employee: req.token = Some(token), без заголовка Authorization
// // /// Inspector: req.token = None, Authorization: Bearer <jwt>
// // pub async fn create_submission(
// //     State(state): State<AppState>,
// //     OptionalClaims(maybe_claims): OptionalClaims,
// //     Json(req): Json<CreateSubmissionRequest>,
// // ) -> Result<Json<SubmissionResponse>, AppError> {
// //     match (&req.token, &maybe_claims) {
// //         (Some(token), _) => create_from_employee_token(&state, token, req).await,
// //         (None, Some(claims)) => create_from_inspector(&state, claims, req).await,
// //         (None, None) => Err(AppError::BadRequest(
// //             "either token or a valid inspector session is required".into(),
// //         )),
// //     }
// // }

// // async fn create_from_employee_token(
// //     state: &AppState,
// //     token: &str,
// //     req: CreateSubmissionRequest,
// // ) -> Result<Json<SubmissionResponse>, AppError> {
// //     let mut tx = state.pool.begin().await?;

// //     // for update — лочим строку, чтобы два параллельных запроса с одним
// //     // токеном не оба прошли проверку used = false до того, как кто-то
// //     // из них успеет проставить used = true (race condition на сабмите).
// //     let token_row = sqlx::query!(
// //         r#"
// //         select telegram_user_id, used, expires_at
// //         from one_time_tokens
// //         where token = $1
// //         for update
// //         "#,
// //         token
// //     )
// //     .fetch_optional(&mut *tx)
// //     .await?
// //     .ok_or(AppError::TokenNotFound)?;

// //     if token_row.used {
// //         return Err(AppError::TokenAlreadyUsed);
// //     }
// //     if token_row.expires_at < chrono::Utc::now() {
// //         return Err(AppError::TokenExpired);
// //     }

// //     let row = sqlx::query!(
// //         r#"
// //         insert into submissions (title, description, file_url, status, telegram_user_id)
// //         values ($1, $2, $3, 'pending', $4)
// //         returning id, title, description, file_url,
// //                   status as "status: SubmissionStatus", created_at
// //         "#,
// //         req.title,
// //         req.description,
// //         req.file_url,
// //         token_row.telegram_user_id,
// //     )
// //     .fetch_one(&mut *tx)
// //     .await?;

// //     sqlx::query!(
// //         "update one_time_tokens set used = true where token = $1",
// //         token
// //     )
// //     .execute(&mut *tx)
// //     .await?;

// //     tx.commit().await?;

// //     Ok(Json(SubmissionResponse {
// //         id: row.id,
// //         title: row.title,
// //         description: row.description,
// //         file_url: row.file_url,
// //         status: row.status,
// //         created_at: row.created_at,
// //         submitted_by_tabel: None,
// //     }))
// // }

// // async fn create_from_inspector(
// //     state: &AppState,
// //     claims: &Claims,
// //     req: CreateSubmissionRequest,
// // ) -> Result<Json<SubmissionResponse>, AppError> {
// //     let row = sqlx::query!(
// //         r#"
// //         insert into submissions (title, description, file_url, status, submitted_by_tabel)
// //         values ($1, $2, $3, 'pending', $4)
// //         returning id, title, description, file_url,
// //                   status as "status: SubmissionStatus", created_at
// //         "#,
// //         req.title,
// //         req.description,
// //         req.file_url,
// //         claims.sub,
// //     )
// //     .fetch_one(&state.pool)
// //     .await?;

// //     Ok(Json(SubmissionResponse {
// //         id: row.id,
// //         title: row.title,
// //         description: row.description,
// //         file_url: row.file_url,
// //         status: row.status,
// //         created_at: row.created_at,
// //         submitted_by_tabel: Some(claims.sub.clone()),
// //     }))
// // }

// // /// GET /api/admin/submissions?status=pending&page=1&per_page=20
// // pub async fn list_submissions(
// //     State(state): State<AppState>,
// //     RequireAdmin(_claims): RequireAdmin,
// //     Query(query): Query<ListSubmissionsQuery>,
// // ) -> Result<Json<Vec<SubmissionResponse>>, AppError> {
// //     let page = query.page.unwrap_or(1).max(1);
// //     let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
// //     let offset = (page - 1) * per_page;

// //     let rows = sqlx::query!(
// //         r#"
// //         select id, title, description, file_url,
// //                status as "status: SubmissionStatus", created_at, submitted_by_tabel
// //         from submissions
// //         where ($1::submission_status is null or status = $1)
// //         order by created_at desc
// //         limit $2 offset $3
// //         "#,
// //         query.status as Option<SubmissionStatus>,
// //         per_page,
// //         offset,
// //     )
// //     .fetch_all(&state.pool)
// //     .await?;

// //     let result = rows
// //         .into_iter()
// //         .map(|r| SubmissionResponse {
// //             id: r.id,
// //             title: r.title,
// //             description: r.description,
// //             file_url: r.file_url,
// //             status: r.status,
// //             created_at: r.created_at,
// //             submitted_by_tabel: r.submitted_by_tabel,
// //         })
// //         .collect();

// //     Ok(Json(result))
// // }

// // /// PATCH /api/admin/submissions/{id}/status
// // pub async fn update_submission_status(
// //     State(state): State<AppState>,
// //     RequireAdmin(_claims): RequireAdmin,
// //     Path(id): Path<i64>,
// //     Json(req): Json<UpdateStatusRequest>,
// // ) -> Result<Json<SubmissionResponse>, AppError> {
// //     if req.status == SubmissionStatus::Rejected && req.reject_reason.is_none() {
// //         return Err(AppError::BadRequest(
// //             "reject_reason is required when rejecting a submission".into(),
// //         ));
// //     }

// //     let row = sqlx::query!(
// //         r#"
// //         update submissions
// //         set status = $1, reject_reason = $2
// //         where id = $3
// //         returning id, title, description, file_url,
// //                   status as "status: SubmissionStatus", created_at, submitted_by_tabel
// //         "#,
// //         req.status as SubmissionStatus,
// //         req.reject_reason,
// //         id,
// //     )
// //     .fetch_optional(&state.pool)
// //     .await?
// //     .ok_or(AppError::SubmissionNotFound)?;

// //     Ok(Json(SubmissionResponse {
// //         id: row.id,
// //         title: row.title,
// //         description: row.description,
// //         file_url: row.file_url,
// //         status: row.status,
// //         created_at: row.created_at,
// //         submitted_by_tabel: row.submitted_by_tabel,
// //     }))
// // }

// // // ============== Inspector auth ==============

// // /// POST /api/inspector/auth
// // pub async fn inspector_auth(
// //     State(state): State<AppState>,
// //     Json(req): Json<InspectorAuthRequest>,
// // ) -> Result<Json<AuthResponse>, AppError> {
// //     let exists = sqlx::query_scalar!(
// //         "select exists(select 1 from inspectors where tabel_number = $1 and is_active = true)",
// //         req.tabel_number
// //     )
// //     .fetch_one(&state.pool)
// //     .await?
// //     .unwrap_or(false);

// //     if !exists {
// //         return Err(AppError::InvalidTabelNumber);
// //     }

// //     // 12 часов — рабочая смена; подбери под реальный режим
// //     let token = state.jwt.issue(&req.tabel_number, Role::Inspector, 12)?;

// //     Ok(Json(AuthResponse { token }))
// // }

// // // ============== Admin auth ==============

// // /// POST /api/admin/login
// // pub async fn admin_login(
// //     State(state): State<AppState>,
// //     Json(req): Json<AdminLoginRequest>,
// // ) -> Result<Json<AuthResponse>, AppError> {
// //     let row = sqlx::query!(
// //         "select email, password_hash from admins where email = $1",
// //         req.email
// //     )
// //     .fetch_optional(&state.pool)
// //     .await?;

// //     // Early-return здесь технически открывает тайминг-атаку (отличить
// //     // существующий email от несуществующего по времени ответа).
// //     // Для MVP — ок, для продакшен-аудита замени на сверку с dummy-хешем.
// //     let row = row.ok_or(AppError::InvalidCredentials)?;

// //     let parsed_hash = PasswordHash::new(&row.password_hash).map_err(|_| AppError::PasswordHash)?;

// //     Argon2::default()
// //         .verify_password(req.password.as_bytes(), &parsed_hash)
// //         .map_err(|_| AppError::InvalidCredentials)?;

// //     // 8 часов на рабочий день; ставь короче, если нет refresh-механизма
// //     let token = state.jwt.issue(&row.email, Role::Admin, 8)?;

// //     Ok(Json(AuthResponse { token }))
// // }
// use crate::AppState;
// use crate::auth::jwt::create_access_token;
// use crate::error::AppError;
// use crate::extractors::{OptionalClaims, RequirePermission, RequireUser};
// use crate::models::{
//     Claims, CreateSubmissionRequest, GenerateTokenRequest, GenerateTokenResponse,
//     InspectorAuthRequest, InspectorAuthResponse, ListSubmissionsQuery, LoginRequest, LoginResponse,
//     MeResponse, RefreshRequest, SubmissionResponse, SubmissionStatus, SubmissionType,
//     UpdateStatusRequest, ValidateTokenResponse,
// };
// use argon2::{Argon2, PasswordHash, PasswordVerifier};
// use axum::{
//     Json,
//     extract::{Path, Query, State},
// };
// use chrono::{Duration, Utc};
// use rand::Rng;
// use rand::distr::Alphanumeric;
// use sha2::{Digest, Sha256};
// use uuid::Uuid;

// // ============== One-time tokens (employee flow entry point) ==============

// /// POST /api/token/generate — вызывается ботом при нажатии "Обращение"
// pub async fn generate_token(
//     State(state): State<AppState>,
//     Json(req): Json<GenerateTokenRequest>,
// ) -> Result<Json<GenerateTokenResponse>, AppError> {
//     let token: String = rand::rng()
//         .sample_iter(&Alphanumeric)
//         .take(64)
//         .map(char::from)
//         .collect();

//     let expires_at = Utc::now() + Duration::hours(24);

//     sqlx::query!(
//         "insert into one_time_tokens (token, telegram_user_id, expires_at) values ($1, $2, $3)",
//         token,
//         req.telegram_user_id,
//         expires_at,
//     )
//     .execute(&state.pool)
//     .await?;

//     Ok(Json(GenerateTokenResponse {
//         url: format!("{}/form?token={}", state.public_base_url, token),
//         token,
//     }))
// }

// /// GET /api/token/validate/{token} — фронт дёргает при открытии формы
// pub async fn validate_token(
//     State(state): State<AppState>,
//     Path(token): Path<String>,
// ) -> Result<Json<ValidateTokenResponse>, AppError> {
//     let row = sqlx::query!(
//         "select used, expires_at from one_time_tokens where token = $1",
//         token
//     )
//     .fetch_optional(&state.pool)
//     .await?
//     .ok_or(AppError::TokenNotFound)?;

//     if row.used {
//         return Err(AppError::TokenAlreadyUsed);
//     }
//     if row.expires_at < Utc::now() {
//         return Err(AppError::TokenExpired);
//     }

//     Ok(Json(ValidateTokenResponse { valid: true }))
// }

// // ============== Submissions ==============

// /// POST /api/submissions
// /// Employee: req.token = Some(one_time_token), без заголовка Authorization
// /// Inspector/user: req.token = None, Authorization: Bearer <jwt>,
// /// нужен permission "submissions.create_as_inspector"
// pub async fn create_submission(
//     State(state): State<AppState>,
//     OptionalClaims(maybe_claims): OptionalClaims,
//     Json(req): Json<CreateSubmissionRequest>,
// ) -> Result<Json<SubmissionResponse>, AppError> {
//     match (&req.token, &maybe_claims) {
//         (Some(token), _) => create_from_employee_token(&state, token, req).await,
//         (None, Some(claims)) => create_from_authorized_user(&state, claims, req).await,
//         (None, None) => Err(AppError::BadRequest(
//             "either token or a valid session is required".into(),
//         )),
//     }
// }

// async fn create_from_employee_token(
//     state: &AppState,
//     token: &str,
//     req: CreateSubmissionRequest,
// ) -> Result<Json<SubmissionResponse>, AppError> {
//     let mut tx = state.pool.begin().await?;

//     // for update — лочим строку токена против параллельного повторного сабмита
//     let token_row = sqlx::query!(
//         "select telegram_user_id, used, expires_at from one_time_tokens where token = $1 for update",
//         token
//     )
//     .fetch_optional(&mut *tx)
//     .await?
//     .ok_or(AppError::TokenNotFound)?;

//     if token_row.used {
//         return Err(AppError::TokenAlreadyUsed);
//     }
//     if token_row.expires_at < Utc::now() {
//         return Err(AppError::TokenExpired);
//     }

//     let row = sqlx::query!(
//         r#"
//         insert into submissions
//             (type, department_id, category_id, text, file_url, channel, telegram_user_id)
//         values ('EMPLOYEE', $1, $2, $3, $4, $5, $6)
//         returning id, type as "kind: SubmissionType", department_id, category_id, text,
//                   file_url, status as "status: SubmissionStatus", reject_reason, channel,
//                   telegram_user_id, created_by_user, created_at, updated_at
//         "#,
//         req.department_id,
//         req.category_id,
//         req.text,
//         req.file_url,
//         req.channel,
//         token_row.telegram_user_id,
//     )
//     .fetch_one(&mut *tx)
//     .await?;

//     sqlx::query!(
//         "update one_time_tokens set used = true where token = $1",
//         token
//     )
//     .execute(&mut *tx)
//     .await?;

//     tx.commit().await?;

//     Ok(Json(SubmissionResponse {
//         id: row.id,
//         kind: row.kind,
//         department_id: row.department_id,
//         category_id: row.category_id,
//         text: row.text,
//         file_url: row.file_url,
//         status: row.status,
//         reject_reason: row.reject_reason,
//         channel: row.channel,
//         telegram_user_id: row.telegram_user_id,
//         created_by_user: row.created_by_user,
//         created_at: row.created_at,
//         updated_at: row.updated_at,
//     }))
// }

// /// Любой авторизованный пользователь с правом submissions.create_as_inspector
// /// (обычно это роль "inspector", но не обязательно — право проверяется явно,
// /// не роль по имени).
// async fn create_from_authorized_user(
//     state: &AppState,
//     claims: &Claims,
//     req: CreateSubmissionRequest,
// ) -> Result<Json<SubmissionResponse>, AppError> {
//     RequirePermission::check(state, claims, "submissions.create_as_inspector").await?;

//     let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

//     let row = sqlx::query!(
//         r#"
//         insert into submissions
//             (type, department_id, category_id, text, file_url, channel, created_by_user)
//         values ('INSPECTOR', $1, $2, $3, $4, $5, $6)
//         returning id, type as "kind: SubmissionType", department_id, category_id, text,
//                   file_url, status as "status: SubmissionStatus", reject_reason, channel,
//                   telegram_user_id, created_by_user, created_at, updated_at
//         "#,
//         req.department_id,
//         req.category_id,
//         req.text,
//         req.file_url,
//         req.channel,
//         user_id,
//     )
//     .fetch_one(&state.pool)
//     .await?;

//     Ok(Json(SubmissionResponse {
//         id: row.id,
//         kind: row.kind,
//         department_id: row.department_id,
//         category_id: row.category_id,
//         text: row.text,
//         file_url: row.file_url,
//         status: row.status,
//         reject_reason: row.reject_reason,
//         channel: row.channel,
//         telegram_user_id: row.telegram_user_id,
//         created_by_user: row.created_by_user,
//         created_at: row.created_at,
//         updated_at: row.updated_at,
//     }))
// }

// /// GET /api/admin/submissions
// pub async fn list_submissions(
//     State(state): State<AppState>,
//     RequireUser(claims): RequireUser,
//     Query(query): Query<ListSubmissionsQuery>,
// ) -> Result<Json<Vec<SubmissionResponse>>, AppError> {
//     RequirePermission::check(&state, &claims, "submissions.review").await?;

//     let page = query.page.unwrap_or(1).max(1);
//     let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
//     let offset = (page - 1) * per_page;

//     let rows = sqlx::query!(
//         r#"
//         select id, type as "kind: SubmissionType", department_id, category_id, text,
//                file_url, status as "status: SubmissionStatus", reject_reason, channel,
//                telegram_user_id, created_by_user, created_at, updated_at
//         from submissions
//         where ($1::submission_status is null or status = $1)
//           and ($2::bigint is null or department_id = $2)
//           and ($3::bigint is null or category_id = $3)
//         order by created_at desc
//         limit $4 offset $5
//         "#,
//         query.status as Option<SubmissionStatus>,
//         query.department_id,
//         query.category_id,
//         per_page,
//         offset,
//     )
//     .fetch_all(&state.pool)
//     .await?;

//     Ok(Json(
//         rows.into_iter()
//             .map(|r| SubmissionResponse {
//                 id: r.id,
//                 kind: r.kind,
//                 department_id: r.department_id,
//                 category_id: r.category_id,
//                 text: r.text,
//                 file_url: r.file_url,
//                 status: r.status,
//                 reject_reason: r.reject_reason,
//                 channel: r.channel,
//                 telegram_user_id: r.telegram_user_id,
//                 created_by_user: r.created_by_user,
//                 created_at: r.created_at,
//                 updated_at: r.updated_at,
//             })
//             .collect(),
//     ))
// }

// /// PATCH /api/admin/submissions/{id}/status
// pub async fn update_submission_status(
//     State(state): State<AppState>,
//     RequireUser(claims): RequireUser,
//     Path(id): Path<i64>,
//     Json(req): Json<UpdateStatusRequest>,
// ) -> Result<Json<SubmissionResponse>, AppError> {
//     RequirePermission::check(&state, &claims, "submissions.review").await?;

//     if req.status == SubmissionStatus::Rejected && req.reject_reason.is_none() {
//         return Err(AppError::BadRequest(
//             "reject_reason is required when rejecting a submission".into(),
//         ));
//     }

//     let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

//     let mut tx = state.pool.begin().await?;

//     let old_row = sqlx::query!(
//         r#"select status as "status: SubmissionStatus" from submissions where id = $1"#,
//         id
//     )
//     .fetch_optional(&mut *tx)
//     .await?
//     .ok_or(AppError::SubmissionNotFound)?;

//     let row = sqlx::query!(
//         r#"
//         update submissions
//         set status = $1, reject_reason = $2, updated_at = now()
//         where id = $3
//         returning id, type as "kind: SubmissionType", department_id, category_id, text,
//                   file_url, status as "status: SubmissionStatus", reject_reason, channel,
//                   telegram_user_id, created_by_user, created_at, updated_at
//         "#,
//         req.status as SubmissionStatus,
//         req.reject_reason,
//         id,
//     )
//     .fetch_one(&mut *tx)
//     .await?;

//     sqlx::query!(
//         r#"
//         insert into submission_status_history
//             (submission_id, old_status, new_status, changed_by, comment)
//         values ($1, $2, $3, $4, $5)
//         "#,
//         id,
//         old_row.status as SubmissionStatus,
//         req.status as SubmissionStatus,
//         user_id,
//         req.comment,
//     )
//     .execute(&mut *tx)
//     .await?;

//     tx.commit().await?;

//     Ok(Json(SubmissionResponse {
//         id: row.id,
//         kind: row.kind,
//         department_id: row.department_id,
//         category_id: row.category_id,
//         text: row.text,
//         file_url: row.file_url,
//         status: row.status,
//         reject_reason: row.reject_reason,
//         channel: row.channel,
//         telegram_user_id: row.telegram_user_id,
//         created_by_user: row.created_by_user,
//         created_at: row.created_at,
//         updated_at: row.updated_at,
//     }))
// }

// // ============== Inspector auth (теперь это просто users с tabel_number) ==============

// /// POST /api/inspector/auth — вход по табельному номеру, без пароля
// pub async fn inspector_auth(
//     State(state): State<AppState>,
//     Json(req): Json<InspectorAuthRequest>,
// ) -> Result<Json<InspectorAuthResponse>, AppError> {
//     let user = sqlx::query!(
//         "select id, name_ru from users where tabel_number = $1 and is_active = true",
//         req.tabel_number
//     )
//     .fetch_optional(&state.pool)
//     .await?
//     .ok_or(AppError::InvalidTabelNumber)?;

//     let session_id = Uuid::new_v4();
//     let refresh_token = generate_refresh_token();
//     let refresh_token_hash = hash_refresh_token(&refresh_token);
//     let expires_at = Utc::now() + Duration::hours(12);

//     sqlx::query!(
//         r#"
//         insert into sessions (id, user_id, refresh_token_hash, expires_at)
//         values ($1, $2, $3, $4)
//         "#,
//         session_id,
//         user.id,
//         refresh_token_hash,
//         expires_at,
//     )
//     .execute(&state.pool)
//     .await?;

//     let token = create_access_token(user.id, session_id, state.jwt_secret.as_bytes())
//         .map_err(|_| AppError::Unauthorized)?;

//     Ok(Json(InspectorAuthResponse {
//         token,
//         refresh_token,
//         user_id: user.id,
//         name_ru: user.name_ru.unwrap_or_default(),
//     }))
// }

// // ============== RBAC auth (email/password users) ==============

// /// POST /api/auth/login
// pub async fn login(
//     State(state): State<AppState>,
//     Json(req): Json<LoginRequest>,
// ) -> Result<Json<LoginResponse>, AppError> {
//     let user = sqlx::query!(
//         "select id, password_hash from users where email = $1 and is_active = true",
//         req.email
//     )
//     .fetch_optional(&state.pool)
//     .await?
//     .ok_or(AppError::InvalidCredentials)?;

//     // password_hash nullable (инспекторы без пароля) — но через /auth/login
//     // инспектор не логинится, так что None здесь = неправильный флоу.
//     let password_hash = user.password_hash.ok_or(AppError::InvalidCredentials)?;
//     let parsed_hash = PasswordHash::new(&password_hash).map_err(|_| AppError::PasswordHash)?;

//     Argon2::default()
//         .verify_password(req.password.as_bytes(), &parsed_hash)
//         .map_err(|_| AppError::InvalidCredentials)?;

//     let session_id = Uuid::new_v4();
//     let refresh_token = generate_refresh_token();
//     let refresh_token_hash = hash_refresh_token(&refresh_token);
//     let expires_at = Utc::now() + Duration::days(30);

//     sqlx::query!(
//         "insert into sessions (id, user_id, refresh_token_hash, expires_at) values ($1, $2, $3, $4)",
//         session_id,
//         user.id,
//         refresh_token_hash,
//         expires_at,
//     )
//     .execute(&state.pool)
//     .await?;

//     let access_token = create_access_token(user.id, session_id, state.jwt_secret.as_bytes())
//         .map_err(|_| AppError::Unauthorized)?;

//     Ok(Json(LoginResponse {
//         access_token,
//         refresh_token,
//         user_id: user.id,
//     }))
// }

// /// POST /api/auth/refresh
// pub async fn refresh(
//     State(state): State<AppState>,
//     Json(req): Json<RefreshRequest>,
// ) -> Result<Json<LoginResponse>, AppError> {
//     let incoming_hash = hash_refresh_token(&req.refresh_token);

//     let session = sqlx::query!(
//         "select id, user_id, revoked, expires_at from sessions where refresh_token_hash = $1",
//         incoming_hash
//     )
//     .fetch_optional(&state.pool)
//     .await?
//     .ok_or(AppError::Unauthorized)?;

//     // Reuse detection: использование уже отозванного refresh — признак
//     // компрометации, реакция — убить все сессии пользователя.
//     if session.revoked {
//         sqlx::query!(
//             "update sessions set revoked = true where user_id = $1",
//             session.user_id
//         )
//         .execute(&state.pool)
//         .await?;
//         return Err(AppError::Unauthorized);
//     }

//     if session.expires_at < Utc::now() {
//         return Err(AppError::Unauthorized);
//     }

//     let new_refresh_token = generate_refresh_token();
//     let new_refresh_hash = hash_refresh_token(&new_refresh_token);
//     let new_expires_at = Utc::now() + Duration::days(30);

//     sqlx::query!(
//         "update sessions set refresh_token_hash = $1, expires_at = $2 where id = $3",
//         new_refresh_hash,
//         new_expires_at,
//         session.id,
//     )
//     .execute(&state.pool)
//     .await?;

//     let access_token =
//         create_access_token(session.user_id, session.id, state.jwt_secret.as_bytes())
//             .map_err(|_| AppError::Unauthorized)?;

//     Ok(Json(LoginResponse {
//         access_token,
//         refresh_token: new_refresh_token,
//         user_id: session.user_id,
//     }))
// }

// /// POST /api/auth/logout
// pub async fn logout(
//     State(state): State<AppState>,
//     RequireUser(claims): RequireUser,
// ) -> Result<axum::http::StatusCode, AppError> {
//     let session_id: Uuid = claims.sid.parse().map_err(|_| AppError::Unauthorized)?;
//     sqlx::query!(
//         "update sessions set revoked = true where id = $1",
//         session_id
//     )
//     .execute(&state.pool)
//     .await?;
//     Ok(axum::http::StatusCode::NO_CONTENT)
// }

// /// POST /api/auth/logout_all
// pub async fn logout_all(
//     State(state): State<AppState>,
//     RequireUser(claims): RequireUser,
// ) -> Result<axum::http::StatusCode, AppError> {
//     RequirePermission::check(&state, &claims, "auth.logout_all").await?;
//     let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;
//     sqlx::query!(
//         "update sessions set revoked = true where user_id = $1",
//         user_id
//     )
//     .execute(&state.pool)
//     .await?;
//     Ok(axum::http::StatusCode::NO_CONTENT)
// }

// /// GET /api/auth/me
// pub async fn me(
//     State(state): State<AppState>,
//     RequireUser(claims): RequireUser,
// ) -> Result<Json<MeResponse>, AppError> {
//     let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

//     let user = sqlx::query!(
//         "select id, email, username from users where id = $1",
//         user_id
//     )
//     .fetch_optional(&state.pool)
//     .await?
//     .ok_or(AppError::Unauthorized)?;

//     let role_rows = sqlx::query!(
//         r#"
//         select r.code
//         from roles r
//         join user_roles ur on ur.role_id = r.id
//         where ur.user_id = $1
//         "#,
//         user_id
//     )
//     .fetch_all(&state.pool)
//     .await?;

//     Ok(Json(MeResponse {
//         id: user.id,
//         email: user.email.unwrap_or_default(),
//         username: user.username.unwrap_or_default(),
//         roles: role_rows.into_iter().map(|r| r.code).collect(),
//     }))
// }

// // ============== Helpers ==============

// fn generate_refresh_token() -> String {
//     rand::rng()
//         .sample_iter(&Alphanumeric)
//         .take(64)
//         .map(char::from)
//         .collect()
// }

// fn hash_refresh_token(token: &str) -> String {
//     let mut hasher = Sha256::new();
//     hasher.update(token.as_bytes());
//     hex::encode(hasher.finalize())
// }
use crate::AppState;
use crate::auth::jwt::{Claims, create_access_token};
use crate::error::AppError;
use crate::extractors::{OptionalClaims, RequirePermission, RequireUser};
use crate::models::{
    CreateSubmissionRequest, GenerateTokenRequest, GenerateTokenResponse, InspectorAuthRequest,
    InspectorAuthResponse, ListSubmissionsQuery, LoginRequest, LoginResponse, MeResponse,
    RefreshRequest, SubmissionResponse, SubmissionStatus, SubmissionType, UpdateStatusRequest,
    ValidateTokenResponse,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{Duration, Utc};
// use rand::RngExt;
use rand::distr::Alphanumeric;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use rand::Rng;
// ============== One-time tokens (employee flow entry point) ==============

/// POST /api/token/generate — вызывается ботом при нажатии "Обращение"
pub async fn generate_token(
    State(state): State<AppState>,
    Json(req): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, AppError> {
    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let expires_at = Utc::now() + Duration::hours(24);

    sqlx::query!(
        "insert into one_time_tokens (token, telegram_user_id, expires_at) values ($1, $2, $3)",
        token,
        req.telegram_user_id,
        expires_at,
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(GenerateTokenResponse {
        url: format!("{}/form?token={}", state.public_base_url, token),
        token,
    }))
}

/// GET /api/token/validate/{token} — фронт дёргает при открытии формы
pub async fn validate_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<ValidateTokenResponse>, AppError> {
    let row = sqlx::query!(
        "select used, expires_at from one_time_tokens where token = $1",
        token
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::TokenNotFound)?;

    if row.used {
        return Err(AppError::TokenAlreadyUsed);
    }
    if row.expires_at < Utc::now() {
        return Err(AppError::TokenExpired);
    }

    Ok(Json(ValidateTokenResponse { valid: true }))
}

// ============== Submissions ==============

/// POST /api/submissions
/// Employee: req.token = Some(one_time_token), без заголовка Authorization
/// Inspector/user: req.token = None, Authorization: Bearer <jwt>,
/// нужен permission "submissions.create_as_inspector"
pub async fn create_submission(
    State(state): State<AppState>,
    OptionalClaims(maybe_claims): OptionalClaims,
    Json(req): Json<CreateSubmissionRequest>,
) -> Result<Json<SubmissionResponse>, AppError> {
    match (&req.token.clone(), &maybe_claims) {
        (Some(token), _) => create_from_employee_token(&state, token, req).await,
        (None, Some(claims)) => create_from_authorized_user(&state, claims, req).await,
        (None, None) => Err(AppError::BadRequest(
            "either token or a valid session is required".into(),
        )),
    }
}

async fn create_from_employee_token(
    state: &AppState,
    token: &str,
    req: CreateSubmissionRequest,
) -> Result<Json<SubmissionResponse>, AppError> {
    let mut tx = state.pool.begin().await?;

    // for update — лочим строку токена против параллельного повторного сабмита
    let token_row = sqlx::query!(
        "select telegram_user_id, used, expires_at from one_time_tokens where token = $1 for update",
        token
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::TokenNotFound)?;

    if token_row.used {
        return Err(AppError::TokenAlreadyUsed);
    }
    if token_row.expires_at < Utc::now() {
        return Err(AppError::TokenExpired);
    }

    let row = sqlx::query!(
        r#"
        insert into submissions
            (type, department_id, category_id, text, file_url, channel, telegram_user_id)
        values ('EMPLOYEE', $1, $2, $3, $4, $5, $6)
        returning id, type as "kind: SubmissionType", department_id, category_id, text,
                  file_url, status as "status: SubmissionStatus", reject_reason, channel,
                  telegram_user_id, created_by_user, created_at, updated_at
        "#,
        req.department_id,
        req.category_id,
        req.text,
        req.file_url,
        req.channel,
        token_row.telegram_user_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!("update one_time_tokens set used = true where token = $1", token)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(SubmissionResponse {
        id: row.id,
        kind: row.kind,
        department_id: row.department_id,
        category_id: row.category_id,
        text: row.text,
        file_url: row.file_url,
        status: row.status,
        reject_reason: row.reject_reason,
        channel: row.channel,
        telegram_user_id: row.telegram_user_id,
        created_by_user: row.created_by_user,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

/// Любой авторизованный пользователь с правом submissions.create_as_inspector
/// (обычно это роль "inspector", но не обязательно — право проверяется явно,
/// не роль по имени).
async fn create_from_authorized_user(
    state: &AppState,
    claims: &Claims,
    req: CreateSubmissionRequest,
) -> Result<Json<SubmissionResponse>, AppError> {
    RequirePermission::check(state, claims, "submissions.create_as_inspector").await?;

    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    let row = sqlx::query!(
        r#"
        insert into submissions
            (type, department_id, category_id, text, file_url, channel, created_by_user)
        values ('INSPECTOR', $1, $2, $3, $4, $5, $6)
        returning id, type as "kind: SubmissionType", department_id, category_id, text,
                  file_url, status as "status: SubmissionStatus", reject_reason, channel,
                  telegram_user_id, created_by_user, created_at, updated_at
        "#,
        req.department_id,
        req.category_id,
        req.text,
        req.file_url,
        req.channel,
        user_id,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(SubmissionResponse {
        id: row.id,
        kind: row.kind,
        department_id: row.department_id,
        category_id: row.category_id,
        text: row.text,
        file_url: row.file_url,
        status: row.status,
        reject_reason: row.reject_reason,
        channel: row.channel,
        telegram_user_id: row.telegram_user_id,
        created_by_user: row.created_by_user,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

/// GET /api/admin/submissions
pub async fn list_submissions(
    State(state): State<AppState>,
    RequireUser(claims): RequireUser,
    Query(query): Query<ListSubmissionsQuery>,
) -> Result<Json<Vec<SubmissionResponse>>, AppError> {
    RequirePermission::check(&state, &claims, "submissions.review").await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query!(
        r#"
        select id, type as "kind: SubmissionType", department_id, category_id, text,
               file_url, status as "status: SubmissionStatus", reject_reason, channel,
               telegram_user_id, created_by_user, created_at, updated_at
        from submissions
        where ($1::submission_status is null or status = $1)
          and ($2::bigint is null or department_id = $2)
          and ($3::bigint is null or category_id = $3)
        order by created_at desc
        limit $4 offset $5
        "#,
        query.status as Option<SubmissionStatus>,
        query.department_id,
        query.category_id,
        per_page,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| SubmissionResponse {
                id: r.id,
                kind: r.kind,
                department_id: r.department_id,
                category_id: r.category_id,
                text: r.text,
                file_url: r.file_url,
                status: r.status,
                reject_reason: r.reject_reason,
                channel: r.channel,
                telegram_user_id: r.telegram_user_id,
                created_by_user: r.created_by_user,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect(),
    ))
}

/// PATCH /api/admin/submissions/{id}/status
pub async fn update_submission_status(
    State(state): State<AppState>,
    RequireUser(claims): RequireUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<Json<SubmissionResponse>, AppError> {
    RequirePermission::check(&state, &claims, "submissions.review").await?;

    if req.status == SubmissionStatus::Rejected && req.reject_reason.is_none() {
        return Err(AppError::BadRequest(
            "reject_reason is required when rejecting a submission".into(),
        ));
    }

    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    let mut tx = state.pool.begin().await?;

    let old_row = sqlx::query!(
        r#"select status as "status: SubmissionStatus" from submissions where id = $1"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::SubmissionNotFound)?;

    let row = sqlx::query!(
        r#"
        update submissions
        set status = $1, reject_reason = $2, updated_at = now()
        where id = $3
        returning id, type as "kind: SubmissionType", department_id, category_id, text,
                  file_url, status as "status: SubmissionStatus", reject_reason, channel,
                  telegram_user_id, created_by_user, created_at, updated_at
        "#,
        req.status as SubmissionStatus,
        req.reject_reason,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        insert into submission_status_history
            (submission_id, old_status, new_status, changed_by, comment)
        values ($1, $2, $3, $4, $5)
        "#,
        id,
        old_row.status as SubmissionStatus,
        req.status as SubmissionStatus,
        user_id,
        req.comment,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(SubmissionResponse {
        id: row.id,
        kind: row.kind,
        department_id: row.department_id,
        category_id: row.category_id,
        text: row.text,
        file_url: row.file_url,
        status: row.status,
        reject_reason: row.reject_reason,
        channel: row.channel,
        telegram_user_id: row.telegram_user_id,
        created_by_user: row.created_by_user,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

// ============== Inspector auth (теперь это просто users с tabel_number) ==============

/// POST /api/inspector/auth — вход по табельному номеру, без пароля
pub async fn inspector_auth(
    State(state): State<AppState>,
    Json(req): Json<InspectorAuthRequest>,
) -> Result<Json<InspectorAuthResponse>, AppError> {
    let user = sqlx::query!(
        "select id, name_ru from users where tabel_number = $1 and is_active = true",
        req.tabel_number
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::InvalidTabelNumber)?;

    let session_id = Uuid::new_v4();
    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::hours(12);

    sqlx::query!(
        r#"
        insert into sessions (id, user_id, refresh_token_hash, expires_at)
        values ($1, $2, $3, $4)
        "#,
        session_id,
        user.id,
        refresh_token_hash,
        expires_at,
    )
    .execute(&state.pool)
    .await?;

    let token = create_access_token(user.id, session_id, state.jwt_secret.as_bytes())
        .map_err(|_| AppError::Unauthorized)?;

    Ok(Json(InspectorAuthResponse {
        token,
        refresh_token,
        user_id: user.id,
        name_ru: user.name_ru.unwrap_or_default(),
    }))
}

// ============== RBAC auth (email/password users) ==============

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query!(
        "select id, password_hash from users where email = $1 and is_active = true",
        req.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::InvalidCredentials)?;

    // password_hash nullable (инспекторы без пароля) — но через /auth/login
    // инспектор не логинится, так что None здесь = неправильный флоу.
    let password_hash = user.password_hash.ok_or(AppError::InvalidCredentials)?;
    let parsed_hash = PasswordHash::new(&password_hash).map_err(|_| AppError::PasswordHash)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    let session_id = Uuid::new_v4();
    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::days(30);

    sqlx::query!(
        "insert into sessions (id, user_id, refresh_token_hash, expires_at) values ($1, $2, $3, $4)",
        session_id,
        user.id,
        refresh_token_hash,
        expires_at,
    )
    .execute(&state.pool)
    .await?;

    let access_token = create_access_token(user.id, session_id, state.jwt_secret.as_bytes())
        .map_err(|_| AppError::Unauthorized)?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.id,
    }))
}

/// POST /api/auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let incoming_hash = hash_refresh_token(&req.refresh_token);

    let session = sqlx::query!(
        "select id, user_id, revoked, expires_at from sessions where refresh_token_hash = $1",
        incoming_hash
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    // Reuse detection: использование уже отозванного refresh — признак
    // компрометации, реакция — убить все сессии пользователя.
    if session.revoked {
        sqlx::query!("update sessions set revoked = true where user_id = $1", session.user_id)
            .execute(&state.pool)
            .await?;
        return Err(AppError::Unauthorized);
    }

    if session.expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    let new_refresh_token = generate_refresh_token();
    let new_refresh_hash = hash_refresh_token(&new_refresh_token);
    let new_expires_at = Utc::now() + Duration::days(30);

    sqlx::query!(
        "update sessions set refresh_token_hash = $1, expires_at = $2 where id = $3",
        new_refresh_hash,
        new_expires_at,
        session.id,
    )
    .execute(&state.pool)
    .await?;

    let access_token = create_access_token(session.user_id, session.id, state.jwt_secret.as_bytes())
        .map_err(|_| AppError::Unauthorized)?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token: new_refresh_token,
        user_id: session.user_id,
    }))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    RequireUser(claims): RequireUser,
) -> Result<axum::http::StatusCode, AppError> {
    let session_id: Uuid = claims.sid.parse().map_err(|_| AppError::Unauthorized)?;
    sqlx::query!("update sessions set revoked = true where id = $1", session_id)
        .execute(&state.pool)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// POST /api/auth/logout_all
pub async fn logout_all(
    State(state): State<AppState>,
    RequireUser(claims): RequireUser,
) -> Result<axum::http::StatusCode, AppError> {
    RequirePermission::check(&state, &claims, "auth.logout_all").await?;
    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;
    sqlx::query!("update sessions set revoked = true where user_id = $1", user_id)
        .execute(&state.pool)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// GET /api/auth/me
pub async fn me(
    State(state): State<AppState>,
    RequireUser(claims): RequireUser,
) -> Result<Json<MeResponse>, AppError> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    let user = sqlx::query!("select id, email, username from users where id = $1", user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let role_rows = sqlx::query!(
        r#"
        select r.code
        from roles r
        join user_roles ur on ur.role_id = r.id
        where ur.user_id = $1
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(MeResponse {
        id: user.id,
        email: user.email.unwrap_or_default(),
        username: user.username.unwrap_or_default(),
        roles: role_rows.into_iter().map(|r| r.code).collect(),
    }))
}

// ============== Helpers ==============

fn generate_refresh_token() -> String {
    rand::rng()
    .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
