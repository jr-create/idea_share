# 项目共创平台 - 完整文档

## 1. 项目概述

项目共创平台是一个面向全民参与的项目出谋划策与协作网站，旨在为用户提供一个发布项目、征集创意、协同补充方案、报名参与、跟踪进展的平台。

### 核心功能
- 用户注册登录
- 项目发布与管理
- 创意提交与讨论
- 项目参与与协作
- 进展跟踪与更新
- 标签管理与搜索

## 2. 技术栈

### 后端
- **Rust**：提供高性能、内存安全的基础
- **Axum**：基于Tower的Web框架，提供人体工学和模块化设计
- **SQLx**：异步、纯Rust的数据库访问库，支持编译期SQL检查
- **PostgreSQL**：强大的关系型数据库

### 前端
- **Askama**：Rust模板引擎，用于服务端渲染
- **HTMX**：用于局部页面刷新，增强用户交互
- **Tailwind CSS**：现代化的CSS框架

### 其他工具
- **tracing**：日志和可观测性
- **Criterion**：性能基准测试
- **Docker**：容器化部署

## 3. 项目架构

### 目录结构
```
idea_share/
├── src/
│   ├── auth/            # 认证相关代码
│   ├── cache/           # 缓存管理
│   ├── db/              # 数据库连接和初始化
│   ├── handlers/        # 请求处理
│   ├── models/          # 数据模型
│   ├── routes/          # 路由配置
│   └── main.rs          # 主入口
├── templates/           # Askama模板
├── migrations/          # 数据库迁移文件
├── benches/             # 基准测试
├── Cargo.toml           # 项目配置
├── Dockerfile           # Docker配置
└── docker-compose.yml   # Docker Compose配置
```

### 核心模块
1. **认证模块**：处理用户注册、登录和会话管理
2. **项目模块**：处理项目的创建、更新和查询
3. **创意模块**：处理用户提交的创意和方案
4. **参与模块**：处理用户参与项目的流程
5. **进展模块**：处理项目的进展更新
6. **缓存模块**：提供文件系统缓存功能

### 数据流
1. 用户请求通过Axum路由系统分发到相应的处理器
2. 处理器通过SQLx与PostgreSQL数据库交互
3. 服务端使用Askama模板渲染页面
4. 前端使用HTMX进行局部交互

## 4. 数据库设计

### 主要表结构
- **users**：用户信息
- **projects**：项目信息
- **project_tags**：项目标签关联
- **ideas**：创意和方案
- **project_participants**：项目参与者
- **project_progress**：项目进展
- **project_tasks**：项目任务
- **project_needs**：项目需求
- **comments**：评论

## 5. 部署方案

### 本地开发
1. 安装Rust和PostgreSQL
2. 克隆项目代码
3. 配置`.env`文件
4. 运行`cargo run`启动服务器

### Docker部署
1. 构建Docker镜像：`docker build -t idea_share .`
2. 使用Docker Compose：`docker-compose up -d`

## 6. 开发指南

### 代码规范
- 遵循Rust官方风格指南
- 使用`cargo fmt`格式化代码
- 使用`cargo clippy`检查代码质量

### 测试
- 单元测试：`cargo test`
- 基准测试：`cargo bench`
- 负载测试：`cargo run --bin load_test`

## 7. 未来规划

### 功能扩展
- 实时通知系统
- 文件上传与管理
- 项目推荐算法
- 社区积分系统

### 性能优化
- 数据库索引优化
- 缓存策略改进
- 异步任务处理
- 水平扩展

## 8. 贡献指南

1. Fork项目仓库
2. 创建功能分支
3. 提交代码
4. 运行测试
5. 提交Pull Request

## 9. 许可证

本项目采用MIT许可证。