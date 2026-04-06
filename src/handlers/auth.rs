use axum::{extract::{State, Form}, response::{Html, IntoResponse, Redirect}, http::StatusCode};
use askama::Template;
use sqlx::PgPool;
use crate::handlers::AppState;
use crate::auth::hash::{hash_password, verify_password};


use serde::Deserialize;

// 登录请求结构
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

// 注册请求结构
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}



// 处理登录POST请求
pub async fn login_post_handler(State(state): State<AppState>, Form(login_data): Form<LoginRequest>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 查找用户
    let user = sqlx::query!(r#"
        SELECT id, username, password_hash
        FROM users
        WHERE email = $1
    "#, login_data.email)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    match user {
        Some(user) => {
            // 验证密码
            if verify_password(&login_data.password, &user.password_hash).await.unwrap() {
                // 创建会话
                let (token, csrf_token) = state.session_store.create_session(user.id, &user.username).await;
                
                // 设置会话cookie
                let cookie = format!("session_token={}; Path=/; HttpOnly; SameSite=Lax", token);
                let csrf_cookie = format!("csrf_token={}; Path=/; HttpOnly; SameSite=Lax", csrf_token);
                
                // 重定向到首页
                let mut response = Redirect::to("/").into_response();
                response.headers_mut().insert("Set-Cookie", cookie.parse().unwrap());
                response.headers_mut().append("Set-Cookie", csrf_cookie.parse().unwrap());
                response
            } else {
                // 密码错误
                let template = crate::handlers::pages::LoginTemplate {
                    error: "邮箱或密码错误".to_string(),
                    csrf_token: "".to_string(),
                };
                Html(template.render().unwrap()).into_response()
            }
        }
        None => {
            // 用户不存在
            let template = crate::handlers::pages::LoginTemplate {
                error: "邮箱或密码错误".to_string(),
                csrf_token: "".to_string(),
            };
            Html(template.render().unwrap()).into_response()
        }
    }
}

// 处理注册POST请求
pub async fn register_post_handler(State(state): State<AppState>, Form(register_data): Form<RegisterRequest>) -> impl IntoResponse {
    let pool = &state.pool;
    
    // 检查用户名是否已存在
    let existing_username = sqlx::query!(r#"
        SELECT id
        FROM users
        WHERE username = $1
    "#, register_data.username)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if existing_username.is_some() {
        let template = crate::handlers::pages::RegisterTemplate {
            error: "用户名已存在".to_string(),
            csrf_token: "".to_string(),
        };
        return Html(template.render().unwrap()).into_response();
    }
    
    // 检查邮箱是否已被注册
    let existing_user = sqlx::query!(r#"
        SELECT id
        FROM users
        WHERE email = $1
    "#, register_data.email)
    .fetch_optional(pool)
    .await
    .unwrap();
    
    if existing_user.is_some() {
        let template = crate::handlers::pages::RegisterTemplate {
            error: "邮箱已被注册".to_string(),
            csrf_token: "".to_string(),
        };
        return Html(template.render().unwrap()).into_response();
    }
    
    // 哈希密码
    let password_hash = hash_password(&register_data.password).await.unwrap();
    
    // 创建用户
    let user = sqlx::query!(r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username
    "#, register_data.username, register_data.email, password_hash)
    .fetch_one(pool)
    .await
    .unwrap();
    
    // 创建会话
    let (token, csrf_token) = state.session_store.create_session(user.id, &user.username).await;
    
    // 设置会话cookie
    let cookie = format!("session_token={}; Path=/; HttpOnly; SameSite=Lax", token);
    let csrf_cookie = format!("csrf_token={}; Path=/; HttpOnly; SameSite=Lax", csrf_token);
    
    // 重定向到首页
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert("Set-Cookie", cookie.parse().unwrap());
    response.headers_mut().append("Set-Cookie", csrf_cookie.parse().unwrap());
    response
}

// 处理登出请求
pub async fn logout_handler(State(state): State<AppState>) -> impl IntoResponse {
    // 清除会话cookie
    let cookie = "session_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    
    // 重定向到首页
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert("Set-Cookie", cookie.parse().unwrap());
    response
}