# API 文档

## 1. 页面路由

### 1.1 首页

- **URL**: `/`
- **方法**: `GET`
- **描述**: 获取首页内容，包括精选项目、热门标签和最新进展
- **响应**: HTML页面

### 1.2 用户个人资料

- **URL**: `/u/:username`
- **方法**: `GET`
- **描述**: 获取指定用户的个人资料
- **参数**:
  - `username`: 用户名（路径参数）
- **响应**: HTML页面

### 1.3 标签页面

- **URL**: `/tags`
- **方法**: `GET`
- **描述**: 获取所有标签列表
- **响应**: HTML页面

### 1.4 标签详情

- **URL**: `/tags/:tag_name`
- **方法**: `GET`
- **描述**: 获取指定标签的相关项目
- **参数**:
  - `tag_name`: 标签名称（路径参数）
- **响应**: HTML页面

## 2. 项目进展相关API

### 2.1 创建项目进展

- **URL**: `/projects/:project_id/progress`
- **方法**: `POST`
- **描述**: 为指定项目创建进展更新
- **参数**:
  - `project_id`: 项目ID（路径参数）
  - `content`: 进展内容（表单参数）
- **认证**: 需要用户登录
- **响应**: 重定向到项目详情页面

### 2.2 获取项目进展

- **URL**: `/projects/:project_id/progress`
- **方法**: `GET`
- **描述**: 获取指定项目的所有进展
- **参数**:
  - `project_id`: 项目ID（路径参数）
- **响应**: JSON格式的进展列表

## 3. 项目任务相关API

### 3.1 创建项目任务

- **URL**: `/projects/:project_id/tasks`
- **方法**: `POST`
- **描述**: 为指定项目创建任务
- **参数**:
  - `project_id`: 项目ID（路径参数）
  - `title`: 任务标题（表单参数）
  - `description`: 任务描述（表单参数）
  - `assignee_id`: 负责人ID（表单参数）
  - `due_date`: 截止日期（表单参数）
- **认证**: 需要用户登录
- **响应**: 重定向到项目详情页面

### 3.2 获取项目任务

- **URL**: `/projects/:project_id/tasks`
- **方法**: `GET`
- **描述**: 获取指定项目的所有任务
- **参数**:
  - `project_id`: 项目ID（路径参数）
- **响应**: JSON格式的任务列表

### 3.3 更新项目任务

- **URL**: `/projects/:project_id/tasks/:task_id`
- **方法**: `POST`
- **描述**: 更新指定项目的任务
- **参数**:
  - `project_id`: 项目ID（路径参数）
  - `task_id`: 任务ID（路径参数）
  - `title`: 任务标题（可选，表单参数）
  - `description`: 任务描述（可选，表单参数）
  - `status`: 任务状态（可选，表单参数）
  - `assignee_id`: 负责人ID（可选，表单参数）
  - `due_date`: 截止日期（可选，表单参数）
- **认证**: 需要用户登录
- **响应**: 重定向到项目详情页面

## 4. 项目需求相关API

### 4.1 创建项目需求

- **URL**: `/projects/:project_id/needs`
- **方法**: `POST`
- **描述**: 为指定项目创建需求
- **参数**:
  - `project_id`: 项目ID（路径参数）
  - `title`: 需求标题（表单参数）
  - `description`: 需求描述（表单参数）
  - `priority`: 优先级（表单参数）
- **认证**: 需要用户登录
- **响应**: 重定向到项目详情页面

### 4.2 获取项目需求

- **URL**: `/projects/:project_id/needs`
- **方法**: `GET`
- **描述**: 获取指定项目的所有需求
- **参数**:
  - `project_id`: 项目ID（路径参数）
- **响应**: JSON格式的需求列表

### 4.3 更新项目需求

- **URL**: `/projects/:project_id/needs/:need_id`
- **方法**: `POST`
- **描述**: 更新指定项目的需求
- **参数**:
  - `project_id`: 项目ID（路径参数）
  - `need_id`: 需求ID（路径参数）
  - `title`: 需求标题（可选，表单参数）
  - `description`: 需求描述（可选，表单参数）
  - `priority`: 优先级（可选，表单参数）
  - `status`: 状态（可选，表单参数）
- **认证**: 需要用户登录
- **响应**: 重定向到项目详情页面

## 5. 响应格式

### 5.1 成功响应

- HTML页面请求：返回HTML内容
- API请求：返回JSON格式数据

### 5.2 错误响应

- 404 Not Found: 资源不存在
- 401 Unauthorized: 未授权访问
- 500 Internal Server Error: 服务器内部错误

## 6. 认证

- 使用Cookie Session进行认证
- 受保护的路由需要用户登录
- 登录用户信息通过Extension注入到处理器

## 7. 示例请求

### 创建项目进展

```bash
POST /projects/1/progress
Content-Type: application/x-www-form-urlencoded

content=已完成包装设计，正在准备直播脚本
```

### 获取项目任务

```bash
GET /projects/1/tasks
```

### 更新项目需求

```bash
POST /projects/1/needs/1
Content-Type: application/x-www-form-urlencoded

title=寻找直播场地&status=in_progress
```

