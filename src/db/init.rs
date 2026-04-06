use sqlx::PgPool;
use tracing::info;

pub async fn init_database(pool: &PgPool) {
    info!("Initializing database tables");

    // Create users table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id bigserial primary key,
            username varchar(32) unique not null,
            email varchar(128) unique not null,
            password_hash text not null,
            bio text not null default '',
            avatar_url text not null default '',
            created_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create users table");

    // Create project_categories table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_categories (
            id bigserial primary key,
            name varchar(255) not null unique,
            description text,
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_categories table");

    // Create index for project_categories
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_categories_name ON project_categories(name)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_categories table");

    // Create projects table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS projects (
            id bigserial primary key,
            user_id bigint not null references users(id) on delete cascade,
            category_id bigint references project_categories(id) on delete set null,
            title varchar(200) not null,
            slug varchar(220) unique not null,
            summary text not null default '',
            description text not null default '',
            stage varchar(50) not null default 'idea',
            location varchar(100) not null default '',
            budget_range varchar(50) not null default '',
            existing_resources text not null default '',
            needed_resources text not null default '',
            deadline date,
            is_public boolean not null default true,
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create projects table");

    // Create project_tags table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_tags (
            project_id bigint not null references projects(id) on delete cascade,
            tag varchar(50) not null,
            primary key (project_id, tag)
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_tags table");

    // Create ideas table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS ideas (
            id bigserial primary key,
            project_id bigint not null references projects(id) on delete cascade,
            user_id bigint not null references users(id) on delete cascade,
            title varchar(200) not null,
            content text not null,
            idea_type varchar(50) not null default '',
            feasibility_score int not null default 0,
            estimated_cost varchar(50) not null default '',
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create ideas table");

    // Create project_participants table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_participants (
            project_id bigint not null references projects(id) on delete cascade,
            user_id bigint not null references users(id) on delete cascade,
            role varchar(50) not null default 'participant',
            message text not null default '',
            created_at timestamptz not null default now(),
            primary key (project_id, user_id)
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_participants table");

    // Create comments table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS comments (
            id bigserial primary key,
            project_id bigint references projects(id) on delete cascade,
            idea_id bigint references ideas(id) on delete cascade,
            user_id bigint references users(id) on delete cascade,
            name text,
            email text,
            content text not null,
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create comments table");

    // Create idea_votes table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS idea_votes (
            id bigserial primary key,
            idea_id bigint not null references ideas(id) on delete cascade,
            user_id bigint not null references users(id) on delete cascade,
            vote_type smallint not null, -- 1 for upvote, -1 for downvote
            created_at timestamptz not null default now(),
            unique (idea_id, user_id)
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create idea_votes table");

    // Create project_progress table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_progress (
            id bigserial primary key,
            project_id bigint not null references projects(id) on delete cascade,
            user_id bigint not null references users(id) on delete cascade,
            content text not null,
            progress_percentage integer,
            update_date date,
            created_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_progress table");
    
    // Add progress_percentage column if it doesn't exist
    sqlx::query(r#"
        ALTER TABLE project_progress 
        ADD COLUMN IF NOT EXISTS progress_percentage integer
    "#)
    .execute(pool)
    .await
    .expect("Failed to add progress_percentage column");
    
    // Add update_date column if it doesn't exist
    sqlx::query(r#"
        ALTER TABLE project_progress 
        ADD COLUMN IF NOT EXISTS update_date date
    "#)
    .execute(pool)
    .await
    .expect("Failed to add update_date column");

    // Create project_tasks table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_tasks (
            id bigserial primary key,
            project_id bigint not null references projects(id) on delete cascade,
            title varchar(200) not null,
            description text not null default '',
            status varchar(50) not null default 'pending',
            assignee_id bigint references users(id) on delete set null,
            due_date date,
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_tasks table");

    // Create project_needs table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_needs (
            id bigserial primary key,
            project_id bigint not null references projects(id) on delete cascade,
            title varchar(200) not null,
            description text not null default '',
            priority varchar(50) not null default 'medium',
            status varchar(50) not null default 'open',
            created_at timestamptz not null default now(),
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_needs table");

    // Create project_images table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS project_images (
            id bigserial primary key,
            project_id bigint not null references projects(id) on delete cascade,
            user_id bigint not null references users(id) on delete cascade,
            image_url text not null,
            is_main boolean not null default false,
            created_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create project_images table");

    // Create user_profile_history table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS user_profile_history (
            id bigserial primary key,
            user_id bigint not null references users(id) on delete cascade,
            field_name text not null,
            old_value text not null,
            new_value text not null,
            updated_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create user_profile_history table");

    // Create index for faster vote queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_idea_votes_idea_id ON idea_votes(idea_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for idea_votes table");

    // Create index for faster project queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at DESC)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for projects table");

    // Create index for faster project category queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_projects_category_id ON projects(category_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for projects category_id");

    // Create audit_logs table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id bigserial primary key,
            user_id bigint references users(id) on delete cascade,
            action_type varchar(50) not null,
            entity_type varchar(50) not null,
            entity_id bigint not null,
            details text not null,
            created_at timestamptz not null default now()
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create audit_logs table");

    // Create index for faster audit log queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for audit_logs table");

    // Create index for faster user queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for users table");

    // Create index for faster project_tags queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_tags_tag ON project_tags(tag)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_tags table");

    // Create index for faster project_progress queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_progress_project_id ON project_progress(project_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_progress table");

    // Create index for faster project_tasks queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_tasks_project_id ON project_tasks(project_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_tasks table");

    // Create index for faster project_needs queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_needs_project_id ON project_needs(project_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_needs table");

    // Create index for faster ideas queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_ideas_project_id ON ideas(project_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for ideas table");

    // Create index for faster project_images queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_images_project_id ON project_images(project_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_images table");

    // Create index for faster project_participants queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_project_participants_project_id ON project_participants(project_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for project_participants table");
    
    // Add name and email columns to comments table if they don't exist
    sqlx::query(r#"
        ALTER TABLE comments 
        ADD COLUMN IF NOT EXISTS name text
    "#)
    .execute(pool)
    .await
    .expect("Failed to add name column to comments table");
    
    sqlx::query(r#"
        ALTER TABLE comments 
        ADD COLUMN IF NOT EXISTS email text
    "#)
    .execute(pool)
    .await
    .expect("Failed to add email column to comments table");
    
    // Add parent_id column to comments table for reply functionality
    sqlx::query(r#"
        ALTER TABLE comments 
        ADD COLUMN IF NOT EXISTS parent_id bigint REFERENCES comments(id) ON DELETE CASCADE
    "#)
    .execute(pool)
    .await
    .expect("Failed to add parent_id column to comments table");
    
    // Make user_id column nullable
    sqlx::query(r#"
        ALTER TABLE comments 
        ALTER COLUMN user_id DROP NOT NULL
    "#)
    .execute(pool)
    .await
    .expect("Failed to make user_id column nullable");

    // Create need_idea_relations table to link needs and ideas
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS need_idea_relations (
            id bigserial primary key,
            need_id bigint not null references project_needs(id) on delete cascade,
            idea_id bigint not null references ideas(id) on delete cascade,
            created_at timestamptz not null default now(),
            unique (need_id, idea_id)
        )
    "#)
    .execute(pool)
    .await
    .expect("Failed to create need_idea_relations table");

    // Create index for faster need_idea_relations queries
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_need_idea_relations_need_id ON need_idea_relations(need_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for need_idea_relations table");

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_need_idea_relations_idea_id ON need_idea_relations(idea_id)
    "#)
    .execute(pool)
    .await
    .expect("Failed to create index for need_idea_relations table");

    info!("Database tables initialized successfully");

}
