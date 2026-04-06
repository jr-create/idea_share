use axum::{Router, routing::{get, post, delete}};
use crate::handlers::{progress_task_need::*, pages::*, auth::*, AppState};

pub fn create_router() -> Router<AppState> {
    Router::new()
        // 页面路由
        .route("/", get(home_handler))
        .route("/u/:username", get(user_profile_handler))
        .route("/tags", get(tags_handler))
        .route("/tags/:tag_name", get(tag_detail_handler))
        .route("/projects", get(projects_handler))
        .route("/projects/:project_id", get(project_detail_handler))
        // 项目创建和编辑路由
        .route("/projects/create", get(create_project_handler))
        .route("/projects/create", post(create_project_post_handler))
        .route("/projects/:project_id/edit", get(edit_project_handler))
        .route("/projects/:project_id/edit", post(edit_project_post_handler))
        // 认证路由
        .route("/login", get(login_handler))
        .route("/login", post(login_post_handler))
        .route("/register", get(register_handler))
        .route("/register", post(register_post_handler))
        .route("/logout", get(logout_handler))
        
        // 个人资料编辑路由
        .route("/dashboard/profile/edit", get(edit_profile_handler))
        .route("/dashboard/profile/edit", post(edit_profile_post_handler))
        
        // 项目相关路由
        .route("/projects/:project_id/progress", post(create_progress))
        .route("/projects/:project_id/progress", get(get_project_progress))
        .route("/projects/:project_id/progress/new", get(create_progress_form))
        .route("/projects/:project_id/needs", post(create_need))
        .route("/projects/:project_id/needs", get(get_project_needs))
        .route("/projects/:project_id/needs/new", get(create_need_form))
        .route("/projects/:project_id/needs/:need_id", post(update_need))
        .route("/projects/:project_id/needs/:need_id/delete", get(delete_need))
        .route("/projects/:project_id/ideas/new", get(create_idea_form))
        .route("/projects/:project_id/ideas", post(create_idea))
        .route("/projects/:project_id/ideas/:idea_id/delete", get(delete_idea))
        
        // 评论相关路由
        .route("/comments", post(create_comment))
        .route("/comments/:idea_id", get(get_comments))
        .route("/comments/:comment_id/delete", delete(delete_comment))
        .route("/comments/:comment_id/edit", post(edit_comment))
        .route("/comments/batch-delete", post(batch_delete_comments))

        // 图片管理路由
        .route("/projects/:project_id/images/upload", post(upload_project_image))
        .route("/projects/:project_id/images/:image_id/set-main", get(set_main_image))
        .route("/projects/:project_id/images/:image_id/delete", get(delete_project_image))
        
        // 参与项目路由
        .route("/projects/:project_id/join", get(join_project_handler))
        
        // 管理者管理路由
        .route("/projects/:project_id/managers/add/:user_id", get(add_manager_handler))
        .route("/projects/:project_id/participants/:user_id/remove", get(remove_participant_handler))
        .route("/projects/:project_id/delete", delete(delete_project_handler))
        
        // 项目归档路由
        .route("/projects/:project_id/archive/:archive", get(archive_project_handler))
        
        // 需求详情路由
        .route("/projects/:project_id/needs/:need_id", get(need_detail_handler))
        
        // 项目导出路由
        .route("/projects/:project_id/export/:format", get(export_project_handler))
        
        // 用户管理路由
        .route("/dashboard/users", get(user_management_handler))
        .route("/api/users/search", get(search_users_api))
        
        // 贡献度统计API
        .route("/api/users/:user_id/contributions", get(get_user_contributions_api))
        .route("/api/projects/:project_id/participants/contributions", get(get_project_participants_contributions_api))
        
        // 项目模板路由
        .route("/templates", get(templates_handler))
        .route("/templates/create", get(create_template_handler))
        .route("/templates/create", post(create_template_post_handler))
        .route("/templates/:template_id/edit", get(edit_template_handler))
        .route("/templates/:template_id/edit", post(edit_template_post_handler))
        .route("/templates/:template_id/delete", delete(delete_template_handler))
        .route("/templates/:template_id/create-project", get(create_project_from_template_handler))
        .route("/projects/create-from-template", post(create_project_from_template_post_handler))
}