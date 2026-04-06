-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id bigserial primary key,
    username varchar(32) unique not null,
    email varchar(128) unique not null,
    password_hash text not null,
    bio text not null default '',
    avatar_url text not null default '',
    created_at timestamptz not null default now()
);

-- Create projects table
CREATE TABLE IF NOT EXISTS projects (
    id bigserial primary key,
    user_id bigint not null references users(id) on delete cascade,
    title varchar(200) not null,
    slug varchar(220) unique not null,
    summary text not null default '',
    description text not null default '',
    category varchar(50) not null default '',
    stage varchar(50) not null default 'idea',
    location varchar(100) not null default '',
    budget_range varchar(50) not null default '',
    existing_resources text not null default '',
    needed_resources text not null default '',
    deadline date,
    is_public boolean not null default true,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- Create project_tags table
CREATE TABLE IF NOT EXISTS project_tags (
    project_id bigint not null references projects(id) on delete cascade,
    tag varchar(50) not null,
    primary key (project_id, tag)
);

-- Create ideas table
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
);

-- Create project_participants table
CREATE TABLE IF NOT EXISTS project_participants (
    project_id bigint not null references projects(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade,
    role varchar(50) not null default 'participant',
    message text not null default '',
    created_at timestamptz not null default now(),
    primary key (project_id, user_id)
);

-- Create comments table
CREATE TABLE IF NOT EXISTS comments (
    id bigserial primary key,
    project_id bigint references projects(id) on delete cascade,
    idea_id bigint references ideas(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade,
    content text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
