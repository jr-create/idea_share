use sqlx::PgPool;
use tracing::info;
use bcrypt::hash;

pub async fn seed_database(pool: &PgPool) {
    info!("Seeding database with initial data");

    // Insert default project categories
    sqlx::query!(r#"
        INSERT INTO project_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
    "#, "技术创新", "与技术相关的创新项目")
    .execute(pool)
    .await
    .expect("Failed to insert category");

    sqlx::query!(r#"
        INSERT INTO project_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
    "#, "社会公益", "致力于社会公益的项目")
    .execute(pool)
    .await
    .expect("Failed to insert category");

    sqlx::query!(r#"
        INSERT INTO project_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
    "#, "教育培训", "教育和培训相关的项目")
    .execute(pool)
    .await
    .expect("Failed to insert category");

    sqlx::query!(r#"
        INSERT INTO project_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
    "#, "环保节能", "环保和节能相关的项目")
    .execute(pool)
    .await
    .expect("Failed to insert category");

    sqlx::query!(r#"
        INSERT INTO project_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
    "#, "文化艺术", "文化和艺术相关的项目")
    .execute(pool)
    .await
    .expect("Failed to insert category");

    sqlx::query!(r#"
        INSERT INTO project_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
    "#, "商业创业", "商业和创业相关的项目")
    .execute(pool)
    .await
    .expect("Failed to insert category");

    // Check if users table is empty
    let user_count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    if user_count == 0 {
        // Create admin user
        let password_hash = hash("admin123", 10).unwrap();
        sqlx::query!(r#"
            INSERT INTO users (username, email, password_hash, bio)
            VALUES ($1, $2, $3, $4)
        "#, "admin", "admin@example.com", password_hash, "Admin user")
        .execute(pool)
        .await
        .expect("Failed to create admin user");

        // Create test user
        let password_hash = hash("user123", 10).unwrap();
        sqlx::query!(r#"
            INSERT INTO users (username, email, password_hash, bio)
            VALUES ($1, $2, $3, $4)
        "#, "user1", "user1@example.com", password_hash, "Test user")
        .execute(pool)
        .await
        .expect("Failed to create test user");

        info!("Created admin and test users");
    }

    // Check if projects table is empty
    let project_count = sqlx::query_scalar!("SELECT COUNT(*) FROM projects")
        .fetch_one(pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    if project_count == 0 {
        // Create test project
        let project_id = sqlx::query!(r#"
            INSERT INTO projects (user_id, category_id, title, slug, summary, description, stage, location, budget_range, existing_resources, needed_resources)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
        "#, 1, 6, "桃子直播销售计划", "peach-live-sale-plan", "通过直播销售新鲜桃子", "我们有优质的桃子资源，希望通过直播方式扩大销售渠道，需要包装设计、品牌故事和直播脚本等支持。", "idea", "山东", "5000-10000", "桃子、果园", "包装、品牌故事、直播脚本、物流建议")
        .fetch_one(pool)
        .await
        .expect("Failed to create test project")
        .id;

        // Add tags to project
        sqlx::query!(r#"
            INSERT INTO project_tags (project_id, tag)
            VALUES ($1, $2)
        "#, project_id, "直播")
        .execute(pool)
        .await
        .expect("Failed to add tag");

        sqlx::query!(r#"
            INSERT INTO project_tags (project_id, tag)
            VALUES ($1, $2)
        "#, project_id, "农业")
        .execute(pool)
        .await
        .expect("Failed to add tag");

        sqlx::query!(r#"
            INSERT INTO project_tags (project_id, tag)
            VALUES ($1, $2)
        "#, project_id, "销售")
        .execute(pool)
        .await
        .expect("Failed to add tag");

        // Add idea to project
        let idea_id = sqlx::query!(r#"
            INSERT INTO ideas (project_id, user_id, title, content, idea_type, feasibility_score, estimated_cost)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
        "#, project_id, 2, "包装设计方案", "建议使用环保材料制作包装盒，设计突出新鲜、天然的主题，加入桃子图案和品牌故事。", "包装", 8, "1000-2000")
        .fetch_one(pool)
        .await
        .expect("Failed to create idea")
        .id;

        // Add comment to idea
        sqlx::query!(r#"
            INSERT INTO comments (idea_id, user_id, content)
            VALUES ($1, $2, $3)
        "#, idea_id, 1, "这个包装设计方案很好，建议加入二维码链接到直播平台。")
        .execute(pool)
        .await
        .expect("Failed to add comment");

        // Add participant to project
        sqlx::query!(r#"
            INSERT INTO project_participants (project_id, user_id, role, message)
            VALUES ($1, $2, $3, $4)
        "#, project_id, 2, "designer", "我可以提供包装设计服务")
        .execute(pool)
        .await
        .expect("Failed to add participant");

        info!("Created test project with related data");
    }

    info!("Database seeding completed");
}
