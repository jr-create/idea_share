use axum::{http::{Request, Response, StatusCode, Method}, middleware::Next, Extension, body::Body};
use crate::handlers::AppState;
use crate::auth::session::Session;

pub async fn session_middleware(
    app_state: AppState,
    mut req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // 从请求的cookie中提取session_token
    let session_token = req
        .headers()
        .get("cookie")
        .and_then(|cookie| {
            cookie.to_str().ok()
        })
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|part| part.trim().starts_with("session_token="))
                .map(|part| part.trim().trim_start_matches("session_token="))
        });

    // 如果找到session_token，尝试获取会话
    if let Some(token) = session_token {
        if let Some(session) = app_state.session_store.get_session(token).await {
            // 将会话信息添加到请求的extension中
            req.extensions_mut().insert(Extension(session));
        }
    }

    // 继续处理请求
    next.run(req).await
}

pub async fn csrf_middleware(
    app_state: AppState,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // 只对修改数据的请求进行CSRF验证
    let method = req.method();
    if method == &Method::POST || method == &Method::PUT || method == &Method::DELETE || method == &Method::PATCH {
        // 从请求的cookie中提取session_token
        let session_token = req
            .headers()
            .get("cookie")
            .and_then(|cookie| {
                cookie.to_str().ok()
            })
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .find(|part| part.trim().starts_with("session_token="))
                    .map(|part| part.trim().trim_start_matches("session_token="))
            });

        // 从请求的cookie中提取csrf_token
        let csrf_token = req
            .headers()
            .get("cookie")
            .and_then(|cookie| {
                cookie.to_str().ok()
            })
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .find(|part| part.trim().starts_with("csrf_token="))
                    .map(|part| part.trim().trim_start_matches("csrf_token="))
            });

        // 验证CSRF令牌
        if let (Some(session_token), Some(csrf_token)) = (session_token, csrf_token) {
            if let Some(session) = app_state.session_store.get_session(session_token).await {
                if session.csrf_token == csrf_token {
                    // CSRF令牌验证通过，继续处理请求
                    return next.run(req).await;
                }
            }
        }

        // CSRF令牌验证失败，返回403 Forbidden
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::empty())
            .unwrap();
    }

    // 非修改数据的请求，直接通过
    next.run(req).await
}
