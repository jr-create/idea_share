-- Create project_progress table for progress updates
CREATE TABLE IF NOT EXISTS project_progress (
    id bigserial primary key,
    project_id bigint not null references projects(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade,
    content text not null,
    progress_percentage integer,
    update_date date,
    created_at timestamptz not null default now()
);

-- Create project_tasks table for task tracking
CREATE TABLE IF NOT EXISTS project_tasks (
    id bigserial primary key,
    project_id bigint not null references projects(id) on delete cascade,
    title varchar(200) not null,
    description text not null default '',
    status varchar(50) not null default 'pending', -- pending, in_progress, completed
    assignee_id bigint references users(id) on delete set null,
    due_date date,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- Create project_needs table for current needs management
CREATE TABLE IF NOT EXISTS project_needs (
    id bigserial primary key,
    project_id bigint not null references projects(id) on delete cascade,
    title varchar(200) not null,
    description text not null default '',
    priority varchar(50) not null default 'medium', -- low, medium, high
    status varchar(50) not null default 'open', -- open, in_progress, fulfilled
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);