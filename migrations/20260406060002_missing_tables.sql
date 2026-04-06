-- Create need_idea_relations table
CREATE TABLE IF NOT EXISTS need_idea_relations (
    need_id bigint not null references project_needs(id) on delete cascade,
    idea_id bigint not null references ideas(id) on delete cascade,
    primary key (need_id, idea_id)
);

-- Create project_images table
CREATE TABLE IF NOT EXISTS project_images (
    id bigserial primary key,
    project_id bigint not null references projects(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade,
    image_url text not null,
    is_main boolean not null default false,
    created_at timestamptz not null default now()
);