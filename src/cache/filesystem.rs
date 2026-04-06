use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// 缓存目录结构
pub struct CacheDirectories {
    pub cache: PathBuf,
    pub uploads: PathBuf,
    pub training: PathBuf,
}

impl CacheDirectories {
    /// 创建默认的缓存目录结构
    pub fn new() -> Self {
        let base = Path::new("data");
        Self {
            cache: base.join("cache"),
            uploads: base.join("uploads"),
            training: base.join("training"),
        }
    }

    /// 确保所有目录存在
    pub fn ensure_directories(&self) -> io::Result<()> {
        fs::create_dir_all(&self.cache)?;
        fs::create_dir_all(&self.uploads)?;
        fs::create_dir_all(&self.training)?;
        Ok(())
    }
}

/// 写入数据到缓存文件
pub fn write_cache_file<T>(path: &Path, data: &T) -> io::Result<()> 
where
    T: serde::Serialize,
{
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    let json = serde_json::to_string(data)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// 从缓存文件读取数据
pub fn read_cache_file<T>(path: &Path) -> io::Result<T> 
where
    T: serde::de::DeserializeOwned,
{
    let mut file = File::open(path)?;
    let mut json = String::new();
    file.read_to_string(&mut json)?;
    let data: T = serde_json::from_str(&json)?;
    Ok(data)
}

/// 删除缓存文件
pub fn delete_cache_file(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// 检查缓存文件是否存在
pub fn cache_file_exists(path: &Path) -> bool {
    path.exists()
}

/// 获取缓存文件的修改时间
pub fn get_cache_file_modified_time(path: &Path) -> io::Result<std::time::SystemTime> {
    fs::metadata(path)?.modified()
}
