-- Create project_categories table
CREATE TABLE IF NOT EXISTS project_categories (
  id bigserial primary key,
  name varchar(255) not null unique,
  description text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

-- Create index on name for faster search
CREATE INDEX IF NOT EXISTS idx_project_categories_name ON project_categories(name);

-- Add category_id column to projects table
ALTER TABLE projects ADD COLUMN IF NOT EXISTS category_id bigint references project_categories(id) on delete set null;

-- Create index on category_id for faster filtering
CREATE INDEX IF NOT EXISTS idx_projects_category_id ON projects(category_id);

-- Insert some default categories
INSERT INTO project_categories (name, description) VALUES
('技术创新', '与技术相关的创新项目'),
('社会公益', '致力于社会公益的项目'),
('教育培训', '教育和培训相关的项目'),
('环保节能', '环保和节能相关的项目'),
('文化艺术', '文化和艺术相关的项目'),
('商业创业', '商业和创业相关的项目')
ON CONFLICT (name) DO NOTHING;