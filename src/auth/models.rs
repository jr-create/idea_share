use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub bio: String,
    pub avatar_url: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub bio: String,
    pub avatar_url: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub bio: String,
    pub avatar_url: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            bio: user.bio,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
        }
    }
}

// Project models
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub deadline: Option<String>,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub summary: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub deadline: Option<String>,
    pub is_public: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub title: String,
    pub summary: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub deadline: Option<String>,
    pub is_public: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ProjectTag {
    pub project_id: i64,
    pub tag: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Idea {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub idea_type: String,
    pub feasibility_score: i32,
    pub estimated_cost: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ProjectParticipant {
    pub project_id: i64,
    pub user_id: i64,
    pub role: String,
    pub message: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub project_id: Option<i64>,
    pub idea_id: Option<i64>,
    pub user_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub deadline: Option<String>,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub tags: Vec<String>,
    pub creator: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSearchRequest {
    pub query: String,
    pub tag: Option<String>,
    pub category: Option<String>,
    pub location: Option<String>,
}

// Idea models
#[derive(Debug, Deserialize)]
pub struct CreateIdeaRequest {
    pub title: String,
    pub content: String,
    pub idea_type: String,
    pub feasibility_score: i32,
    pub estimated_cost: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIdeaRequest {
    pub title: String,
    pub content: String,
    pub idea_type: String,
    pub feasibility_score: i32,
    pub estimated_cost: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct IdeaVote {
    pub id: i64,
    pub idea_id: i64,
    pub user_id: i64,
    pub vote_type: i16,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub vote_type: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeaResponse {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub idea_type: String,
    pub feasibility_score: i32,
    pub estimated_cost: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub creator: UserResponse,
    pub vote_count: i32,
    pub user_vote: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentResponse {
    pub id: i64,
    pub project_id: Option<i64>,
    pub idea_id: Option<i64>,
    pub user_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user: UserResponse,
}

// Project participant models
#[derive(Debug, Deserialize)]
pub struct JoinProjectRequest {
    pub message: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateParticipantRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectParticipantResponse {
    pub project_id: i64,
    pub user_id: i64,
    pub role: String,
    pub message: String,
    pub created_at: NaiveDateTime,
    pub user: UserResponse,
}

// Collaboration request models
#[derive(Debug, Deserialize)]
pub struct CreateCollaborationRequest {
    pub project_id: i64,
    pub message: String,
    pub requested_role: String,
}

#[derive(Debug, Deserialize)]
pub struct RespondToCollaborationRequest {
    pub accepted: bool,
    pub role: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CollaborationRequest {
    pub id: i64,
    pub project_id: i64,
    pub requester_id: i64,
    pub message: String,
    pub requested_role: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollaborationRequestResponse {
    pub id: i64,
    pub project_id: i64,
    pub requester_id: i64,
    pub message: String,
    pub requested_role: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub requester: UserResponse,
    pub project: Option<ProjectResponse>,
}