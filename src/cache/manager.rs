use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::cache::filesystem::{self, CacheDirectories};

/// 缓存管理器
pub struct CacheManager {
    directories: CacheDirectories,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            directories: CacheDirectories::new(),
        }
    }

    /// 初始化缓存管理器
    pub fn init(&self) -> std::io::Result<()> {
        self.directories.ensure_directories()
    }

    /// 生成缓存文件路径
    pub fn get_cache_path(&self, category: &str, key: &str) -> PathBuf {
        self.directories
            .cache
            .join(category)
            .join(format!("{}.json", key))
    }

    /// 生成训练数据缓存路径
    pub fn get_training_path(&self, key: &str) -> PathBuf {
        self.directories
            .training
            .join(format!("{}.json", key))
    }

    /// 生成上传文件路径
    pub fn get_upload_path(&self, filename: &str) -> PathBuf {
        self.directories
            .uploads
            .join(filename)
    }

    /// 写入缓存
    pub fn set<T>(&self, category: &str, key: &str, data: &T) -> std::io::Result<()>
    where
        T: serde::Serialize,
    {
        let path = self.get_cache_path(category, key);
        filesystem::write_cache_file(&path, data)
    }

    /// 读取缓存
    pub fn get<T>(&self, category: &str, key: &str) -> std::io::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self.get_cache_path(category, key);
        if filesystem::cache_file_exists(&path) {
            Ok(Some(filesystem::read_cache_file(&path)?))
        } else {
            Ok(None)
        }
    }

    /// 读取缓存并检查过期时间
    pub fn get_with_expiry<T>(&self, category: &str, key: &str, max_age: Duration) -> std::io::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self.get_cache_path(category, key);
        if filesystem::cache_file_exists(&path) {
            let modified = filesystem::get_cache_file_modified_time(&path)?;
            let now = SystemTime::now();
            if now.duration_since(modified).unwrap_or(max_age + Duration::from_secs(1)) <= max_age {
                Ok(Some(filesystem::read_cache_file(&path)?))
            } else {
                // 缓存已过期，删除
                filesystem::delete_cache_file(&path)?;
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 删除缓存
    pub fn delete(&self, category: &str, key: &str) -> std::io::Result<()>
    {
        let path = self.get_cache_path(category, key);
        filesystem::delete_cache_file(&path)
    }

    /// 写入训练数据
    pub fn set_training_data<T>(&self, key: &str, data: &T) -> std::io::Result<()>
    where
        T: serde::Serialize,
    {
        let path = self.get_training_path(key);
        filesystem::write_cache_file(&path, data)
    }

    /// 读取训练数据
    pub fn get_training_data<T>(&self, key: &str) -> std::io::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self.get_training_path(key);
        if filesystem::cache_file_exists(&path) {
            Ok(Some(filesystem::read_cache_file(&path)?))
        } else {
            Ok(None)
        }
    }

    /// 清理过期缓存
    pub fn clean_expired(&self, max_age: Duration) -> std::io::Result<()>
    {
        self.clean_expired_in_dir(&self.directories.cache, max_age)
    }

    /// 清理目录中的过期文件
    fn clean_expired_in_dir(&self, dir: &Path, max_age: Duration) -> std::io::Result<()>
    {
        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.clean_expired_in_dir(&path, max_age)?;
                } else if path.is_file() {
                    if let Ok(modified) = filesystem::get_cache_file_modified_time(&path) {
                        let now = SystemTime::now();
                        if now.duration_since(modified).unwrap_or(max_age + Duration::from_secs(1)) > max_age {
                            std::fs::remove_file(&path)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
