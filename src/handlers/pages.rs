use axum::{extract::{Path, State, Query, Extension, Form}, response::{Html, IntoResponse, Redirect, Response}, http::{HeaderMap, HeaderValue}, Json};
use askama::Template;
use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use crate::handlers::AppState;
use crate::auth::session::Session;
use chrono::{DateTime, Utc, NaiveDateTime};
use crate::models::project_template::ProjectTemplate;
use crate::models::project_template::NewProjectTemplate;
use std::fmt::Write;

// 从cookie中提取会话信息
pub async fn get_session_from_cookies(headers: &HeaderMap, state: &AppState) -> Option<Session> {
    let token = headers
        .get("cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|part| part.trim().starts_with("session_token="))
                .map(|part| part.trim().trim_start_matches("session_token="))
        });
    
    if let Some(token) = token {
        state.session_store.get_session(token).await
    } else {
        None
    }
}

// 记录审计日志
pub async fn log_audit_action(pool: &sqlx::Pool<sqlx::Postgres>, user_id: i64, action_type: &str, entity_type: &str, entity_id: i64, details: &str) {
    let _ = sqlx::query(r#"
        INSERT INTO audit_logs (user_id, action_type, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5)
    "#)
    .bind(user_id)
    .bind(action_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details)
    .execute(pool)
    .await;
}

// 记录用户活动
pub async fn log_user_activity(pool: &sqlx::Pool<sqlx::Postgres>, user_id: i64, activity_type: &str, entity_type: &str, entity_id: Option<i64>, action: &str, details: Option<&str>) {
    let _ = sqlx::query(r#"
        INSERT INTO user_activity_logs (user_id, activity_type, entity_type, entity_id, action, details)
        VALUES ($1, $2, $3, $4, $5, $6)
    "#)
    .bind(user_id)
    .bind(activity_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(details)
    .execute(pool)
    .await;
}

// 从session获取用户信息，包括头像URL
pub async fn get_user_info_from_session(session: &Session, state: &AppState) -> UserInfo {
    let pool = &state.pool;
    
    // 从数据库中获取用户的头像URL
    let user = sqlx::query!(r#"
        SELECT avatar_url
        FROM users
        WHERE id = $1
    "#, session.user_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    let avatar_url = user
        .map(|u| u.avatar_url)
        .unwrap_or("https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=user%20avatar%20profile%20picture&image_size=square".to_string());
    
    UserInfo {
        id: session.user_id,
        username: session.username.clone(),
        avatar_url,
    }
}

// 首页模板数据
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    featured_projects: Vec<FeaturedProject>,
    popular_tags: Vec<PopularTag>,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 精选项目结构
#[derive(Debug)]
struct FeaturedProject {
    id: i64,
    title: String,
    summary: String,
    username: String,
    created_at: DateTime<Utc>,
    stage: String,
    tags: Vec<String>,
}

// 热门标签结构
#[derive(Debug)]
struct PopularTag {
    name: String,
    count: i64,
}



// 用户信息结构
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub avatar_url: String,
}

// 用户活动记录结构
#[derive(Debug)]
struct UserActivity {
    id: i64,
    activity_type: String,
    entity_type: String,
    action: String,
    details: String,
    created_at: DateTime<Utc>,
}

// 用户个人资料模板数据
#[derive(Template)]
#[template(path = "user_profile.html")]
struct UserProfileTemplate {
    profile_user: UserDetail,
    stats: UserStats,
    user_projects: Vec<UserProject>,
    participated_projects: Vec<ParticipatedProject>,
    user_ideas: Vec<UserIdea>,
    user_activities: Vec<UserActivity>,
    current_user: Option<UserInfo>,
    current_user_id: Option<i64>,
    current_user_exists: bool,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 用户详情结构
#[derive(Debug)]
struct UserDetail {
    id: i64,
    username: String,
    bio: String,
    avatar_url: String,
    created_at: DateTime<Utc>,
}

// 编辑个人资料模板
#[derive(Template)]
#[template(path = "edit_profile.html")]
struct EditProfileTemplate {
    user: Option<UserInfo>,
    user_exists: bool,
    user_detail: UserDetail,
}

// 用户统计结构
#[derive(Debug)]
struct UserStats {
    projects_count: i64,
    ideas_count: i64,
    participations_count: i64,
    contribution_score: i64,
}

// 用户项目结构
#[derive(Debug)]
struct UserProject {
    id: i64,
    title: String,
    summary: String,
    stage: String,
    created_at: DateTime<Utc>,
    tags: Vec<String>,
}

// 参与的项目结构
#[derive(Debug)]
struct ParticipatedProject {
    id: i64,
    title: String,
    summary: String,
    role: String,
    joined_at: DateTime<Utc>,
    tags: Vec<String>,
}

// 用户创意结构
#[derive(Debug)]
struct UserIdea {
    id: i64,
    title: String,
    content: String,
    idea_type: String,
    project_id: i64,
    created_at: DateTime<Utc>,
}

// 标签页面模板数据
#[derive(Template)]
#[template(path = "tags.html")]
struct TagsTemplate {
    all_tags: Vec<PopularTag>,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 标签详情页面模板数据
#[derive(Template)]
#[template(path = "tag_detail.html")]
struct TagDetailTemplate {
    tag_name: String,
    projects: Vec<FeaturedProject>,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 项目详情页面模板数据
#[derive(Template)]
#[template(path = "project_detail.html")]
struct ProjectDetailTemplate {
    project: ProjectDetail,
    progress_updates: Vec<ProgressUpdate>,
    needs: Vec<ProjectNeed>,
    ideas: Vec<ProjectIdea>,
    participants: Vec<ProjectParticipant>,
    user: Option<UserInfo>,
    user_exists: bool,
    is_admin: bool,
}

// 项目详情结构
#[derive(Debug)]
struct ProjectImage {
    id: i64,
    image_url: String,
    is_main: bool,
}

#[derive(Debug)]
struct ProjectDetail {
    id: i64,
    title: String,
    summary: String,
    description: String,
    username: String,
    avatar_url: String,
    created_at: DateTime<Utc>,
    stage: String,
    tags: Vec<String>,
    existing_resources: String,
    needed_resources: String,
    user_id: i64,
    images: Vec<ProjectImage>,
}

// 进度更新结构
#[derive(Debug)]
struct ProgressUpdate {
    content: String,
    username: String,
    created_at: DateTime<Utc>,
    progress_percentage: Option<i32>,
    update_date: Option<chrono::NaiveDate>,
}

// 为ProgressUpdate实现Display trait，以便在模板中使用
impl std::fmt::Display for ProgressUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

// 项目需求结构
#[derive(Debug)]
struct ProjectNeed {
    id: i64,
    title: String,
    description: String,
    priority: String,
    status: String,
}

// 项目创意结构
#[derive(Debug)]
struct ProjectIdea {
    id: i64,
    project_id: i64,
    title: String,
    content: String,
    idea_type: String,
    feasibility_score: i32,
    estimated_cost: String,
    username: String,
    avatar_url: String,
    created_at: DateTime<Utc>,
}

impl ProjectIdea {
    fn rendered_content(&self) -> String {
        use comrak::{markdown_to_html, ComrakOptions};
        let options = ComrakOptions::default();
        markdown_to_html(&self.content, &options)
    }
}

// 项目参与者结构
#[derive(Debug)]
struct ProjectParticipant {
    user_id: i64,
    username: String,
    avatar_url: String,
    role: String,
    contribution_score: i64,
}

// 用于导出的项目数据结构
#[derive(Debug, Serialize)]
pub struct ExportProject {
    id: i64,
    title: String,
    summary: String,
    description: String,
    username: String,
    created_at: DateTime<Utc>,
    stage: String,
    tags: Vec<String>,
    existing_resources: String,
    needed_resources: String,
    progress_updates: Vec<ExportProgressUpdate>,
    needs: Vec<ExportProjectNeed>,
    ideas: Vec<ExportProjectIdea>,
    participants: Vec<ExportProjectParticipant>,
}

#[derive(Debug, Serialize)]
pub struct ExportProgressUpdate {
    content: String,
    username: String,
    created_at: DateTime<Utc>,
    progress_percentage: Option<i32>,
    update_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct ExportProjectNeed {
    id: i64,
    title: String,
    description: String,
    priority: String,
    status: String,
}

#[derive(Debug, Serialize)]
pub struct ExportProjectIdea {
    id: i64,
    title: String,
    content: String,
    idea_type: String,
    feasibility_score: i32,
    estimated_cost: String,
    username: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ExportProjectParticipant {
    user_id: i64,
    username: String,
    role: String,
    contribution_score: i64,
}

// 处理首页请求
pub async fn home_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取精选项目
    let featured_projects = get_featured_projects(pool).await;
    
    // 获取热门标签
    let popular_tags = get_popular_tags(pool).await;
    
    // 提取用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    let template = IndexTemplate {
        featured_projects,
        popular_tags,
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理用户个人资料请求
pub async fn user_profile_handler(Path(username): Path<String>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取用户详情
    let user = get_user_detail(pool, &username).await;
    
    if user.is_none() {
        return Html("用户不存在").into_response();
    }
    
    let user = user.unwrap();
    
    // 获取用户统计信息
    let stats = get_user_stats(pool, user.id).await;
    
    // 获取用户发起的项目
    let user_projects = get_user_projects(pool, user.id).await;
    
    // 获取用户参与的项目
    let participated_projects = get_participated_projects(pool, user.id).await;
    
    // 获取用户提交的创意
    let user_ideas = get_user_ideas(pool, user.id).await;
    
    // 获取用户活动记录
    let user_activities = get_user_activities(pool, user.id).await;
    
    // 提取当前用户信息
    let current_user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let current_user_exists = current_user.is_some();
    let user_exists = current_user.is_some();
    let current_user_id = current_user.as_ref().map(|u| u.id);
    
    let template = UserProfileTemplate {
        profile_user: user,
        stats,
        user_projects,
        participated_projects,
        user_ideas,
        user_activities,
        current_user: current_user.clone(),
        current_user_id,
        current_user_exists,
        user: current_user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理标签页面请求
pub async fn tags_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取所有标签
    let all_tags = get_all_tags(pool).await;
    
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    let template = TagsTemplate {
        all_tags,
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理标签详情页面请求
pub async fn tag_detail_handler(Path(tag_name): Path<String>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取标签相关的项目
    let projects = get_projects_by_tag(pool, &tag_name).await;
    
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    let template = TagDetailTemplate {
        tag_name,
        projects,
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 登录模板
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: String,
    pub csrf_token: String,
}

// 注册模板
#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub error: String,
    pub csrf_token: String,
}

// 项目表单模板数据
#[derive(Template)]
#[template(path = "project_form.html")]
struct ProjectFormTemplate {
    title: String,
    action: String,
    cancel_url: String,
    error: String,
    project: ProjectFormData,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 项目表单数据结构
#[derive(Debug)]
struct ProjectFormData {
    title: String,
    summary: String,
    description: String,
    stage: String,
    tags: String,
}

// 项目创建请求结构
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    title: String,
    summary: String,
    description: String,
    stage: String,
    tags: String,
}

// 项目编辑请求结构
#[derive(Debug, Deserialize)]
pub struct EditProjectRequest {
    title: String,
    summary: String,
    description: String,
    stage: String,
    tags: String,
}

// 项目列表模板数据
#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate {
    projects: Vec<ProjectItem>,
    categories: Vec<(i64, String)>,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 项目搜索参数结构
#[derive(Debug, serde::Deserialize)]
pub struct ProjectSearchParams {
    search: Option<String>,
    tag: Option<String>,
    category: Option<i64>,
    stage: Option<String>,
    sort_by: Option<String>,
    archived: Option<bool>,
}

// 项目项结构
#[derive(Debug)]
struct ProjectItem {
    id: i64,
    title: String,
    summary: String,
    username: String,
    created_at: DateTime<Utc>,
    stage: String,
    category: String,
    tags: Vec<String>,
    is_archived: bool,
}
// 处理登录页面请求
pub async fn login_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 从cookie中获取CSRF令牌
    let csrf_token = headers
        .get("cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|part| part.trim().starts_with("csrf_token="))
                .map(|part| part.trim().trim_start_matches("csrf_token="))
        })
        .unwrap_or("")
        .to_string();
    
    let template = LoginTemplate {
        error: "".to_string(),
        csrf_token,
    };
    Html(template.render().unwrap()).into_response()
}

// 处理注册页面请求
pub async fn register_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 从cookie中获取CSRF令牌
    let csrf_token = headers
        .get("cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|part| part.trim().starts_with("csrf_token="))
                .map(|part| part.trim().trim_start_matches("csrf_token="))
        })
        .unwrap_or("")
        .to_string();
    
    let template = RegisterTemplate {
        error: "".to_string(),
        csrf_token,
    };
    Html(template.render().unwrap()).into_response()
}

// 处理项目列表页面请求
pub async fn projects_handler(State(state): State<AppState>, headers: HeaderMap, Query(params): Query<ProjectSearchParams>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 构建搜索查询
    let mut query = String::from(r#"
        SELECT p.id, p.title, p.summary, u.username, p.created_at, p.stage, pc.name as category_name, p.is_archived
        FROM projects p
        JOIN users u ON p.user_id = u.id
        LEFT JOIN project_categories pc ON p.category_id = pc.id
    "#);
    
    let mut where_clauses = Vec::new();
    let mut query_params = Vec::new();
    let mut param_index = 1;
    
    // 添加搜索条件
    if let Some(search) = &params.search {
        if !search.is_empty() {
            where_clauses.push(format!("(p.title ILIKE ${} OR p.summary ILIKE ${} OR p.description ILIKE ${})
", param_index, param_index, param_index));
            query_params.push(format!("%{search}%"));
            param_index += 1;
        }
    }
    
    // 添加标签过滤
    if let Some(tag) = &params.tag {
        if !tag.is_empty() {
            query = String::from(r#"
                SELECT DISTINCT p.id, p.title, p.summary, u.username, p.created_at, p.stage
                FROM projects p
                JOIN users u ON p.user_id = u.id
                JOIN project_tags pt ON p.id = pt.project_id
            "#);
            where_clauses.push(format!("pt.tag ILIKE ${}
", param_index));
            query_params.push(format!("%{tag}%"));
            param_index += 1;
        }
    }
    
    // 添加分类过滤
    if let Some(category) = &params.category {
        where_clauses.push(format!("p.category_id = ${}
", param_index));
        query_params.push(category.to_string());
        param_index += 1;
    }
    
    // 添加阶段过滤
    if let Some(stage) = &params.stage {
        if !stage.is_empty() {
            where_clauses.push(format!("p.stage = ${}
", param_index));
            query_params.push(stage.clone());
            param_index += 1;
        }
    }
    
    // 添加归档状态过滤
    if params.archived != Some(true) {
        // 默认只显示未归档的项目
        where_clauses.push("p.is_archived = false
".to_string());
    }
    
    // 添加WHERE子句
    if !where_clauses.is_empty() {
        query += "WHERE ";
        query += &where_clauses.join("AND ");
    }
    
    // 添加排序
    match &params.sort_by {
        Some(sort) => {
            match sort.as_str() {
                "newest" => query += "ORDER BY p.created_at DESC\n",
                "oldest" => query += "ORDER BY p.created_at ASC\n",
                _ => query += "ORDER BY p.created_at DESC\n",
            }
        }
        None => query += "ORDER BY p.created_at DESC\n",
    }
    
    // 执行查询
    let mut sql_query = sqlx::query(&query);
    for param in query_params {
        sql_query = sql_query.bind(param);
    }
    
    let projects = sql_query
        .fetch_all(pool)
        .await
        .unwrap();
    
    // 构建项目列表
    let mut project_items = Vec::new();
    for project in projects {
        // 从PgRow中获取字段值
        let id: i64 = project.get(0);
        let title: String = project.get(1);
        let summary: String = project.get(2);
        let username: String = project.get(3);
        let created_at: DateTime<Utc> = project.get(4);
        let stage: String = project.get(5);
        let category: String = project.get::<Option<String>, _>(6).unwrap_or_else(|| "".to_string());
        let is_archived: bool = project.get(7);
        
        // 获取项目标签
        let tags = sqlx::query!(r#"
            SELECT tag
            FROM project_tags
            WHERE project_id = $1
        "#, id)
        .fetch_all(pool)
        .await
        .unwrap();
        
        let tag_names = tags.iter().map(|t| t.tag.clone()).collect::<Vec<String>>();
        
        project_items.push(ProjectItem {
            id,
            title,
            summary,
            username,
            created_at,
            stage,
            category,
            tags: tag_names,
            is_archived,
        });
    }
    
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 获取所有分类
    let categories = sqlx::query!(r#"
        SELECT id, name
        FROM project_categories
        ORDER BY name
    "#)
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|c| (c.id, c.name))
    .collect::<Vec<(i64, String)>>();
    
    let template = ProjectsTemplate {
        projects: project_items,
        categories,
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理项目创建页面请求
pub async fn create_project_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let template = ProjectFormTemplate {
        title: "创建新项目".to_string(),
        action: "/projects/create".to_string(),
        cancel_url: "/".to_string(),
        error: "".to_string(),
        project: ProjectFormData {
            title: "".to_string(),
            summary: "".to_string(),
            description: "".to_string(),
            stage: "planning".to_string(),
            tags: "".to_string(),
        },
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理项目创建表单提交
pub async fn create_project_post_handler(State(state): State<AppState>, headers: HeaderMap, Form(form): Form<CreateProjectRequest>) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 创建项目
    let project = sqlx::query!(r#"
        INSERT INTO projects (title, summary, description, stage, user_id, slug)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
    "#, form.title, form.summary, form.description, form.stage, user_id, form.title.replace(' ', "-").to_lowercase())
    .fetch_one(&mut *tx)
    .await;
    
    match project {
        Ok(project) => {
            // 处理标签
            let tags = form.tags.split(",").map(|tag| tag.trim()).filter(|tag| !tag.is_empty());
            for tag in tags {
                // 直接在project_tags表中插入标签
                sqlx::query!(r#"
                    INSERT INTO project_tags (project_id, tag)
                    VALUES ($1, $2)
                    ON CONFLICT DO NOTHING
                "#, project.id, tag)
                .execute(&mut *tx)
                .await
                .unwrap();
            }
            
            // 添加项目创建进度更新
            sqlx::query!(r#"
                INSERT INTO project_progress (project_id, user_id, content, created_at)
                VALUES ($1, $2, $3, now())
            "#, project.id, user_id, format!("项目 '{}' 创建成功", form.title))
            .execute(&mut *tx)
            .await
            .unwrap();
            
            // 添加创建者为项目参与者，角色为creator
            sqlx::query!(r#"
                INSERT INTO project_participants (project_id, user_id, role, message)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (project_id, user_id) DO NOTHING
            "#, project.id, user_id, "creator", "项目创建者")
            .execute(&mut *tx)
            .await
            .unwrap();
            
            // 提交事务
            tx.commit().await.unwrap();
            
            // 重定向到项目详情页
            Redirect::to(&format!("/projects/{}", project.id)).into_response()
        },
        Err(_) => {
            // 回滚事务
            tx.rollback().await.unwrap();
            
            // 显示错误信息
            let template = ProjectFormTemplate {
                title: "创建新项目".to_string(),
                action: "/projects/create".to_string(),
                cancel_url: "/".to_string(),
                error: "创建项目失败，请重试".to_string(),
                project: ProjectFormData {
                    title: form.title,
                    summary: form.summary,
                    description: form.description,
                    stage: form.stage,
                    tags: form.tags,
                },
                user,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        }
    }
}

// 处理项目编辑页面请求
pub async fn edit_project_handler(Path(project_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 获取项目详情
    let project = sqlx::query!(r#"
        SELECT id, title, summary, description, stage, user_id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    match project {
        Some(project) => {
            // 检查用户是否有项目编辑权限
            if !has_project_edit_permission(pool, project_id, user_id).await {
                return Redirect::to("/").into_response();
            }
            
            // 获取项目标签
            let tags = sqlx::query!(r#"
                SELECT tag
                FROM project_tags
                WHERE project_id = $1
            "#, project_id)
            .fetch_all(pool)
            .await
            .unwrap();
            
            let tags_str = tags.iter().map(|t| t.tag.clone()).collect::<Vec<String>>().join(", ");
            
            let template = ProjectFormTemplate {
                title: "编辑项目".to_string(),
                action: format!("/projects/{}/edit", project_id),
                cancel_url: format!("/projects/{}", project_id),
                error: "".to_string(),
                project: ProjectFormData {
                    title: project.title,
                    summary: project.summary,
                    description: project.description,
                    stage: project.stage,
                    tags: tags_str,
                },
                user,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        None => {
            // 项目不存在，重定向到首页
            Redirect::to("/").into_response()
        }
    }
}

// 处理项目编辑表单提交
pub async fn edit_project_post_handler(Path(project_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap, Form(form): Form<EditProjectRequest>) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 检查用户是否是项目创建者
    let project = sqlx::query!(r#"
        SELECT user_id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    match project {
        Some(_) => {
            // 检查用户是否有项目编辑权限
            if !has_project_edit_permission(pool, project_id, user_id).await {
                return Redirect::to("/").into_response();
            }
            
            // 开始事务
            let mut tx = pool.begin().await.unwrap();
            
            // 更新项目
            sqlx::query!(r#"
                UPDATE projects
                SET title = $1, summary = $2, description = $3, stage = $4, slug = $5
                WHERE id = $6
            "#, form.title, form.summary, form.description, form.stage, form.title.replace(' ', "-").to_lowercase(), project_id)
            .execute(&mut *tx)
            .await
            .unwrap();
            
            // 删除旧标签关联
            sqlx::query!(r#"
                DELETE FROM project_tags
                WHERE project_id = $1
            "#, project_id)
            .execute(&mut *tx)
            .await
            .unwrap();
            
            // 处理新标签
            let tags = form.tags.split(",").map(|tag| tag.trim()).filter(|tag| !tag.is_empty());
            for tag in tags {
                // 直接在project_tags表中插入标签
                sqlx::query!(r#"
                    INSERT INTO project_tags (project_id, tag)
                    VALUES ($1, $2)
                "#, project_id, tag)
                .execute(&mut *tx)
                .await
                .unwrap();
            }
            
            // 提交事务
            tx.commit().await.unwrap();
            
            // 重定向到项目详情页
            Redirect::to(&format!("/projects/{}", project_id)).into_response()
        },
        None => {
            // 项目不存在，重定向到首页
            Redirect::to("/").into_response()
        }
    }
}

// 处理项目详情页面请求
pub async fn project_detail_handler(Path(project_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取项目详情
    let project = get_project_detail(pool, project_id).await;
    
    if project.is_none() {
        return Html("项目不存在").into_response();
    }
    
    let project = project.unwrap();
    
    // 获取项目进度更新
    let progress_updates = get_project_progress_updates(pool, project_id).await;
    
    // 获取项目需求
    let needs = get_project_needs_list(pool, project_id).await;
    
    // 获取项目创意
    let ideas = get_project_ideas(pool, project_id).await;
    
    // 获取项目参与者
    let participants = get_project_participants(pool, project_id).await;
    
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 检查当前用户是否是项目管理员（创建者或管理者）
    let is_admin = if let Some(user_info) = &user {
        // 检查是否是项目创建者
        if user_info.id == project.user_id {
            true
        } else {
            // 检查是否是项目管理者
            let participant_role = sqlx::query_scalar!(r#"
                SELECT role
                FROM project_participants
                WHERE project_id = $1 AND user_id = $2
            "#, project_id, user_info.id)
            .fetch_optional(pool)
            .await
            .unwrap();
            participant_role == Some("manager".to_string())
        }
    } else {
        false
    };
    
    let template = ProjectDetailTemplate {
        project,
        progress_updates,
        needs,
        ideas,
        participants,
        user,
        user_exists,
        is_admin,
    };
    
    Html(template.render().unwrap()).into_response()
}

async fn get_project_detail(pool: &PgPool, project_id: i64) -> Option<ProjectDetail> {
    let project = sqlx::query!(r#"
        SELECT p.id, p.title, p.summary, p.description, u.username, u.avatar_url, p.created_at, p.stage, p.existing_resources, p.needed_resources, p.user_id
        FROM projects p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    match project {
        Some(p) => {
            // 查询项目标签
            let tags = sqlx::query!(r#"
                SELECT tag
                FROM project_tags
                WHERE project_id = $1
            "#, p.id)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect();
            
            // 查询项目图片
            let images = sqlx::query(r#"
                SELECT id, image_url, is_main
                FROM project_images
                WHERE project_id = $1
                ORDER BY is_main DESC, created_at DESC
            "#)
            .bind(p.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|row| ProjectImage {
                id: row.get("id"),
                image_url: row.get("image_url"),
                is_main: row.get("is_main"),
            })
            .collect();
            
            Some(ProjectDetail {
                id: p.id,
                title: p.title,
                summary: p.summary,
                description: p.description,
                username: p.username,
                avatar_url: p.avatar_url,
                created_at: p.created_at,
                stage: p.stage,
                tags,
                existing_resources: p.existing_resources,
                needed_resources: p.needed_resources,
                user_id: p.user_id,
                images,
            })
        }
        None => None
    }
}

async fn get_project_progress_updates(pool: &PgPool, project_id: i64) -> Vec<ProgressUpdate> {
    let rows = sqlx::query(r#"
        SELECT pp.content, u.username, pp.created_at, pp.progress_percentage, pp.update_date
        FROM project_progress pp
        JOIN users u ON pp.user_id = u.id
        WHERE pp.project_id = $1
        ORDER BY pp.created_at DESC
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap();
    
    rows.into_iter()
    .map(|p| ProgressUpdate {
        content: p.get::<String, _>("content"),
        username: p.get::<String, _>("username"),
        created_at: p.get::<DateTime<Utc>, _>("created_at"),
        progress_percentage: p.get::<Option<i32>, _>("progress_percentage"),
        update_date: p.get::<Option<chrono::NaiveDate>, _>("update_date"),
    })
    .collect()
}



async fn get_project_needs_list(pool: &PgPool, project_id: i64) -> Vec<ProjectNeed> {
    sqlx::query(r#"
        SELECT id, title, description, priority, status
        FROM project_needs
        WHERE project_id = $1
        ORDER BY created_at DESC
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| ProjectNeed {
        id: row.get::<i64, _>("id"),
        title: row.get::<String, _>("title"),
        description: row.get::<String, _>("description"),
        priority: row.get::<String, _>("priority"),
        status: row.get::<String, _>("status"),
    })
    .collect()
}

async fn get_project_ideas(pool: &PgPool, project_id: i64) -> Vec<ProjectIdea> {
    sqlx::query(r#"
        SELECT i.id, i.title, i.content, i.idea_type, i.feasibility_score, i.estimated_cost, u.username, u.avatar_url, i.created_at
        FROM ideas i
        JOIN users u ON i.user_id = u.id
        WHERE i.project_id = $1
        ORDER BY i.created_at DESC
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| ProjectIdea {
        id: row.get::<i64, _>("id"),
        project_id,
        title: row.get::<String, _>("title"),
        content: row.get::<String, _>("content"),
        idea_type: row.get::<String, _>("idea_type"),
        feasibility_score: row.get::<i32, _>("feasibility_score"),
        estimated_cost: row.get::<String, _>("estimated_cost"),
        username: row.get::<String, _>("username"),
        avatar_url: row.get::<String, _>("avatar_url"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    })
    .collect()
}

async fn get_project_participants(pool: &PgPool, project_id: i64) -> Vec<ProjectParticipant> {
    let participants = sqlx::query(r#"
        SELECT u.id, u.username, u.avatar_url, pp.role
        FROM project_participants pp
        JOIN users u ON pp.user_id = u.id
        WHERE pp.project_id = $1
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut result = Vec::new();
    for participant in participants {
        let user_id = participant.get::<i64, _>("id");
        
        // 计算参与者在该项目中的贡献度
        // 1. 创意数量
        let ideas_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as count
            FROM ideas
            WHERE project_id = $1 AND user_id = $2
        "#, project_id, user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        // 2. 评论数量
        let comments_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as count
            FROM comments
            WHERE project_id = $1 AND user_id = $2
        "#, project_id, user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        // 3. 投票数量
        let votes_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as count
            FROM idea_votes iv
            JOIN ideas i ON iv.idea_id = i.id
            WHERE i.project_id = $1 AND iv.user_id = $2
        "#, project_id, user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        // 计算贡献度得分
        let contribution_score = (ideas_count * 5) + (comments_count * 2) + (votes_count * 1);
        
        result.push(ProjectParticipant {
            user_id,
            username: participant.get::<String, _>("username"),
            avatar_url: participant.get::<String, _>("avatar_url"),
            role: participant.get::<String, _>("role"),
            contribution_score,
        });
    }
    
    // 按贡献度排序
    result.sort_by(|a, b| b.contribution_score.cmp(&a.contribution_score));
    
    result
}

// 数据库查询函数
async fn get_featured_projects(pool: &PgPool) -> Vec<FeaturedProject> {
    // 查询项目及其标签
    let projects = sqlx::query(r#"
        SELECT p.id, p.title, p.summary, u.username, p.created_at, p.stage
        FROM projects p
        JOIN users u ON p.user_id = u.id
        WHERE p.is_public = true
        ORDER BY p.created_at DESC
        LIMIT 3
    "#)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    let mut featured_projects = Vec::new();
    
    for project in projects {
        // 查询项目标签
        let tags = sqlx::query(r#"
            SELECT tag
            FROM project_tags
            WHERE project_id = $1
        "#)
        .bind(project.get::<i64, _>("id"))
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.get::<String, _>("tag"))
        .collect();
        
        featured_projects.push(FeaturedProject {
            id: project.get::<i64, _>("id"),
            title: project.get::<String, _>("title"),
            summary: project.get::<String, _>("summary"),
            username: project.get::<String, _>("username"),
            created_at: project.get::<DateTime<Utc>, _>("created_at"),
            stage: project.get::<String, _>("stage"),
            tags,
        });
    }
    
    featured_projects
}

async fn get_popular_tags(pool: &PgPool) -> Vec<PopularTag> {
    sqlx::query(r#"
        SELECT tag, COUNT(*) as count
        FROM project_tags
        GROUP BY tag
        ORDER BY count DESC
        LIMIT 6
    "#)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| PopularTag { 
        name: row.get::<String, _>("tag"), 
        count: row.get::<i64, _>("count") 
    })
    .collect()
}



// 处理个人资料编辑页面请求
pub async fn edit_profile_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 获取用户信息
    let user_detail = get_user_detail(pool, &user.as_ref().unwrap().username).await;
    
    match user_detail {
        Some(user_detail) => {
            let template = EditProfileTemplate {
                user: user.clone(),
                user_exists,
                user_detail,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        None => {
            // 用户不存在，重定向到首页
            Redirect::to("/").into_response()
        }
    }
}

// 处理个人资料编辑表单提交
pub async fn edit_profile_post_handler(State(state): State<AppState>, headers: HeaderMap, mut req: axum::extract::Multipart) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 处理表单数据
    let mut bio = String::new();
    let mut avatar_url = String::new();
    let mut avatar_bytes = None;
    let mut avatar_file_name = String::new();
    
    // 处理多部分表单数据
    while let Ok(Some(field)) = req.next_field().await {
        if let Some(name) = field.name() {
            match name {
                "bio" => {
                    if let Ok(value) = field.text().await {
                        bio = value;
                    }
                }
                "avatar_url" => {
                    if let Ok(value) = field.text().await {
                        avatar_url = value;
                    }
                }
                "avatar" => {
                    let file_name = field.file_name().unwrap_or("").to_string();
                    let content_type = field.content_type().unwrap_or(&"");
                    
                    // 验证文件类型
                    let allowed_types = vec!["image/jpeg", "image/png", "image/webp"];
                    if allowed_types.contains(&content_type) {
                        if let Ok(bytes) = field.bytes().await {
                            // 验证文件大小
                            if bytes.len() <= 5 * 1024 * 1024 { // 5MB
                                avatar_bytes = Some(bytes);
                                avatar_file_name = file_name;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    // 处理头像上传
    if let Some(bytes) = avatar_bytes {
        // 确保上传目录存在
        let upload_dir = std::path::Path::new("uploads/avatars");
        if !upload_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(upload_dir) {
                eprintln!("创建上传目录失败: {:?}", e);
            }
        }
        
        // 生成唯一文件名
        let timestamp = chrono::Utc::now().timestamp();
        let file_extension = avatar_file_name.split('.').last().unwrap_or("jpg");
        let unique_file_name = format!("{}_{}.{}", user_id, timestamp, file_extension);
        let file_path = upload_dir.join(&unique_file_name);
        
        // 保存文件
        if let Ok(mut file) = std::fs::File::create(&file_path) {
            if let Err(e) = std::io::Write::write_all(&mut file, &bytes) {
                eprintln!("写入文件失败: {:?}", e);
            } else {
                // 更新头像URL
                avatar_url = format!("/uploads/avatars/{}", unique_file_name);
            }
        } else {
            eprintln!("创建文件失败: {:?}", file_path);
        }
    }
    
    // 更新用户资料
    sqlx::query!(r#"
        UPDATE users
        SET bio = $1, avatar_url = $2
        WHERE id = $3
    "#, bio, avatar_url, user_id)
    .execute(pool)
    .await
    .unwrap();
    
    // 重定向到个人资料页面
    Redirect::to(&format!("/u/{}", user.as_ref().unwrap().username)).into_response()
}

// 处理图片上传
pub async fn upload_project_image(State(state): State<AppState>, Path(project_id): Path<i64>, headers: HeaderMap, mut req: axum::extract::Multipart) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 处理文件上传
    let mut image_url = String::new();
    let image_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_images WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    
    // 处理上传的文件
    while let Some(field) = req.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            let file_name = field.file_name().unwrap_or("").to_string();
            let content_type = field.content_type().unwrap_or(&"").to_string();
            
            // 验证文件类型
            let allowed_types = vec!["image/jpeg", "image/png", "image/webp"];
            if !allowed_types.contains(&content_type.as_str()) {
                return Html("不支持的文件类型，请上传JPG、PNG或WEBP格式的图片").into_response();
            }
            
            // 读取文件内容
            let data = field.bytes().await.unwrap();
            
            // 验证文件大小（不超过5MB）
            if data.len() > 5 * 1024 * 1024 {
                return Html("文件大小超过限制，请上传不超过5MB的图片").into_response();
            }
            
            // 生成唯一的文件名
            use rand::Rng;
            use std::time::SystemTime;
            let random_id = rand::thread_rng().gen_range(1..1000000);
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let extension = file_name.split('.').last().unwrap_or("jpg");
            let file_path = format!("uploads/{}_{}.{}", timestamp, random_id, extension);
            
            // 确保uploads目录存在
            std::fs::create_dir_all("uploads").unwrap();
            
            // 保存文件
            std::fs::write(&file_path, data).unwrap();
            
            // 生成图片URL
            image_url = format!("/{}", file_path);
            break;
        }
    }
    
    if image_url.is_empty() {
        return Html("请选择要上传的图片").into_response();
    }
    
    let is_main = image_count == 0;
    
    // 保存图片信息到数据库
    sqlx::query(r#"
        INSERT INTO project_images (project_id, user_id, image_url, is_main)
        VALUES ($1, $2, $3, $4)
    "#)
    .bind(project_id)
    .bind(user_id)
    .bind(image_url)
    .bind(is_main)
    .execute(pool)
    .await
    .unwrap();
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 处理设置主图
pub async fn set_main_image(State(state): State<AppState>, Path((project_id, image_id)): Path<(i64, i64)>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 将所有图片的is_main设置为false
    sqlx::query("UPDATE project_images SET is_main = false WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .unwrap();
    
    // 将指定图片的is_main设置为true
    sqlx::query("UPDATE project_images SET is_main = true WHERE id = $1 AND project_id = $2")
        .bind(image_id)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .unwrap();
    
    // 提交事务
    tx.commit().await.unwrap();
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 处理删除图片
pub async fn delete_project_image(State(state): State<AppState>, Path((project_id, image_id)): Path<(i64, i64)>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    
    // 获取图片信息，以便删除物理文件
    let image = sqlx::query!(r#"
        SELECT image_url
        FROM project_images
        WHERE id = $1 AND project_id = $2
    "#, image_id, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    // 删除图片记录
    sqlx::query("DELETE FROM project_images WHERE id = $1 AND project_id = $2")
        .bind(image_id)
        .bind(project_id)
        .execute(pool)
        .await
        .unwrap();
    
    // 删除物理文件
    let mut image_url = "".to_string();
    if let Some(img) = image {
        image_url = img.image_url.clone();
        let image_path = img.image_url.trim_start_matches('/');
        if !image_path.is_empty() && std::path::Path::new(image_path).exists() {
            std::fs::remove_file(image_path).unwrap_or(());
        }
    }
    
    // 记录审计日志
    log_audit_action(
        pool,
        user.as_ref().unwrap().id,
        "delete",
        "project_image",
        image_id,
        &format!("删除项目图片: project_id={}, image_url={}", project_id, image_url)
    ).await;
    
    // 检查是否还有图片，如果有，将第一张设为主图
    let image_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_images WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    
    if image_count > 0 {
        sqlx::query(r#"
            UPDATE project_images 
            SET is_main = true 
            WHERE id = (
                SELECT id 
                FROM project_images 
                WHERE project_id = $1 
                ORDER BY created_at ASC 
                LIMIT 1
            )
        "#)
            .bind(project_id)
            .execute(pool)
            .await
            .unwrap();
    }
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 处理添加管理者
pub async fn add_manager_handler(State(state): State<AppState>, Path((project_id, user_id)): Path<(i64, i64)>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let current_user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = current_user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let current_user_id = current_user.as_ref().unwrap().id;
    
    // 检查当前用户是否是项目创建者
    let project_creator = sqlx::query_scalar!(r#"
        SELECT user_id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if let Some(creator_id) = project_creator {
        if creator_id != current_user_id {
            return Redirect::to(&format!("/projects/{}", project_id)).into_response();
        }
    }
    
    // 将用户添加为管理者
    sqlx::query!(r#"
        INSERT INTO project_participants (project_id, user_id, role, message)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (project_id, user_id) DO UPDATE SET role = $3
    "#, project_id, user_id, "manager", "被添加为项目管理者")
    .execute(pool)
    .await
    .unwrap();
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 处理项目归档
pub async fn archive_project_handler(State(state): State<AppState>, Path((project_id, archive)): Path<(i64, bool)>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 检查用户是否有项目编辑权限
    if !has_project_edit_permission(pool, project_id, user_id).await {
        return Redirect::to("/").into_response();
    }
    
    // 更新项目归档状态
    sqlx::query!(r#"
        UPDATE projects
        SET is_archived = $1
        WHERE id = $2
    "#, archive, project_id)
    .execute(pool)
    .await
    .unwrap();
    
    // 记录审计日志
    log_audit_action(
        pool,
        user_id,
        if archive { "archive" } else { "unarchive" },
        "project",
        project_id,
        &format!("{}{}项目: id={}", if archive { "归档" } else { "取消归档" }, " ", project_id)
    ).await;
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 处理删除项目
pub async fn delete_project_handler(State(state): State<AppState>, Path(project_id): Path<i64>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，返回错误
    if !user_exists {
        let response = serde_json::json!({"status": "error", "message": "用户未登录"});
        return (axum::http::StatusCode::UNAUTHORIZED, Json(response)).into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 检查用户是否是项目创建者
    let project_creator = sqlx::query_scalar!(r#"
        SELECT user_id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if let Some(creator_id) = project_creator {
        if creator_id != user_id {
            let response = serde_json::json!({"status": "error", "message": "只有项目创建者可以删除项目"});
            return (axum::http::StatusCode::FORBIDDEN, Json(response)).into_response();
        }
    } else {
        let response = serde_json::json!({"status": "error", "message": "项目不存在"});
        return (axum::http::StatusCode::NOT_FOUND, Json(response)).into_response();
    }
    
    // 开始事务
    let mut tx = pool.begin().await.unwrap();
    
    // 删除项目的所有相关数据
    // 删除项目参与者
    sqlx::query!(r#"
        DELETE FROM project_participants
        WHERE project_id = $1
    "#, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除项目的评论
    sqlx::query!(r#"
        DELETE FROM comments
        WHERE project_id = $1
    "#, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除项目的创意
    let ideas = sqlx::query!(r#"
        SELECT id
        FROM ideas
        WHERE project_id = $1
    "#, project_id)
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    
    for idea in ideas {
        // 删除创意的投票
        sqlx::query!(r#"
            DELETE FROM idea_votes
            WHERE idea_id = $1
        "#, idea.id)
        .execute(&mut *tx)
        .await
        .unwrap();
        
        // 删除创意的评论
        sqlx::query!(r#"
            DELETE FROM comments
            WHERE idea_id = $1
        "#, idea.id)
        .execute(&mut *tx)
        .await
        .unwrap();
        
        // 删除需求与创意的关联关系
        sqlx::query!(r#"
            DELETE FROM need_idea_relations
            WHERE idea_id = $1
        "#, idea.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    
    // 删除项目的创意
    sqlx::query!(r#"
        DELETE FROM ideas
        WHERE project_id = $1
    "#, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除项目的需求
    let needs = sqlx::query!(r#"
        SELECT id
        FROM project_needs
        WHERE project_id = $1
    "#, project_id)
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    
    for need in needs {
        // 删除需求与创意的关联关系
        sqlx::query!(r#"
            DELETE FROM need_idea_relations
            WHERE need_id = $1
        "#, need.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    
    // 删除项目的需求
    sqlx::query!(r#"
        DELETE FROM project_needs
        WHERE project_id = $1
    "#, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除项目的进度更新
    sqlx::query!(r#"
        DELETE FROM project_progress
        WHERE project_id = $1
    "#, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除项目的图片
    let images = sqlx::query!(r#"
        SELECT image_url
        FROM project_images
        WHERE project_id = $1
    "#, project_id)
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    
    // 提交事务
    tx.commit().await.unwrap();
    
    // 删除物理文件
    for image in images {
        let image_path = image.image_url.trim_start_matches('/');
        if !image_path.is_empty() && std::path::Path::new(image_path).exists() {
            std::fs::remove_file(image_path).unwrap_or(());
        }
    }
    
    // 再次开始事务删除项目本身
    let mut tx = pool.begin().await.unwrap();
    
    // 删除项目的标签
    sqlx::query!(r#"
        DELETE FROM project_tags
        WHERE project_id = $1
    "#, project_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    
    // 删除项目
    sqlx::query!(r#"
        DELETE FROM projects
        WHERE id = $1
    "#, project_id)
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
        "project",
        project_id,
        &format!("删除项目: project_id={}", project_id)
    ).await;
    
    // 返回成功响应
    let response = serde_json::json!({"status": "success", "message": "项目删除成功"});
    Json(response).into_response()
}

// 处理删除项目参与者
pub async fn remove_participant_handler(State(state): State<AppState>, Path((project_id, user_id)): Path<(i64, i64)>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let current_user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = current_user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let current_user_id = current_user.as_ref().unwrap().id;
    
    // 检查当前用户是否是项目管理员（创建者或管理者）
    let project_creator = sqlx::query_scalar!(r#"
        SELECT user_id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    let is_admin = match project_creator {
        Some(creator_id) => {
            if creator_id == current_user_id {
                true
            } else {
                // 检查是否是项目管理者
                let participant_role = sqlx::query_scalar!(r#"
                    SELECT role
                    FROM project_participants
                    WHERE project_id = $1 AND user_id = $2
                "#, project_id, current_user_id)
                .fetch_optional(pool)
                .await
                .unwrap();
                participant_role == Some("manager".to_string())
            }
        },
        None => false,
    };
    
    // 只有管理员可以删除参与者
    if !is_admin {
        return Redirect::to(&format!("/projects/{}", project_id)).into_response();
    }
    
    // 不能删除项目创建者
    if let Some(creator_id) = project_creator {
        if creator_id == user_id {
            return Redirect::to(&format!("/projects/{}", project_id)).into_response();
        }
    }
    
    // 删除项目参与者
    sqlx::query!(r#"
        DELETE FROM project_participants
        WHERE project_id = $1 AND user_id = $2
    "#, project_id, user_id)
    .execute(pool)
    .await
    .unwrap();
    
    // 记录审计日志
    log_audit_action(
        pool,
        current_user_id,
        "delete",
        "project_participant",
        user_id,
        &format!("删除项目参与者: project_id={}, participant_id={}", project_id, user_id)
    ).await;
    
    // 重定向到项目详情页
    Redirect::to(&format!("/projects/{}", project_id)).into_response()
}

// 需求详情结构
#[derive(Debug)]
struct NeedDetail {
    id: i64,
    project_id: i64,
    title: String,
    description: String,
    priority: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// 需求详情模板
#[derive(Template)]
#[template(path = "need_detail.html")]
struct NeedDetailTemplate {
    need: NeedDetail,
    project_title: String,
    related_ideas: Vec<ProjectIdea>,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 处理需求详情页面请求
pub async fn need_detail_handler(Path((project_id, need_id)): Path<(i64, i64)>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 获取需求详情
    let need = sqlx::query_as!(NeedDetail, r#"
        SELECT id, project_id, title, description, priority, status, created_at, updated_at
        FROM project_needs
        WHERE id = $1 AND project_id = $2
    "#, need_id, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if need.is_none() {
        return Html("需求不存在").into_response();
    }
    
    let need = need.unwrap();
    
    // 获取项目标题
    let project = sqlx::query!(r#"
        SELECT title
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if project.is_none() {
        return Html("项目不存在").into_response();
    }
    
    let project_title = project.unwrap().title;
    
    // 获取与需求相关的创意
    let related_ideas = match sqlx::query(r#"
        SELECT i.id, i.project_id, i.title, i.content, i.idea_type, i.feasibility_score, i.estimated_cost, u.username, u.avatar_url, i.created_at
        FROM ideas i
        JOIN users u ON i.user_id = u.id
        JOIN need_idea_relations nir ON i.id = nir.idea_id
        WHERE nir.need_id = $1
        ORDER BY i.created_at DESC
    "#)
    .bind(need_id)
    .fetch_all(pool)
    .await {
        Ok(rows) => rows.into_iter()
            .map(|row| ProjectIdea {
                id: row.get::<i64, _>("id"),
                project_id: row.get::<i64, _>("project_id"),
                title: row.get::<String, _>("title"),
                content: row.get::<String, _>("content"),
                idea_type: row.get::<String, _>("idea_type"),
                feasibility_score: row.get::<i32, _>("feasibility_score"),
                estimated_cost: row.get::<String, _>("estimated_cost"),
                username: row.get::<String, _>("username"),
                avatar_url: row.get::<Option<String>, _>("avatar_url").unwrap_or("https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=user%20avatar%20profile%20picture&image_size=square".to_string()),
                created_at: row.get::<DateTime<Utc>, _>("created_at"),
            })
            .collect(),
        Err(_) => Vec::new(), // 如果表不存在，返回空列表
    };
    
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    let template = NeedDetailTemplate {
        need,
        project_title,
        related_ideas,
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

async fn get_user_detail(pool: &PgPool, username: &str) -> Option<UserDetail> {
    sqlx::query!(r#"
        SELECT id, username, bio, avatar_url, created_at
        FROM users
        WHERE username = $1
    "#, username)
    .fetch_optional(pool)
    .await
    .unwrap()
    .map(|u| UserDetail {
        id: u.id,
        username: u.username,
        bio: u.bio,
        avatar_url: u.avatar_url,
        created_at: u.created_at,
    })
}

async fn get_user_activities(pool: &PgPool, user_id: i64) -> Vec<UserActivity> {
    sqlx::query!(r#"
        SELECT id, activity_type, entity_type, action, details, created_at
        FROM user_activity_logs
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
    "#, user_id as i32)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|a| UserActivity {
        id: a.id as i64,
        activity_type: a.activity_type,
        entity_type: a.entity_type,
        action: a.action,
        details: a.details.unwrap_or_else(|| "".to_string()),
        created_at: a.created_at.map(|naive| chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc)).unwrap_or_else(|| chrono::Utc::now()),
    })
    .collect()
}

// 检查用户在项目中的角色
async fn get_user_project_role(pool: &PgPool, project_id: i64, user_id: i64) -> Option<String> {
    sqlx::query_scalar!(r#"
        SELECT role
        FROM project_participants
        WHERE project_id = $1 AND user_id = $2
    "#, project_id, user_id)
    .fetch_optional(pool)
    .await
    .unwrap()
}

// 检查用户是否有项目编辑权限
pub async fn has_project_edit_permission(pool: &PgPool, project_id: i64, user_id: i64) -> bool {
    // 检查用户是否是项目创建者
    let project_creator = sqlx::query_scalar!(r#"
        SELECT user_id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if let Some(creator_id) = project_creator {
        if creator_id == user_id {
            return true;
        }
    }
    
    // 检查用户是否是项目管理者
    let role = get_user_project_role(pool, project_id, user_id).await;
    if let Some(role) = role {
        return role == "creator" || role == "manager";
    }
    
    false
}

async fn get_user_stats(pool: &PgPool, user_id: i64) -> UserStats {
    // 统计用户发起的项目数
    let projects_count = sqlx::query!(r#"
        SELECT COUNT(*) as count
        FROM projects
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .count;

    // 统计用户提交的创意数
    let ideas_count = sqlx::query!(r#"
        SELECT COUNT(*) as count
        FROM ideas
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .count;

    // 统计用户参与的项目数
    let participations_count = sqlx::query!(r#"
        SELECT COUNT(*) as count
        FROM project_participants
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .count;

    // 统计用户的评论数
    let comments_count = sqlx::query!(r#"
        SELECT COUNT(*) as count
        FROM comments
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .count;

    // 统计用户的投票数
    let votes_count = sqlx::query!(r#"
        SELECT COUNT(*) as count
        FROM idea_votes
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .count;

    // 计算贡献度得分
    // 权重：项目 * 10，创意 * 5，评论 * 2，投票 * 1，参与 * 3
    let contribution_score = 
        (projects_count.unwrap_or(0) * 10) + 
        (ideas_count.unwrap_or(0) * 5) + 
        (comments_count.unwrap_or(0) * 2) + 
        (votes_count.unwrap_or(0) * 1) + 
        (participations_count.unwrap_or(0) * 3);

    UserStats {
        projects_count: projects_count.unwrap_or(0),
        ideas_count: ideas_count.unwrap_or(0),
        participations_count: participations_count.unwrap_or(0),
        contribution_score,
    }
}

async fn get_user_projects(pool: &PgPool, user_id: i64) -> Vec<UserProject> {
    let projects = sqlx::query!(r#"
        SELECT id, title, summary, stage, created_at
        FROM projects
        WHERE user_id = $1
        ORDER BY created_at DESC
    "#, user_id)
    .fetch_all(pool)
    .await
    .unwrap();

    let mut user_projects = Vec::new();
    for project in projects {
        // 查询项目标签
        let tags = sqlx::query!(r#"
            SELECT tag
            FROM project_tags
            WHERE project_id = $1
        "#, project.id)
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.tag)
        .collect();

        user_projects.push(UserProject {
            id: project.id,
            title: project.title,
            summary: project.summary,
            stage: project.stage,
            created_at: project.created_at,
            tags,
        });
    }
    user_projects
}

async fn get_participated_projects(pool: &PgPool, user_id: i64) -> Vec<ParticipatedProject> {
    let participations = sqlx::query!(r#"
        SELECT p.id, p.title, p.summary, pp.role, pp.created_at as joined_at
        FROM project_participants pp
        JOIN projects p ON pp.project_id = p.id
        WHERE pp.user_id = $1
        ORDER BY pp.created_at DESC
    "#, user_id)
    .fetch_all(pool)
    .await
    .unwrap();

    let mut participated_projects = Vec::new();
    for participation in participations {
        // 查询项目标签
        let tags = sqlx::query!(r#"
            SELECT tag
            FROM project_tags
            WHERE project_id = $1
        "#, participation.id)
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.tag)
        .collect();

        participated_projects.push(ParticipatedProject {
            id: participation.id,
            title: participation.title,
            summary: participation.summary,
            role: participation.role,
            joined_at: participation.joined_at,
            tags,
        });
    }
    participated_projects
}

async fn get_user_ideas(pool: &PgPool, user_id: i64) -> Vec<UserIdea> {
    sqlx::query!(r#"
        SELECT id, title, content, idea_type, project_id, created_at
        FROM ideas
        WHERE user_id = $1
        ORDER BY created_at DESC
    "#, user_id)
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|i| UserIdea {
        id: i.id,
        title: i.title,
        content: i.content,
        idea_type: i.idea_type,
        project_id: i.project_id,
        created_at: i.created_at,
    })
    .collect()
}

async fn get_all_tags(pool: &PgPool) -> Vec<PopularTag> {
    sqlx::query!(r#"
        SELECT tag, COUNT(*) as count
        FROM project_tags
        GROUP BY tag
        ORDER BY count DESC
    "#)
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|t| PopularTag { name: t.tag, count: t.count.unwrap_or(0) })
    .collect()
}

// 项目模板相关的模板结构体

// 项目模板列表模板
#[derive(Template)]
#[template(path = "templates.html")]
pub struct TemplatesTemplate {
    pub templates: Vec<ProjectTemplate>,
    pub user: Option<UserInfo>,
    pub user_exists: bool,
}

// 项目模板表单模板
#[derive(Template)]
#[template(path = "template_form.html")]
pub struct TemplateFormTemplate {
    pub title: String,
    pub action: String,
    pub cancel_url: String,
    pub error: String,
    pub template: TemplateFormData,
    pub user: Option<UserInfo>,
    pub user_exists: bool,
}

// 项目模板表单数据
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateFormData {
    pub name: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub tags: String,
}

// 创建项目模板请求
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub is_public: bool,
    pub tags: String,
}

// 基于模板创建项目请求
#[derive(Debug, Deserialize)]
pub struct CreateProjectFromTemplateRequest {
    pub template_id: i64,
    pub title: String,
    pub summary: String,
    pub tags: String,
}

// 项目模板相关处理函数

// 处理项目模板列表页面请求
pub async fn templates_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 获取所有公开模板和用户自己的模板
    let mut templates = ProjectTemplate::get_all_public(&state.db).await.unwrap();
    
    if let Some(user_info) = &user {
        let user_templates = ProjectTemplate::get_by_user(&state.db, user_info.id).await.unwrap();
        // 过滤掉已在公开模板中的用户模板
        let user_only_templates: Vec<ProjectTemplate> = user_templates
            .into_iter()
            .filter(|t| !t.is_public)
            .collect();
        templates.extend(user_only_templates);
    }
    
    let template = TemplatesTemplate {
        templates,
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理项目模板创建页面请求
pub async fn create_template_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let template = TemplateFormTemplate {
        title: "创建项目模板".to_string(),
        action: "/templates/create".to_string(),
        cancel_url: "/templates".to_string(),
        error: "".to_string(),
        template: TemplateFormData {
                name: "".to_string(),
                description: "".to_string(),
                category: "".to_string(),
                stage: "planning".to_string(),
                location: "".to_string(),
                budget_range: "".to_string(),
                existing_resources: "".to_string(),
                needed_resources: "".to_string(),
                tags: "".to_string(),
            },
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理项目模板创建表单提交
pub async fn create_template_post_handler(State(state): State<AppState>, headers: HeaderMap, Form(form): Form<CreateTemplateRequest>) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let user_id = user.as_ref().unwrap().id;
    
    // 处理标签
    let tags: Vec<String> = form.tags
        .split(",")
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    
    // 保存表单数据用于错误处理
    let form_data = TemplateFormData {
        name: form.name.clone(),
        description: form.description.clone(),
        category: form.category.clone(),
        stage: form.stage.clone(),
        location: form.location.clone(),
        budget_range: form.budget_range.clone(),
        existing_resources: form.existing_resources.clone(),
        needed_resources: form.needed_resources.clone(),
        tags: form.tags.clone(),
    };
    
    // 创建新模板
    let new_template = NewProjectTemplate {
        name: form.name,
        description: form.description,
        category: form.category,
        stage: form.stage,
        location: form.location,
        budget_range: form.budget_range,
        existing_resources: form.existing_resources,
        needed_resources: form.needed_resources,
        is_public: form.is_public,
        tags,
    };
    
    match ProjectTemplate::create(&state.db, user_id, new_template).await {
        Ok(_) => {
            // 重定向到模板列表页面
            Redirect::to("/templates").into_response()
        },
        Err(_) => {
            // 显示错误信息
            let template = TemplateFormTemplate {
                title: "创建项目模板".to_string(),
                action: "/templates/create".to_string(),
                cancel_url: "/templates".to_string(),
                error: "创建模板失败，请重试".to_string(),
                template: form_data,
                user,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        }
    }
}

// 处理项目模板编辑页面请求
pub async fn edit_template_handler(Path(template_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let user_id = user.as_ref().unwrap().id;
    
    // 获取模板详情
    match ProjectTemplate::get_by_id(&state.db, template_id).await {
        Ok(Some(template)) => {
            // 检查用户是否是模板创建者
            if template.user_id != user_id {
                return Redirect::to("/templates").into_response();
            }
            
            let tags_str = template.tags.join(", ");
            
            let template = TemplateFormTemplate {
                title: "编辑项目模板".to_string(),
                action: format!("/templates/{}/edit", template_id),
                cancel_url: "/templates".to_string(),
                error: "".to_string(),
                template: TemplateFormData {
                    name: template.name,
                    description: template.description,
                    category: template.category,
                    stage: template.stage,
                    location: template.location,
                    budget_range: template.budget_range,
                    existing_resources: template.existing_resources,
                    needed_resources: template.needed_resources,
                    tags: tags_str,
                },
                user,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        _ => {
            // 模板不存在，重定向到模板列表页面
            Redirect::to("/templates").into_response()
        }
    }
}

// 处理项目模板编辑表单提交
pub async fn edit_template_post_handler(Path(template_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap, Form(form): Form<CreateTemplateRequest>) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let user_id = user.as_ref().unwrap().id;
    
    // 检查用户是否是模板创建者
    match ProjectTemplate::get_by_id(&state.db, template_id).await {
        Ok(Some(template)) => {
            if template.user_id != user_id {
                return Redirect::to("/templates").into_response();
            }
            
            // 处理标签
            let tags: Vec<String> = form.tags
                .split(",")
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect();
            
            // 保存表单数据用于错误处理
            let form_data = TemplateFormData {
                name: form.name.clone(),
                description: form.description.clone(),
                category: form.category.clone(),
                stage: form.stage.clone(),
                location: form.location.clone(),
                budget_range: form.budget_range.clone(),
                existing_resources: form.existing_resources.clone(),
                needed_resources: form.needed_resources.clone(),
                tags: form.tags.clone(),
            };
            
            // 更新模板
            let updated_template = NewProjectTemplate {
                name: form.name,
                description: form.description,
                category: form.category,
                stage: form.stage,
                location: form.location,
                budget_range: form.budget_range,
                existing_resources: form.existing_resources,
                needed_resources: form.needed_resources,
                is_public: form.is_public,
                tags,
            };
            
            match ProjectTemplate::update(&state.db, template_id, updated_template).await {
                Ok(_) => {
                    // 重定向到模板列表页面
                    Redirect::to("/templates").into_response()
                },
                Err(_) => {
                    // 显示错误信息
                    let template = TemplateFormTemplate {
                        title: "编辑项目模板".to_string(),
                        action: format!("/templates/{}/edit", template_id),
                        cancel_url: "/templates".to_string(),
                        error: "更新模板失败，请重试".to_string(),
                        template: form_data,
                        user,
                        user_exists,
                    };
                    
                    Html(template.render().unwrap()).into_response()
                }
            }
        },
        _ => {
            // 模板不存在，重定向到模板列表页面
            Redirect::to("/templates").into_response()
        }
    }
}

// 处理项目模板删除请求
pub async fn delete_template_handler(Path(template_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，返回错误
    if !user_exists {
        let response = serde_json::json!({"status": "error", "message": "用户未登录"});
        return (axum::http::StatusCode::UNAUTHORIZED, Json(response)).into_response();
    }
    
    let user_id = user.as_ref().unwrap().id;
    
    // 检查用户是否是模板创建者
    match ProjectTemplate::get_by_id(&state.db, template_id).await {
        Ok(Some(template)) => {
            if template.user_id != user_id {
                let response = serde_json::json!({"status": "error", "message": "只有模板创建者可以删除模板"});
                return (axum::http::StatusCode::FORBIDDEN, Json(response)).into_response();
            }
            
            // 删除模板
            match ProjectTemplate::delete(&state.db, template_id).await {
                Ok(_) => {
                    let response = serde_json::json!({"status": "success", "message": "模板删除成功"});
                    (axum::http::StatusCode::OK, Json(response)).into_response()
                },
                Err(_) => {
                    let response = serde_json::json!({"status": "error", "message": "删除模板失败，请重试"});
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                }
            }
        },
        _ => {
            let response = serde_json::json!({"status": "error", "message": "模板不存在"});
            (axum::http::StatusCode::NOT_FOUND, Json(response)).into_response()
        }
    }
}

// 处理基于模板创建项目的页面请求
pub async fn create_project_from_template_handler(Path(template_id): Path<i64>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    // 获取模板详情
    match ProjectTemplate::get_by_id(&state.db, template_id).await {
        Ok(Some(template)) => {
            let tags_str = template.tags.join(", ");
            
            let template = ProjectFormTemplate {
                title: "基于模板创建项目".to_string(),
                action: "/projects/create-from-template".to_string(),
                cancel_url: "/templates".to_string(),
                error: "".to_string(),
                project: ProjectFormData {
                    title: format!("{}", template.name),
                    summary: "".to_string(),
                    description: template.description,
                    stage: template.stage,
                    tags: tags_str,
                },
                user,
                user_exists,
            };
            
            Html(template.render().unwrap()).into_response()
        },
        _ => {
            // 模板不存在，重定向到模板列表页面
            Redirect::to("/templates").into_response()
        }
    }
}

// 处理基于模板创建项目的表单提交
pub async fn create_project_from_template_post_handler(State(state): State<AppState>, headers: HeaderMap, Form(form): Form<CreateProjectFromTemplateRequest>) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 获取模板详情
    match ProjectTemplate::get_by_id(&state.db, form.template_id).await {
        Ok(Some(template)) => {
            // 开始事务
            let mut tx = pool.begin().await.unwrap();
            
            // 创建项目
            let project = sqlx::query!(r#"
                INSERT INTO projects (title, summary, description, stage, user_id, slug, category, location, budget_range, existing_resources, needed_resources)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING id
            "#, form.title, form.summary, template.description, template.stage, user_id, form.title.replace(' ', "-").to_lowercase(), template.category, template.location, template.budget_range, template.existing_resources, template.needed_resources)
            .fetch_one(&mut *tx)
            .await;
            
            match project {
                Ok(project) => {
                    // 处理标签
                    let tags = form.tags.split(",").map(|tag| tag.trim()).filter(|tag| !tag.is_empty());
                    for tag in tags {
                        // 直接在project_tags表中插入标签
                        sqlx::query!(r#"
                            INSERT INTO project_tags (project_id, tag)
                            VALUES ($1, $2)
                            ON CONFLICT DO NOTHING
                        "#, project.id, tag)
                        .execute(&mut *tx)
                        .await
                        .unwrap();
                    }
                    
                    // 添加项目创建进度更新
                    sqlx::query!(r#"
                        INSERT INTO project_progress (project_id, user_id, content, created_at)
                        VALUES ($1, $2, $3, now())
                    "#, project.id, user_id, format!("项目 '{}' 创建成功（基于模板）", form.title))
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                    
                    // 添加创建者为项目参与者，角色为creator
                    sqlx::query!(r#"
                        INSERT INTO project_participants (project_id, user_id, role, message)
                        VALUES ($1, $2, $3, $4)
                        ON CONFLICT (project_id, user_id) DO NOTHING
                    "#, project.id, user_id, "creator", "项目创建者")
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                    
                    // 提交事务
                    tx.commit().await.unwrap();
                    
                    // 重定向到项目详情页
                    Redirect::to(&format!("/projects/{}", project.id)).into_response()
                },
                Err(_) => {
                    // 回滚事务
                    tx.rollback().await.unwrap();
                    
                    // 显示错误信息
                    let template = ProjectFormTemplate {
                        title: "基于模板创建项目".to_string(),
                        action: "/projects/create-from-template".to_string(),
                        cancel_url: "/templates".to_string(),
                        error: "创建项目失败，请重试".to_string(),
                        project: ProjectFormData {
                            title: form.title,
                            summary: form.summary,
                            description: "".to_string(),
                            stage: "planning".to_string(),
                            tags: form.tags,
                        },
                        user,
                        user_exists,
                    };
                    
                    Html(template.render().unwrap()).into_response()
                }
            }
        },
        _ => {
            // 模板不存在，重定向到模板列表页面
            Redirect::to("/templates").into_response()
        }
    }
}

async fn get_projects_by_tag(pool: &PgPool, tag_name: &str) -> Vec<FeaturedProject> {
    let projects = sqlx::query!(r#"
        SELECT p.id, p.title, p.summary, u.username, p.created_at, p.stage
        FROM projects p
        JOIN project_tags pt ON p.id = pt.project_id
        JOIN users u ON p.user_id = u.id
        WHERE pt.tag = $1 AND p.is_public = true
        ORDER BY p.created_at DESC
    "#, tag_name)
    .fetch_all(pool)
    .await
    .unwrap();

    let mut featured_projects = Vec::new();
    for project in projects {
        // 查询项目标签
        let tags = sqlx::query!(r#"
            SELECT tag
            FROM project_tags
            WHERE project_id = $1
        "#, project.id)
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.tag)
        .collect();

        featured_projects.push(FeaturedProject {
            id: project.id,
            title: project.title,
            summary: project.summary,
            username: project.username,
            created_at: project.created_at,
            stage: project.stage,
            tags,
        });
    }
    featured_projects
}

// 参与项目处理函数
pub async fn join_project_handler(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    let pool = &state.pool;
    let user_id = user.as_ref().unwrap().id;
    
    // 检查用户是否已经是参与者
    let existing_participant = sqlx::query!(r#"
        SELECT project_id
        FROM project_participants
        WHERE project_id = $1 AND user_id = $2
    "#, project_id, user_id)
    .fetch_optional(pool)
    .await;
    
    match existing_participant {
        Ok(Some(_)) => {
            // 用户已经是参与者，直接重定向到项目详情页
            Redirect::to(&format!("/projects/{}", project_id)).into_response()
        },
        Ok(None) => {
            // 添加用户为参与者
            let result = sqlx::query!(r#"
                INSERT INTO project_participants (project_id, user_id, role)
                VALUES ($1, $2, 'participant')
            "#, project_id, user_id)
            .execute(pool)
            .await;
            
            match result {
                Ok(_) => {
                    // 成功添加参与者，重定向到项目详情页
                    Redirect::to(&format!("/projects/{}", project_id)).into_response()
                },
                Err(_) => {
                    // 添加失败，重定向到项目详情页
                    Redirect::to(&format!("/projects/{}", project_id)).into_response()
                }
            }
        },
        Err(_) => {
            // 查询失败，重定向到项目详情页
            Redirect::to(&format!("/projects/{}", project_id)).into_response()
        }
    }
}

// 用户搜索参数结构
#[derive(Debug, serde::Deserialize)]
pub struct UserSearchParams {
    search: Option<String>,
}

// 用户管理页面模板数据
#[derive(Template)]
#[template(path = "user_management.html")]
struct UserManagementTemplate {
    users: Vec<UserDetail>,
    search_query: String,
    user: Option<UserInfo>,
    user_exists: bool,
}

// 处理用户管理页面请求
pub async fn user_management_handler(State(state): State<AppState>, headers: HeaderMap, Query(params): Query<UserSearchParams>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 构建搜索查询
    let mut query = String::from(r#"
        SELECT id, username, bio, avatar_url, created_at
        FROM users
    "#);
    
    let mut where_clauses = Vec::new();
    let mut query_params = Vec::new();
    let mut param_index = 1;
    
    // 添加搜索条件
    if let Some(search) = &params.search {
        if !search.is_empty() {
            where_clauses.push(format!("(username ILIKE ${} OR email ILIKE ${})
", param_index, param_index));
            query_params.push(format!("%{search}%"));
            param_index += 1;
        }
    }
    
    // 添加WHERE子句
    if !where_clauses.is_empty() {
        query.push_str("WHERE ");
        query.push_str(&where_clauses.join("AND "));
    }
    
    // 添加排序
    query.push_str("ORDER BY created_at DESC");
    
    // 执行查询
    let mut sql_query = sqlx::query(&query);
    for param in query_params {
        sql_query = sql_query.bind(param);
    }
    
    let users = sql_query
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| UserDetail {
            id: row.get(0),
            username: row.get(1),
            bio: row.get(2),
            avatar_url: row.get(3),
            created_at: row.get(4),
        })
        .collect();
    
    // 提取当前用户信息
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    let template = UserManagementTemplate {
        users,
        search_query: params.search.unwrap_or_default(),
        user,
        user_exists,
    };
    
    Html(template.render().unwrap()).into_response()
}

// 处理用户搜索API请求
pub async fn search_users_api(State(state): State<AppState>, Query(params): Query<UserSearchParams>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 构建搜索查询
    let mut query = String::from(r#"
        SELECT id, username, email, bio, avatar_url, created_at
        FROM users
    "#);
    
    let mut where_clauses = Vec::new();
    let mut query_params = Vec::new();
    let mut param_index = 1;
    
    // 添加搜索条件
    if let Some(search) = &params.search {
        if !search.is_empty() {
            where_clauses.push(format!("(username ILIKE ${} OR email ILIKE ${})
", param_index, param_index));
            query_params.push(format!("%{search}%"));
            param_index += 1;
        }
    }
    
    // 添加WHERE子句
    if !where_clauses.is_empty() {
        query.push_str("WHERE ");
        query.push_str(&where_clauses.join("AND "));
    }
    
    // 添加排序和限制
    query.push_str("ORDER BY created_at DESC LIMIT 50");
    
    // 执行查询
    let mut sql_query = sqlx::query_as::<_, crate::auth::models::User>(&query);
    for param in query_params {
        sql_query = sql_query.bind(param);
    }
    
    let users = sql_query
        .fetch_all(pool)
        .await
        .unwrap();
    
    // 转换为UserResponse
    let user_responses: Vec<crate::auth::models::UserResponse> = users.into_iter().map(|user| user.into()).collect();
    
    Json(user_responses).into_response()
}

// 贡献度统计API响应结构
#[derive(Debug, Serialize, Deserialize)]
struct ContributionStats {
    user_id: i64,
    username: String,
    contribution_score: i64,
    projects_count: i64,
    ideas_count: i64,
    comments_count: i64,
    votes_count: i64,
    participations_count: i64,
}

// 处理用户贡献度统计API
pub async fn get_user_contributions_api(State(state): State<AppState>, Path(user_id): Path<i64>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 检查用户是否存在
    let user = sqlx::query!(r#"
        SELECT id, username
        FROM users
        WHERE id = $1
    "#, user_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if user.is_none() {
        return Json(serde_json::json!({
            "error": "User not found"
        })).into_response();
    }
    
    let user = user.unwrap();
    
    // 统计用户发起的项目数
    let projects_count = sqlx::query_scalar!(r#"
        SELECT COUNT(*) as count
        FROM projects
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .unwrap_or(0);
    
    // 统计用户提交的创意数
    let ideas_count = sqlx::query_scalar!(r#"
        SELECT COUNT(*) as count
        FROM ideas
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .unwrap_or(0);
    
    // 统计用户参与的项目数
    let participations_count = sqlx::query_scalar!(r#"
        SELECT COUNT(*) as count
        FROM project_participants
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .unwrap_or(0);
    
    // 统计用户的评论数
    let comments_count = sqlx::query_scalar!(r#"
        SELECT COUNT(*) as count
        FROM comments
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .unwrap_or(0);
    
    // 统计用户的投票数
    let votes_count = sqlx::query_scalar!(r#"
        SELECT COUNT(*) as count
        FROM idea_votes
        WHERE user_id = $1
    "#, user_id)
    .fetch_one(pool)
    .await
    .unwrap()
    .unwrap_or(0);
    
    // 计算贡献度得分
    let contribution_score = 
        (projects_count * 10) + 
        (ideas_count * 5) + 
        (comments_count * 2) + 
        (votes_count * 1) + 
        (participations_count * 3);
    
    let stats = ContributionStats {
        user_id: user.id,
        username: user.username,
        contribution_score,
        projects_count,
        ideas_count,
        comments_count,
        votes_count,
        participations_count,
    };
    
    Json(stats).into_response()
}

// 项目参与者贡献度API响应结构
#[derive(Debug, Serialize, Deserialize)]
struct ProjectParticipantContribution {
    user_id: i64,
    username: String,
    avatar_url: String,
    role: String,
    contribution_score: i64,
    ideas_count: i64,
    comments_count: i64,
    votes_count: i64,
}

// 处理项目参与者贡献度统计API
pub async fn get_project_participants_contributions_api(State(state): State<AppState>, Path(project_id): Path<i64>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 检查项目是否存在
    let project = sqlx::query_scalar!(r#"
        SELECT id
        FROM projects
        WHERE id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if project.is_none() {
        return Json(serde_json::json!({
            "error": "Project not found"
        })).into_response();
    }
    
    let participants = sqlx::query(r#"
        SELECT u.id, u.username, u.avatar_url, pp.role
        FROM project_participants pp
        JOIN users u ON pp.user_id = u.id
        WHERE pp.project_id = $1
    "#)
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    let mut result = Vec::new();
    for participant in participants {
        let user_id = participant.get::<i64, _>("id");
        
        // 计算参与者在该项目中的贡献度
        let ideas_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as count
            FROM ideas
            WHERE project_id = $1 AND user_id = $2
        "#, project_id, user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        let comments_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as count
            FROM comments
            WHERE project_id = $1 AND user_id = $2
        "#, project_id, user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        let votes_count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as count
            FROM idea_votes iv
            JOIN ideas i ON iv.idea_id = i.id
            WHERE i.project_id = $1 AND iv.user_id = $2
        "#, project_id, user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
        let contribution_score = (ideas_count * 5) + (comments_count * 2) + (votes_count * 1);
        
        result.push(ProjectParticipantContribution {
            user_id,
            username: participant.get::<String, _>("username"),
            avatar_url: participant.get::<String, _>("avatar_url"),
            role: participant.get::<String, _>("role"),
            contribution_score,
            ideas_count,
            comments_count,
            votes_count,
        });
    }
    
    // 按贡献度排序
    result.sort_by(|a, b| b.contribution_score.cmp(&a.contribution_score));
    
    Json(result).into_response()
}

// 处理项目导出请求
pub async fn export_project_handler(
    State(state): State<AppState>,
    Path((project_id, format)): Path<(i64, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 检查用户是否登录
    let user = if let Some(session) = get_session_from_cookies(&headers, &state).await {
        Some(get_user_info_from_session(&session, &state).await)
    } else {
        None
    };
    let user_exists = user.is_some();
    
    // 如果用户未登录，重定向到登录页面
    if !user_exists {
        return Redirect::to("/login").into_response();
    }
    
    // 检查用户是否有项目导出权限（创建者或管理者）
    let user_id = user.as_ref().unwrap().id;
    if !has_project_edit_permission(pool, project_id, user_id).await {
        return Html("您没有权限导出此项目").into_response();
    }
    
    // 获取项目完整数据
    let export_data = get_export_project_data(pool, project_id).await;
    
    if export_data.is_none() {
        return Html("项目不存在").into_response();
    }
    
    let export_data = export_data.unwrap();
    
    match format.as_str() {
        "json" => export_project_json(export_data).into_response(),
        "csv" => export_project_csv(export_data).into_response(),
        _ => Html("不支持的导出格式").into_response(),
    }
}

// 获取项目完整数据用于导出
async fn get_export_project_data(pool: &PgPool, project_id: i64) -> Option<ExportProject> {
    // 获取项目基本信息
    let project = sqlx::query!(r#"
        SELECT p.id, p.title, p.summary, p.description, u.username, p.created_at, p.stage, p.existing_resources, p.needed_resources
        FROM projects p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1
    "#, project_id)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    match project {
        Some(p) => {
            // 获取项目标签
            let tags = sqlx::query!(r#"
                SELECT tag
                FROM project_tags
                WHERE project_id = $1
            "#, p.id)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect();
            
            // 获取项目进度更新
            let progress_updates = sqlx::query!(r#"
                SELECT pp.content, u.username, pp.created_at, pp.progress_percentage, pp.update_date
                FROM project_progress pp
                JOIN users u ON pp.user_id = u.id
                WHERE pp.project_id = $1
                ORDER BY pp.created_at DESC
            "#, p.id)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|upd| ExportProgressUpdate {
                content: upd.content,
                username: upd.username,
                created_at: upd.created_at,
                progress_percentage: upd.progress_percentage,
                update_date: upd.update_date,
            })
            .collect();
            
            // 获取项目需求
            let needs = sqlx::query!(r#"
                SELECT id, title, description, priority, status
                FROM project_needs
                WHERE project_id = $1
                ORDER BY created_at DESC
            "#, p.id)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|need| ExportProjectNeed {
                id: need.id,
                title: need.title,
                description: need.description,
                priority: need.priority,
                status: need.status,
            })
            .collect();
            
            // 获取项目创意
            let ideas = sqlx::query!(r#"
                SELECT i.id, i.title, i.content, i.idea_type, i.feasibility_score, i.estimated_cost, u.username, i.created_at
                FROM ideas i
                JOIN users u ON i.user_id = u.id
                WHERE i.project_id = $1
                ORDER BY i.created_at DESC
            "#, p.id)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|idea| ExportProjectIdea {
                id: idea.id,
                title: idea.title,
                content: idea.content,
                idea_type: idea.idea_type,
                feasibility_score: idea.feasibility_score,
                estimated_cost: idea.estimated_cost,
                username: idea.username,
                created_at: idea.created_at,
            })
            .collect();
            
            // 获取项目参与者
            let participant_rows = sqlx::query!(r#"
                SELECT u.id, u.username, pp.role
                FROM project_participants pp
                JOIN users u ON pp.user_id = u.id
                WHERE pp.project_id = $1
            "#, p.id)
            .fetch_all(pool)
            .await
            .unwrap();
            
            // 并行计算参与者贡献度
            let mut participants = Vec::new();
            for part in participant_rows {
                // 计算参与者贡献度
                let ideas_count = sqlx::query_scalar!(r#"
                    SELECT COUNT(*) as count
                    FROM ideas
                    WHERE project_id = $1 AND user_id = $2
                "#, p.id, part.id)
                .fetch_one(pool)
                .await
                .unwrap()
                .unwrap_or(0);
                
                let comments_count = sqlx::query_scalar!(r#"
                    SELECT COUNT(*) as count
                    FROM comments
                    WHERE project_id = $1 AND user_id = $2
                "#, p.id, part.id)
                .fetch_one(pool)
                .await
                .unwrap()
                .unwrap_or(0);
                
                let votes_count = sqlx::query_scalar!(r#"
                    SELECT COUNT(*) as count
                    FROM idea_votes iv
                    JOIN ideas i ON iv.idea_id = i.id
                    WHERE i.project_id = $1 AND iv.user_id = $2
                "#, p.id, part.id)
                .fetch_one(pool)
                .await
                .unwrap()
                .unwrap_or(0);
                
                let contribution_score = (ideas_count * 5) + (comments_count * 2) + (votes_count * 1);
                
                participants.push(ExportProjectParticipant {
                    user_id: part.id,
                    username: part.username,
                    role: part.role,
                    contribution_score,
                });
            }
            
            Some(ExportProject {
                id: p.id,
                title: p.title,
                summary: p.summary,
                description: p.description,
                username: p.username,
                created_at: p.created_at,
                stage: p.stage,
                tags,
                existing_resources: p.existing_resources,
                needed_resources: p.needed_resources,
                progress_updates,
                needs,
                ideas,
                participants,
            })
        }
        None => None
    }
}

// 导出项目为JSON格式
fn export_project_json(project: ExportProject) -> impl IntoResponse {
    let json = serde_json::to_string_pretty(&project).unwrap();
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("attachment; filename=project_{}.json", project.id)).unwrap(),
    );
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").unwrap(),
    );
    
    (headers, json).into_response()
}

// 导出项目为CSV格式
fn export_project_csv(project: ExportProject) -> impl IntoResponse {
    let mut csv = String::new();
    
    // 写入项目基本信息
    writeln!(&mut csv, "项目基本信息").unwrap();
    writeln!(&mut csv, "ID,标题,摘要,描述,创建者,创建时间,阶段,标签,现有资源,需要资源").unwrap();
    let tags_str = project.tags.join(",");
    writeln!(&mut csv, "{},{},{},{},{},{},{},{},{},{}",
        project.id,
        escape_csv(&project.title),
        escape_csv(&project.summary),
        escape_csv(&project.description),
        escape_csv(&project.username),
        project.created_at,
        project.stage,
        escape_csv(&tags_str),
        escape_csv(&project.existing_resources),
        escape_csv(&project.needed_resources)
    ).unwrap();
    
    // 写入项目进度更新
    writeln!(&mut csv, "\n项目进度更新").unwrap();
    writeln!(&mut csv, "内容,创建者,创建时间,进度百分比,更新日期").unwrap();
    for update in &project.progress_updates {
        let progress_percentage = update.progress_percentage.map(|p| p.to_string()).unwrap_or("-".to_string());
        let update_date = update.update_date.map(|d| d.to_string()).unwrap_or("-".to_string());
        writeln!(&mut csv, "{},{},{},{},{}",
            escape_csv(&update.content),
            escape_csv(&update.username),
            update.created_at,
            progress_percentage,
            update_date
        ).unwrap();
    }
    
    // 写入项目需求
    writeln!(&mut csv, "\n项目需求").unwrap();
    writeln!(&mut csv, "ID,标题,描述,优先级,状态").unwrap();
    for need in &project.needs {
        writeln!(&mut csv, "{},{},{},{},{}",
            need.id,
            escape_csv(&need.title),
            escape_csv(&need.description),
            need.priority,
            need.status
        ).unwrap();
    }
    
    // 写入项目创意
    writeln!(&mut csv, "\n项目创意").unwrap();
    writeln!(&mut csv, "ID,标题,内容,类型,可行性评分,估计成本,创建者,创建时间").unwrap();
    for idea in &project.ideas {
        writeln!(&mut csv, "{},{},{},{},{},{},{},{}",
            idea.id,
            escape_csv(&idea.title),
            escape_csv(&idea.content),
            idea.idea_type,
            idea.feasibility_score,
            idea.estimated_cost,
            escape_csv(&idea.username),
            idea.created_at
        ).unwrap();
    }
    
    // 写入项目参与者
    writeln!(&mut csv, "\n项目参与者").unwrap();
    writeln!(&mut csv, "用户ID,用户名,角色,贡献度").unwrap();
    for participant in &project.participants {
        writeln!(&mut csv, "{},{},{},{}",
            participant.user_id,
            escape_csv(&participant.username),
            participant.role,
            participant.contribution_score
        ).unwrap();
    }
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("attachment; filename=project_{}.csv", project.id)).unwrap(),
    );
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("text/csv; charset=utf-8").unwrap(),
    );
    
    (headers, csv).into_response()
}

// 转义CSV中的特殊字符
fn escape_csv(s: &str) -> String {
    if s.contains(",") || s.contains("\n") || s.contains('"') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}
