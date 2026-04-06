use crate::cache::manager::CacheManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 训练数据示例结构
#[derive(Serialize, Deserialize, Debug)]
pub struct TrainingData {
    pub project_id: i64,
    pub data: Vec<f64>,
    pub labels: Vec<u32>,
    pub created_at: String,
}

/// 缓存使用示例
pub fn example_usage() {
    // 创建缓存管理器
    let cache_manager = CacheManager::new();
    
    // 初始化缓存管理器
    cache_manager.init().expect("Failed to initialize cache manager");
    
    // 示例训练数据
    let training_data = TrainingData {
        project_id: 1,
        data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        labels: vec![0, 1, 0, 1, 0],
        created_at: "2026-04-04".to_string(),
    };
    
    // 写入训练数据到缓存
    cache_manager
        .set_training_data("project_1", &training_data)
        .expect("Failed to write training data");
    println!("Training data written to cache");
    
    // 从缓存读取训练数据
    let read_data = cache_manager
        .get_training_data::<TrainingData>("project_1")
        .expect("Failed to read training data");
    
    match read_data {
        Some(data) => println!("Read training data: {:?}", data),
        None => println!("No training data found in cache"),
    }
    
    // 使用缓存管理器的其他功能
    // 写入普通缓存
    let user_data = serde_json::json!({
        "id": 1,
        "name": "John Doe",
        "email": "john@example.com"
    });
    
    cache_manager
        .set("users", "1", &user_data)
        .expect("Failed to write user data");
    println!("User data written to cache");
    
    // 读取普通缓存
    let read_user = cache_manager
        .get::<serde_json::Value>("users", "1")
        .expect("Failed to read user data");
    
    match read_user {
        Some(data) => println!("Read user data: {:?}", data),
        None => println!("No user data found in cache"),
    }
    
    // 读取缓存并检查过期时间
    let read_user_with_expiry = cache_manager
        .get_with_expiry::<serde_json::Value>("users", "1", Duration::from_secs(3600))
        .expect("Failed to read user data with expiry");
    
    match read_user_with_expiry {
        Some(data) => println!("Read user data with expiry: {:?}", data),
        None => println!("No user data found in cache or cache expired"),
    }
    
    // 清理过期缓存
    cache_manager
        .clean_expired(Duration::from_secs(3600))
        .expect("Failed to clean expired cache");
    println!("Expired cache cleaned");
}
