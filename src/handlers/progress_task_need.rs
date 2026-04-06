use axum::{extract::{Path, State, Json, Extension}, response::{Html, IntoResponse, Redirect}, Form};
use sqlx::postgres::PgRow;
use sqlx::Row;
use crate::models::progress_task_need::*;
use crate::handlers::AppState;
use crate::auth::models::User;
use crate::handlers::pages::{get_session_from_cookies, get_user_info_from_session, UserInfo, has_project_edit_permission, log_audit_action};
use askama::Template;

// 进度更新表单
#[derive(Template)]
#[template(path = "progress_form.html")]
struct ProgressFormTemplate {
    project_id: i64,
    project_title: String,
    user: Option<UserInfo>,
    user_exists: bool,
}

// Progress handlers
#[axum::debug_handler]
pub async fn create_progress_form(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    // 获取项目信息
    let project = sqlx::query!(r#"
        SELECT title
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap();
    
    match project {
        Some(project) => {
            let template = ProgressFormTemplate {
                project_id,
                project_title: project.title,
                user: user_info,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        None => {
            // 项目不存在，重定向到首页
            axum::response::Redirect::to("/").into_response()
        }
    }
}

#[axum::debug_handler]
pub async fn create_progress(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: axum::http::HeaderMap,
    Form(CreateProgress { content, progress_percentage, update_date }): Form<CreateProgress>,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    let user_id = user_info.as_ref().unwrap().id;
    
    // 检查用户是否有项目编辑权限
    if !has_project_edit_permission(&state.pool, project_id, user_id).await {
        return axum::response::Redirect::to(&format!("/projects/{}", project_id)).into_response();
    }
    
    // Create progress update
    sqlx::query(
        "INSERT INTO project_progress (project_id, user_id, content, progress_percentage, update_date, created_at) VALUES ($1, $2, $3, $4, $5, now())"
    )
    .bind(project_id)
    .bind(user_id)
    .bind(content)
    .bind(progress_percentage)
    .bind(update_date)
    .execute(&state.pool)
    .await
    .unwrap();
    
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

#[axum::debug_handler]
pub async fn get_project_progress(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Json<Vec<ProjectProgress>> {
    let progress: Vec<ProjectProgress> = sqlx::query_as!(ProjectProgress, 
        "SELECT id, project_id, user_id, content, created_at FROM project_progress WHERE project_id = $1 ORDER BY created_at DESC",
        project_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    
    Json(progress)
}



// 需求表单
#[derive(Template)]
#[template(path = "need_form.html")]
struct NeedFormTemplate {
    project_id: i64,
    user: Option<UserInfo>,
    user_exists: bool,
}

// Need form handler
#[axum::debug_handler]
pub async fn create_need_form(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    // 获取项目信息
    let project = sqlx::query!(r#"
        SELECT title
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap();
    
    match project {
        Some(_) => {
            let template = NeedFormTemplate {
                project_id,
                user: user_info,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        None => {
            // 项目不存在，重定向到首页
            axum::response::Redirect::to("/").into_response()
        }
    }
}

// Need handlers
#[axum::debug_handler]
pub async fn create_need(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: axum::http::HeaderMap,
    Form(CreateNeed { title, description, priority }): Form<CreateNeed>,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    let user_id = user_info.as_ref().unwrap().id;
    
    // 开始事务
    let mut tx = state.pool.begin().await.unwrap();
    
    // 创建需求
    sqlx::query(
        "INSERT INTO project_needs (project_id, title, description, priority) VALUES ($1, $2, $3, $4)"
    )
    .bind(project_id)
    .bind(title)
    .bind(description)
    .bind(priority)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 检查用户是否已经是参与者
    let is_participant = sqlx::query_scalar!(r#"
        SELECT COUNT(*) FROM project_participants WHERE project_id = $1 AND user_id = $2
    "#, project_id, user_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap()
    .unwrap_or(0) > 0;
    
    // 如果不是参与者，添加为参与者
    if !is_participant {
        sqlx::query!(r#"
            INSERT INTO project_participants (project_id, user_id, role, message)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (project_id, user_id) DO NOTHING
        "#, project_id, user_id, "participant", "通过提交需求自动加入")
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    
    // 提交事务
    tx.commit().await.unwrap();
    
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

#[axum::debug_handler]
pub async fn update_need(
    State(state): State<AppState>,
    Path((project_id, need_id)): Path<(i64, i64)>,
    headers: axum::http::HeaderMap,
    Form(UpdateNeed { title, description, priority, status }): Form<UpdateNeed>,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    // Get current need
    let need: ProjectNeed = sqlx::query_as!(ProjectNeed, 
        "SELECT id, project_id, title, description, priority, status, created_at, updated_at FROM project_needs WHERE id = $1 AND project_id = $2",
        need_id, project_id
    )
    .fetch_one(&state.pool)
    .await
    .unwrap();
    
    // Update need
    sqlx::query(
        "UPDATE project_needs SET title = $1, description = $2, priority = $3, status = $4, updated_at = now() WHERE id = $5"
    )
    .bind(title.unwrap_or(need.title))
    .bind(description.unwrap_or(need.description))
    .bind(priority.unwrap_or(need.priority))
    .bind(status.unwrap_or(need.status))
    .bind(need_id)
    .execute(&state.pool)
    .await
    .unwrap();
    
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 处理删除需求
#[axum::debug_handler]
pub async fn delete_need(
    State(state): State<AppState>,
    Path((project_id, need_id)): Path<(i64, i64)>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user_info.as_ref().unwrap().id;
    
    // 检查用户是否有项目编辑权限
    if !has_project_edit_permission(pool, project_id, user_id).await {
        return Redirect::to(&format!("/projects/{}", project_id)).into_response();
    }
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 删除需求与创意的关联关系
    sqlx::query!(r#"
        DELETE FROM need_idea_relations
        WHERE need_id = $1
    "#, need_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除需求
    sqlx::query!(r#"
        DELETE FROM project_needs
        WHERE id = $1 AND project_id = $2
    "#, need_id, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 提交事务
    tx.commit().await.unwrap();
    
    // 记录审计日志
    log_audit_action(
        pool,
        user_id,
        "delete",
        "project_need",
        need_id,
        &format!("删除项目需求: project_id={}, need_id={}", project_id, need_id)
    ).await;
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

#[axum::debug_handler]
pub async fn get_project_needs(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Json<Vec<ProjectNeed>> {
    let needs: Vec<ProjectNeed> = sqlx::query_as!(ProjectNeed, 
        "SELECT id, project_id, title, description, priority, status, created_at, updated_at FROM project_needs WHERE project_id = $1 ORDER BY priority DESC, created_at DESC",
        project_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    
    Json(needs)
}

// 创意表单
#[derive(Template)]
#[template(path = "idea_form.html")]
struct IdeaFormTemplate {
    project_id: i64,
    user: Option<UserInfo>,
    user_exists: bool,
}

// Idea form handler
#[axum::debug_handler]
pub async fn create_idea_form(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    // 获取项目信息
    let project = sqlx::query!(r#"
        SELECT title
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap();
    
    match project {
        Some(_) => {
            let template = IdeaFormTemplate {
                project_id,
                user: user_info,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        None => {
            // 项目不存在，重定向到首页
            axum::response::Redirect::to("/").into_response()
        }
    }
}

// 创意提交表单
#[derive(serde::Deserialize)]
pub struct CreateIdeaForm {
    title: String,
    content: String,
    idea_type: String,
    feasibility_score: i32,
    estimated_cost: String,
    need_id: i64,
    project_id: i64,
}

// 评论提交表单
#[derive(serde::Deserialize)]
pub struct CreateCommentForm {
    idea_id: i64,
    project_id: i64,
    content: String,
    parent_id: Option<String>,
}

// 评论编辑表单
#[derive(serde::Deserialize)]
pub struct EditCommentForm {
    content: String,
}

// 评论批量删除表单
#[derive(serde::Deserialize)]
pub struct BatchDeleteCommentsForm {
    comment_ids: String,
    idea_id: i64,
}

// 评论相关的处理函数
#[axum::debug_handler]
pub async fn create_comment(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<CreateCommentForm>,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    
    // 生成评论者身份
    let (user_id, name): (Option<i64>, String) = match user_info {
        Some(user) => (Some(user.id), user.username),
        None => {
            // 生成游客随机数字
            let random_num = rand::random::<u32>();
            let guest_name = format!("游客{}", random_num);
            (None, guest_name)
        }
    };
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 处理 parent_id，将 Option<String> 转换为 Option<i64>
    let parent_id: Option<i64> = form.parent_id.and_then(|id| id.parse().ok());
    
    // 插入评论
    sqlx::query(r#"
        INSERT INTO comments (project_id, idea_id, user_id, name, content, parent_id)
        VALUES ($1, $2, $3, $4, $5, $6)
    "#)
    .bind(form.project_id)
    .bind(form.idea_id)
    .bind(user_id)
    .bind(name)
    .bind(form.content)
    .bind(parent_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 提交事务
    tx.commit().await.unwrap();
    
    // 返回成功响应
    let response = serde_json::json!({"status": "success"});
    Json(response).into_response()
}

// 获取评论列表
#[axum::debug_handler]
pub async fn get_comments(
    State(state): State<AppState>,
    Path(idea_id): Path<i64>,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取评论列表
    let comments = sqlx::query(r#"
        SELECT id, name, content, created_at, parent_id
        FROM comments
        WHERE idea_id = $1
        ORDER BY created_at DESC
    "#)
    .bind(idea_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| {
        serde_json::json! {
            {
                "id": row.get::<i64, _>("id"),
                "name": row.get::<String, _>("name"),
                "content": row.get::<String, _>("content"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "parent_id": row.get::<Option<i64>, _>("parent_id")
            }
        }
    })
    .collect::<Vec<_>>();
    
    // 转换为JSON响应
    let response = serde_json::json!({"comments": comments});
    Json(response).into_response()
}

// 处理删除评论
#[axum::debug_handler]
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，返回错误
    if !user_exists {
        return (axum::http::StatusCode::UNAUTHORIZED, "用户未登录").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user_info.as_ref().unwrap().id;
    
    // 获取评论信息
    let comment = sqlx::query!(r#"
        SELECT user_id, idea_id
        FROM comments
        WHERE id = $1
    "#, comment_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    // 检查用户是否是评论的创建者或管理员
    let username = user_info.as_ref().unwrap().username.clone();
    let is_admin = username == "admin";
    
    let has_permission = match comment {
        Some(comment_info) => is_admin || comment_info.user_id == Some(user_id),
        None => false,
    };
    
    if !has_permission {
        return (axum::http::StatusCode::FORBIDDEN, "没有权限删除此评论").into_response();
    }
    
    // 删除评论
    sqlx::query!(r#"
        DELETE FROM comments
        WHERE id = $1
    "#, comment_id)
    .execute(pool)
    .await
    .unwrap();
    
    // 记录审计日志
    log_audit_action(
        pool,
        user_id,
        "delete",
        "comment",
        comment_id,
        &format!("删除评论: comment_id={}", comment_id)
    ).await;
    
    // 返回成功响应
    let response = serde_json::json!({"status": "success"});
    Json(response).into_response()
}

// 处理编辑评论
#[axum::debug_handler]
pub async fn edit_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
    headers: axum::http::HeaderMap,
    Form(form): Form<EditCommentForm>,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，返回错误
    if !user_exists {
        return (axum::http::StatusCode::UNAUTHORIZED, "用户未登录").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user_info.as_ref().unwrap().id;
    
    // 获取评论信息
    let comment = sqlx::query!(r#"
        SELECT user_id
        FROM comments
        WHERE id = $1
    "#, comment_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    // 检查用户是否是评论的创建者
    let has_permission = match comment {
        Some(comment_info) => comment_info.user_id == Some(user_id),
        None => false,
    };
    
    if !has_permission {
        return (axum::http::StatusCode::FORBIDDEN, "没有权限编辑此评论").into_response();
    }
    
    // 更新评论
    sqlx::query!(r#"
        UPDATE comments
        SET content = $1, updated_at = now()
        WHERE id = $2
    "#, form.content, comment_id)
    .execute(pool)
    .await
    .unwrap();
    
    // 记录审计日志
    log_audit_action(
        pool,
        user_id,
        "update",
        "comment",
        comment_id,
        &format!("编辑评论: comment_id={}", comment_id)
    ).await;
    
    // 返回成功响应
    let response = serde_json::json!({"status": "success"});
    Json(response).into_response()
}

// 处理批量删除评论
#[axum::debug_handler]
pub async fn batch_delete_comments(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<BatchDeleteCommentsForm>,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，返回错误
    if !user_exists {
        return (axum::http::StatusCode::UNAUTHORIZED, "用户未登录").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user_info.as_ref().unwrap().id;
    
    // 解析评论 ID 字符串，按逗号分隔
    let comment_ids: Vec<i64> = form.comment_ids
        .split(',')
        .filter_map(|id| id.parse().ok())
        .collect();
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 检查所有评论的权限并删除
    for comment_id in &comment_ids {
        // 获取评论信息
        let comment = sqlx::query!(r#"
            SELECT user_id
            FROM comments
            WHERE id = $1
        "#, comment_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap();
        
        // 检查用户是否是评论的创建者或管理员
        let username = user_info.as_ref().unwrap().username.clone();
        let is_admin = username == "admin";
        
        let has_permission = match comment {
            Some(comment_info) => is_admin || comment_info.user_id == Some(user_id),
            None => false,
        };
        
        if has_permission {
            // 删除评论
            sqlx::query!(r#"
                DELETE FROM comments
                WHERE id = $1
            "#, comment_id)
            .execute(&mut *tx)
            .await
            .unwrap();
            
            // 记录审计日志
            log_audit_action(
                pool,
                user_id,
                "delete",
                "comment",
                *comment_id,
                &format!("批量删除评论: comment_id={}", comment_id)
            ).await;
        }
    }
    
    // 提交事务
    tx.commit().await.unwrap();
    
    // 返回成功响应
    let response = serde_json::json!({"status": "success"});
    Json(response).into_response()
}

// 处理删除创意
#[axum::debug_handler]
pub async fn delete_idea(
    State(state): State<AppState>,
    Path((project_id, idea_id)): Path<(i64, i64)>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user_info.as_ref().unwrap().id;
    
    // 检查用户是否是创意的创建者或有项目编辑权限
    let idea_creator = sqlx::query_scalar!(r#"
        SELECT user_id
        FROM ideas
        WHERE id = $1 AND project_id = $2
    "#, idea_id, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    let has_permission = match idea_creator {
        Some(creator_id) => creator_id == user_id || has_project_edit_permission(pool, project_id, user_id).await,
        None => false,
    };
    
    if !has_permission {
        return Redirect::to(&format!("/projects/{}", project_id)).into_response();
    }
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 删除创意的评论
    sqlx::query!(r#"
        DELETE FROM comments
        WHERE idea_id = $1
    "#, idea_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除创意的投票
    sqlx::query!(r#"
        DELETE FROM idea_votes
        WHERE idea_id = $1
    "#, idea_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除需求与创意的关联关系
    sqlx::query!(r#"
        DELETE FROM need_idea_relations
        WHERE idea_id = $1
    "#, idea_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除创意
    sqlx::query!(r#"
        DELETE FROM ideas
        WHERE id = $1 AND project_id = $2
    "#, idea_id, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 提交事务
    tx.commit().await.unwrap();
    
    // 检查并移除该创意所属用户在项目参与者列表中的记录
    // 获取创意的创建者
    let idea_creator = sqlx::query_scalar!(r#"
        SELECT user_id
        FROM ideas
        WHERE id = $1 AND project_id = $2
    "#, idea_id, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if let Some(creator_id) = idea_creator {
        // 检查该用户是否还有其他创意
        let other_ideas_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*)
            FROM ideas
            WHERE project_id = $1 AND user_id = $2 AND id != $3
        "#, project_id, creator_id, idea_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        // 检查该用户是否是项目参与者
        let participant_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*)
            FROM project_participants
            WHERE project_id = $1 AND user_id = $2
        "#, project_id, creator_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        let is_participant = participant_count > 0;
        
        // 如果该用户没有其他创意且是项目参与者，则从参与者列表中移除
        if other_ideas_count == 0 && is_participant {
            // 检查该用户是否是项目创建者
            let is_creator = sqlx::query_scalar!(r#"
                SELECT user_id
                FROM projects
                WHERE id = $1
            "#, project_id)
            .fetch_optional(pool)
            .await
            .unwrap() == Some(creator_id);
            
            // 不能移除项目创建者
            if !is_creator {
                // 移除参与者
                sqlx::query!(r#"
                    DELETE FROM project_participants
                    WHERE project_id = $1 AND user_id = $2
                "#, project_id, creator_id)
                .execute(pool)
                .await
                .unwrap();
                
                // 记录审计日志
                log_audit_action(
                    pool,
                    user_id,
                    "delete",
                    "project_participant",
                    creator_id,
                    &format!("自动移除项目参与者: project_id={}, participant_id={}, reason=创意被删除且无其他创意", project_id, creator_id)
                ).await;
            }
        }
    }
    
    // 记录审计日志
    log_audit_action(
        pool,
        user_id,
        "delete",
        "idea",
        idea_id,
        &format!("删除项目创意: project_id={}, idea_id={}", project_id, idea_id)
    ).await;
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// Idea handlers
#[axum::debug_handler]
pub async fn create_idea(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: axum::http::HeaderMap,
    Form(form): Form<CreateIdeaForm>,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user_info = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user_info.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return axum::response::Redirect::to("/login").into_response();
    }
    
    let user_id = user_info.as_ref().unwrap().id;
    
    // 开始事务
    let mut tx = state.pool.begin().await.unwrap();
    
    // 插入创意
    let idea_id = sqlx::query!(r#"
        INSERT INTO ideas (project_id, user_id, title, content, idea_type, feasibility_score, estimated_cost)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
    "#, project_id, user_id, form.title, form.content, form.idea_type, form.feasibility_score, form.estimated_cost)
    .fetch_one(&mut *tx)
    .await
    .unwrap()
    .id;
    
    // 关联需求和创意
    sqlx::query!(r#"
        INSERT INTO need_idea_relations (need_id, idea_id)
        VALUES ($1, $2)
        ON CONFLICT (need_id, idea_id) DO NOTHING
    "#, form.need_id, idea_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 检查用户是否已经是参与者
    let is_participant = sqlx::query_scalar!(r#"
        SELECT COUNT(*) FROM project_participants WHERE project_id = $1 AND user_id = $2
    "#, project_id, user_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap()
    .unwrap_or(0) > 0;
    
    // 如果不是参与者，添加为参与者
    if !is_participant {
        sqlx::query!(r#"
            INSERT INTO project_participants (project_id, user_id, role, message)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (project_id, user_id) DO NOTHING
        "#, project_id, user_id, "participant", "通过提交创意自动加入")
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    
    // 提交事务
    tx.commit().await.unwrap();
    
    Redirect::to(&format!("/projects/{}/needs/{}", project_id, form.need_id)).into_response()
}