# 使用官方Rust镜像作为构建环境
FROM rust:1.78 as builder

# 设置工作目录
WORKDIR /app

# 复制Cargo.toml和Cargo.lock文件
COPY Cargo.toml Cargo.lock ./

# 下载依赖
RUN cargo fetch

# 复制源代码
COPY src/ src/
COPY templates/ templates/
COPY migrations/ migrations/

# 构建应用
RUN cargo build --release

# 使用轻量级的Alpine镜像作为运行环境
FROM debian:bookworm-slim

# 安装必要的依赖
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 从构建阶段复制编译后的二进制文件
COPY --from=builder /app/target/release/idea_share /app/

# 复制环境变量示例文件
COPY .env.example /app/.env

# 暴露端口
EXPOSE 3000

# 运行应用
CMD ["./idea_share"]