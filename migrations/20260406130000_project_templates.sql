-- Create project_templates table
CREATE TABLE IF NOT EXISTS project_templates (
    id bigserial primary key,
    user_id bigint not null references users(id) on delete cascade,
    name varchar(200) not null,
    description text not null default '',
    category varchar(50) not null default '',
    stage varchar(50) not null default 'idea',
    location varchar(100) not null default '',
    budget_range varchar(50) not null default '',
    existing_resources text not null default '',
    needed_resources text not null default '',
    is_public boolean not null default true,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- Create project_template_tags table
CREATE TABLE IF NOT EXISTS project_template_tags (
    template_id bigint not null references project_templates(id) on delete cascade,
    tag varchar(50) not null,
    primary key (template_id, tag)
);