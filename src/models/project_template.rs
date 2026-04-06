use crate::db::connection::DbConnection;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Executor;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewProjectTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub stage: String,
    pub location: String,
    pub budget_range: String,
    pub existing_resources: String,
    pub needed_resources: String,
    pub is_public: bool,
    pub tags: Vec<String>,
}

impl ProjectTemplate {
    pub async fn create(
        db: &DbConnection,
        user_id: i64,
        template: NewProjectTemplate,
    ) -> Result<ProjectTemplate, String> {
        let pool = db.get();
        
        let result = sqlx::query!(r#"
            INSERT INTO project_templates 
            (user_id, name, description, category, stage, location, budget_range, existing_resources, needed_resources, is_public)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, name, description, category, stage, location, budget_range, existing_resources, needed_resources, is_public, created_at, updated_at
        "#,
            user_id,
            template.name,
            template.description,
            template.category,
            template.stage,
            template.location,
            template.budget_range,
            template.existing_resources,
            template.needed_resources,
            template.is_public,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let id: i64 = result.id;
        
        // Insert tags
        for tag in &template.tags {
            sqlx::query!(r#"INSERT INTO project_template_tags (template_id, tag) VALUES ($1, $2)"#,
                id,
                tag,
            )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
        
        Ok(ProjectTemplate {
            id: result.id,
            user_id: result.user_id,
            name: result.name,
            description: result.description,
            category: result.category,
            stage: result.stage,
            location: result.location,
            budget_range: result.budget_range,
            existing_resources: result.existing_resources,
            needed_resources: result.needed_resources,
            is_public: result.is_public,
            created_at: result.created_at,
            updated_at: result.updated_at,
            tags: template.tags,
        })
    }
    
    pub async fn get_by_id(db: &DbConnection, id: i64) -> Result<Option<ProjectTemplate>, String> {
        let pool = db.get();
        
        let result = sqlx::query!(r#"
            SELECT id, user_id, name, description, category, stage, location, budget_range, existing_resources, needed_resources, is_public, created_at, updated_at
            FROM project_templates
            WHERE id = $1
        "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        match result {
            Some(row) => {
                let id: i64 = row.id;
                
                // Get tags
                let tags_result = sqlx::query!(r#"SELECT tag FROM project_template_tags WHERE template_id = $1"#,
                    id
                )
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;
                
                let tags: Vec<String> = tags_result
                    .into_iter()
                    .map(|row| row.tag)
                    .collect();
                
                Ok(Some(ProjectTemplate {
                    id: row.id,
                    user_id: row.user_id,
                    name: row.name,
                    description: row.description,
                    category: row.category,
                    stage: row.stage,
                    location: row.location,
                    budget_range: row.budget_range,
                    existing_resources: row.existing_resources,
                    needed_resources: row.needed_resources,
                    is_public: row.is_public,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    tags,
                }))
            }
            None => Ok(None),
        }
    }
    
    pub async fn get_all_public(db: &DbConnection) -> Result<Vec<ProjectTemplate>, String> {
        let pool = db.get();
        
        let result = sqlx::query!(r#"
            SELECT id, user_id, name, description, category, stage, location, budget_range, existing_resources, needed_resources, is_public, created_at, updated_at
            FROM project_templates
            WHERE is_public = true
            ORDER BY created_at DESC
        "#)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let mut templates = Vec::new();
        for row in result {
            let id: i64 = row.id;
            
            // Get tags
            let tags_result = sqlx::query!(r#"SELECT tag FROM project_template_tags WHERE template_id = $1"#,
                id
            )
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            
            let tags: Vec<String> = tags_result
                .into_iter()
                .map(|row| row.tag)
                .collect();
            
            templates.push(ProjectTemplate {
                id: row.id,
                user_id: row.user_id,
                name: row.name,
                description: row.description,
                category: row.category,
                stage: row.stage,
                location: row.location,
                budget_range: row.budget_range,
                existing_resources: row.existing_resources,
                needed_resources: row.needed_resources,
                is_public: row.is_public,
                created_at: row.created_at,
                updated_at: row.updated_at,
                tags,
            });
        }
        
        Ok(templates)
    }
    
    pub async fn get_by_user(db: &DbConnection, user_id: i64) -> Result<Vec<ProjectTemplate>, String> {
        let pool = db.get();
        
        let result = sqlx::query!(r#"
            SELECT id, user_id, name, description, category, stage, location, budget_range, existing_resources, needed_resources, is_public, created_at, updated_at
            FROM project_templates
            WHERE user_id = $1
            ORDER BY created_at DESC
        "#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let mut templates = Vec::new();
        for row in result {
            let id: i64 = row.id;
            
            // Get tags
            let tags_result = sqlx::query!(r#"SELECT tag FROM project_template_tags WHERE template_id = $1"#,
                id
            )
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            
            let tags: Vec<String> = tags_result
                .into_iter()
                .map(|row| row.tag)
                .collect();
            
            templates.push(ProjectTemplate {
                id: row.id,
                user_id: row.user_id,
                name: row.name,
                description: row.description,
                category: row.category,
                stage: row.stage,
                location: row.location,
                budget_range: row.budget_range,
                existing_resources: row.existing_resources,
                needed_resources: row.needed_resources,
                is_public: row.is_public,
                created_at: row.created_at,
                updated_at: row.updated_at,
                tags,
            });
        }
        
        Ok(templates)
    }
    
    pub async fn update(
        db: &DbConnection,
        id: i64,
        template: NewProjectTemplate,
    ) -> Result<ProjectTemplate, String> {
        let pool = db.get();
        
        let result = sqlx::query!(r#"
            UPDATE project_templates
            SET name = $1, description = $2, category = $3, stage = $4, location = $5, budget_range = $6, existing_resources = $7, needed_resources = $8, is_public = $9, updated_at = now()
            WHERE id = $10
            RETURNING id, user_id, name, description, category, stage, location, budget_range, existing_resources, needed_resources, is_public, created_at, updated_at
        "#,
            template.name,
            template.description,
            template.category,
            template.stage,
            template.location,
            template.budget_range,
            template.existing_resources,
            template.needed_resources,
            template.is_public,
            id,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        // Delete existing tags
        sqlx::query!(r#"DELETE FROM project_template_tags WHERE template_id = $1"#,
            id
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        // Insert new tags
        for tag in &template.tags {
            sqlx::query!(r#"INSERT INTO project_template_tags (template_id, tag) VALUES ($1, $2)"#,
                id,
                tag,
            )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
        
        Ok(ProjectTemplate {
            id: result.id,
            user_id: result.user_id,
            name: result.name,
            description: result.description,
            category: result.category,
            stage: result.stage,
            location: result.location,
            budget_range: result.budget_range,
            existing_resources: result.existing_resources,
            needed_resources: result.needed_resources,
            is_public: result.is_public,
            created_at: result.created_at,
            updated_at: result.updated_at,
            tags: template.tags,
        })
    }
    
    pub async fn delete(db: &DbConnection, id: i64) -> Result<(), String> {
        let pool = db.get();
        
        // Delete tags first
        sqlx::query!(r#"DELETE FROM project_template_tags WHERE template_id = $1"#,
            id
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        // Delete template
        sqlx::query!(r#"DELETE FROM project_templates WHERE id = $1"#,
            id
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}