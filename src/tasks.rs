use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::current_user;
use crate::{stamp, ApiError, AppState};

#[derive(Clone, Serialize)]
pub struct CachedMyTasks {
    pub tasks: Vec<MyTask>,
}

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct MyTask {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: String,
}

#[derive(Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: String,
    pub priority: String,
}

#[derive(Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    title: String,
    description: String,
    status: String,
    priority: String,
    assigned_to_id: Option<String>,
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let user = current_user(state, headers)?;
    if user.role != "admin" {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

fn priority_ok(priority: &str) -> bool {
    matches!(priority, "low" | "medium" | "high")
}

fn status_ok(status: &str) -> bool {
    matches!(status, "todo" | "in_progress" | "done")
}

pub async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTask>,
) -> Result<(StatusCode, Json<TaskResponse>), ApiError> {
    let user = current_user(&state, &headers)?;
    if user.role != "admin" {
        return Err(ApiError::Forbidden);
    }
    if body.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title is required"));
    }
    if !priority_ok(&body.priority) {
        return Err(ApiError::BadRequest("unknown priority"));
    }

    let now = stamp(state.clock.now());
    let id = Uuid::new_v4().to_string();
    let title = body.title.trim().to_string();
    sqlx::query(
        "INSERT INTO tasks (
            id, title, description, status, priority, created_by_id, assigned_to_id, created_at, updated_at
         ) VALUES (?1, ?2, ?3, 'todo', ?4, ?5, NULL, ?6, ?7)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&body.description)
    .bind(&body.priority)
    .bind(&user.sub)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(TaskResponse {
            id,
            title,
            description: body.description,
            status: "todo".to_string(),
            priority: body.priority,
            assigned_to: None,
        }),
    ))
}

pub async fn update_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateTask>,
) -> Result<Json<TaskResponse>, ApiError> {
    require_admin(&state, &headers)?;
    if let Some(title) = &body.title {
        if title.trim().is_empty() {
            return Err(ApiError::BadRequest("title is required"));
        }
    }
    if body
        .status
        .as_deref()
        .is_some_and(|status| !status_ok(status))
    {
        return Err(ApiError::BadRequest("unknown status"));
    }
    if body
        .priority
        .as_deref()
        .is_some_and(|priority| !priority_ok(priority))
    {
        return Err(ApiError::BadRequest("unknown priority"));
    }

    let existing = sqlx::query_as::<_, TaskRow>(
        "SELECT id, title, description, status, priority, assigned_to_id FROM tasks WHERE id = ?1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(existing) = existing else {
        return Err(ApiError::NotFound);
    };

    let title = body
        .title
        .map(|title| title.trim().to_string())
        .unwrap_or(existing.title);
    let description = body.description.unwrap_or(existing.description);
    let status = body.status.unwrap_or(existing.status);
    let priority = body.priority.unwrap_or(existing.priority);
    let now = stamp(state.clock.now());
    sqlx::query(
        "UPDATE tasks
         SET title = ?1, description = ?2, status = ?3, priority = ?4, updated_at = ?5
         WHERE id = ?6",
    )
    .bind(&title)
    .bind(&description)
    .bind(&status)
    .bind(&priority)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    if let Some(assignee_id) = &existing.assigned_to_id {
        state.cache.lock().expect("cache lock").remove(assignee_id);
    }

    Ok(Json(TaskResponse {
        id: existing.id,
        title,
        description,
        status,
        priority,
        assigned_to: assignee_email(&state, existing.assigned_to_id).await?,
    }))
}

async fn assignee_email(
    state: &AppState,
    assignee_id: Option<String>,
) -> Result<Option<String>, ApiError> {
    let Some(assignee_id) = assignee_id else {
        return Ok(None);
    };
    let email: Option<String> = sqlx::query_scalar("SELECT email FROM users WHERE id = ?1")
        .bind(assignee_id)
        .fetch_optional(&state.pool)
        .await?;
    Ok(email)
}

#[derive(Deserialize)]
pub struct AssignRequest {
    pub task_ids: Vec<String>,
    pub assignee_email: String,
}

#[derive(Serialize)]
pub struct AssignResponse {
    pub assigned: usize,
}

pub async fn assign_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AssignRequest>,
) -> Result<Json<AssignResponse>, ApiError> {
    require_admin(&state, &headers)?;
    if body.task_ids.is_empty() {
        return Err(ApiError::BadRequest("task_ids is required"));
    }

    let assignee_id: Option<String> = sqlx::query_scalar("SELECT id FROM users WHERE email = ?1")
        .bind(&body.assignee_email)
        .fetch_optional(&state.pool)
        .await?;
    let Some(assignee_id) = assignee_id else {
        return Err(ApiError::NotFound);
    };

    let mut tx = state.pool.begin().await?;
    let mut previous_assignees = Vec::with_capacity(body.task_ids.len());
    for task_id in &body.task_ids {
        let assigned_to: Option<Option<String>> =
            sqlx::query_scalar("SELECT assigned_to_id FROM tasks WHERE id = ?1")
                .bind(task_id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(assigned_to) = assigned_to else {
            return Err(ApiError::NotFound);
        };
        previous_assignees.push(assigned_to);
    }

    let now = stamp(state.clock.now());
    for task_id in &body.task_ids {
        sqlx::query("UPDATE tasks SET assigned_to_id = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(&assignee_id)
            .bind(&now)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    let mut cache = state.cache.lock().expect("cache lock");
    cache.remove(&assignee_id);
    for previous in previous_assignees.into_iter().flatten() {
        cache.remove(&previous);
    }

    Ok(Json(AssignResponse {
        assigned: body.task_ids.len(),
    }))
}

#[derive(Serialize)]
pub struct MyTasksUser {
    pub email: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct MyTasksSummary {
    pub total_assigned_tasks: usize,
}

#[derive(Serialize)]
pub struct CacheMeta {
    pub hit: bool,
}

#[derive(Serialize)]
pub struct MyTasksResponse {
    pub user: MyTasksUser,
    pub tasks: Vec<MyTask>,
    pub summary: MyTasksSummary,
    pub cache: CacheMeta,
}

pub async fn view_my_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MyTasksResponse>, ApiError> {
    let user = current_user(&state, &headers)?;
    if let Some(cached) = state
        .cache
        .lock()
        .expect("cache lock")
        .get(&user.sub)
        .cloned()
    {
        return Ok(Json(response_for(user, cached.tasks, true)));
    }

    let tasks = sqlx::query_as::<_, MyTask>(
        "SELECT t.id, t.title, t.status, t.priority, u.email AS assigned_to
         FROM tasks t
         JOIN users u ON u.id = t.assigned_to_id
         WHERE t.assigned_to_id = ?1
         ORDER BY t.created_at ASC",
    )
    .bind(&user.sub)
    .fetch_all(&state.pool)
    .await?;

    state.cache.lock().expect("cache lock").insert(
        user.sub.clone(),
        CachedMyTasks {
            tasks: tasks.clone(),
        },
    );
    Ok(Json(response_for(user, tasks, false)))
}

fn response_for(user: crate::identity::Claims, tasks: Vec<MyTask>, hit: bool) -> MyTasksResponse {
    MyTasksResponse {
        summary: MyTasksSummary {
            total_assigned_tasks: tasks.len(),
        },
        user: MyTasksUser {
            email: user.email,
            role: user.role,
        },
        tasks,
        cache: CacheMeta { hit },
    }
}
