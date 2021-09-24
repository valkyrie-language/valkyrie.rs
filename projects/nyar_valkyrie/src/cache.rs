//! 缓存管理模块
//!
//! 提供多层缓存系统，包括内存缓存、磁盘缓存和分布式缓存。

use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock as AsyncRwLock};

use nyar_ast::Ast;
use nyar_hir::Module;

use crate::{
    config::CacheConfig,
    error::{CacheError, RuntimeResult},
};

/// 缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// 键类型
    pub key_type: CacheKeyType,
    /// 标识符
    pub identifier: String,
    /// 版本或哈希
    pub version: String,
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.key_type, self.identifier, self.version)
    }
}

/// 缓存键类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheKeyType {
    /// AST 缓存
    Ast,
    /// HIR 缓存
    Hir,
    /// 类型信息缓存
    TypeInfo,
    /// 符号信息缓存
    SymbolInfo,
    /// 编译结果缓存
    CompilationResult,
    /// 查询结果缓存
    QueryResult,
    /// 代码生成结果缓存
    CodegenResult,
    /// 自定义缓存
    Custom(String),
}

impl fmt::Display for CacheKeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheKeyType::Ast => write!(f, "ast"),
            CacheKeyType::Hir => write!(f, "hir"),
            CacheKeyType::TypeInfo => write!(f, "type_info"),
            CacheKeyType::SymbolInfo => write!(f, "symbol_info"),
            CacheKeyType::CompilationResult => write!(f, "compilation_result"),
            CacheKeyType::QueryResult => write!(f, "query_result"),
            CacheKeyType::CodegenResult => write!(f, "codegen_result"),
            CacheKeyType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// 缓存的数据
    pub data: T,
    /// 创建时间
    pub created_at: SystemTime,
    /// 最后访问时间
    pub last_accessed: SystemTime,
    /// 访问次数
    pub access_count: u64,
    /// 过期时间
    pub expires_at: Option<SystemTime>,
    /// 数据大小（字节）
    pub size: usize,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl<T> CacheEntry<T> {
    /// 创建新的缓存条目
    pub fn new(data: T, ttl: Option<Duration>) -> Self
    where
        T: Serialize,
    {
        let now = SystemTime::now();
        let size = bincode::serialized_size(&data).unwrap_or(0) as usize;

        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            expires_at: ttl.map(|ttl| now + ttl),
            size,
            metadata: HashMap::new(),
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        }
        else {
            false
        }
    }

    /// 更新访问信息
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
        self.access_count += 1;
    }

    /// 获取年龄
    pub fn age(&self) -> Duration {
        SystemTime::now().duration_since(self.created_at).unwrap_or(Duration::ZERO)
    }

    /// 获取自上次访问以来的时间
    pub fn idle_time(&self) -> Duration {
        SystemTime::now().duration_since(self.last_accessed).unwrap_or(Duration::ZERO)
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 条目数量
    pub entry_count: usize,
    /// 总大小（字节）
    pub total_size: usize,
    /// 平均条目大小
    pub average_entry_size: usize,
    /// 最大条目大小
    pub max_entry_size: usize,
    /// 按类型统计
    pub type_stats: HashMap<CacheKeyType, TypeStats>,
}

/// 类型统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeStats {
    /// 条目数量
    pub count: usize,
    /// 总大小
    pub total_size: usize,
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
}

impl CacheStats {
    /// 计算命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        }
        else {
            self.hits as f64 / total as f64
        }
    }

    /// 更新统计信息
    pub fn update_stats(&mut self, entries: &HashMap<CacheKey, CacheEntry<Vec<u8>>>) {
        self.entry_count = entries.len();
        self.total_size = entries.values().map(|e| e.size).sum();
        self.average_entry_size = if self.entry_count > 0 { self.total_size / self.entry_count } else { 0 };
        self.max_entry_size = entries.values().map(|e| e.size).max().unwrap_or(0);

        // 更新类型统计
        self.type_stats.clear();
        for (key, entry) in entries {
            let type_stat = self.type_stats.entry(key.key_type.clone()).or_default();
            type_stat.count += 1;
            type_stat.total_size += entry.size;
        }
    }
}

/// 缓存策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// 最近最少使用
    LRU,
    /// 最近最少访问
    LFU,
    /// 先进先出
    FIFO,
    /// 随机
    Random,
    /// 基于 TTL
    TTL,
}

/// 内存缓存
pub struct MemoryCache {
    /// 配置
    config: CacheConfig,
    /// 缓存数据
    data: Arc<DashMap<CacheKey, CacheEntry<Vec<u8>>>>,
    /// 统计信息
    stats: Arc<RwLock<CacheStats>>,
    /// 访问顺序（用于 LRU）
    access_order: Arc<RwLock<Vec<CacheKey>>>,
}

impl MemoryCache {
    /// 创建新的内存缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            data: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            access_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 获取缓存项
    pub async fn get<T>(&self, key: &CacheKey) -> RuntimeResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(mut entry) = self.data.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                drop(entry);
                self.data.remove(key);
                self.update_access_order(key, false).await;

                let mut stats = self.stats.write();
                stats.misses += 1;
                return Ok(None);
            }

            // 更新访问信息
            entry.touch();

            // 反序列化数据
            let result = bincode::deserialize(&entry.data)
                .map_err(|e| CacheError::SerializationError { message: format!("Failed to deserialize cache entry: {}", e) })?;

            // 更新统计和访问顺序
            let mut stats = self.stats.write();
            stats.hits += 1;
            drop(stats);

            self.update_access_order(key, true).await;

            Ok(Some(result))
        }
        else {
            let mut stats = self.stats.write();
            stats.misses += 1;
            Ok(None)
        }
    }

    /// 设置缓存项
    pub async fn set<T>(&self, key: CacheKey, value: T, ttl: Option<Duration>) -> RuntimeResult<()>
    where
        T: Serialize,
    {
        // 序列化数据
        let data = bincode::serialize(&value)
            .map_err(|e| CacheError::SerializationError { message: format!("Failed to serialize cache entry: {}", e) })?;

        // 检查大小限制
        if data.len() > self.config.max_entry_size {
            return Err(CacheError::EntrySizeExceeded { size: data.len(), max_size: self.config.max_entry_size }.into());
        }

        // 创建缓存条目
        let entry = CacheEntry {
            data,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 0,
            expires_at: ttl.map(|ttl| SystemTime::now() + ttl),
            size: data.len(),
            metadata: HashMap::new(),
        };

        // 检查是否需要清理
        self.maybe_evict().await?;

        // 插入缓存
        self.data.insert(key.clone(), entry);
        self.update_access_order(&key, true).await;

        Ok(())
    }

    /// 删除缓存项
    pub async fn remove(&self, key: &CacheKey) -> RuntimeResult<bool> {
        let removed = self.data.remove(key).is_some();
        if removed {
            self.update_access_order(key, false).await;
        }
        Ok(removed)
    }

    /// 清空缓存
    pub async fn clear(&self) -> RuntimeResult<()> {
        self.data.clear();
        self.access_order.write().clear();
        *self.stats.write() = CacheStats::default();
        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.write();
        let data_map: HashMap<CacheKey, CacheEntry<Vec<u8>>> =
            self.data.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect();
        stats.update_stats(&data_map);
        stats.clone()
    }

    /// 检查缓存项是否存在
    pub fn contains_key(&self, key: &CacheKey) -> bool {
        if let Some(entry) = self.data.get(key) {
            !entry.is_expired()
        }
        else {
            false
        }
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // 私有辅助方法

    /// 更新访问顺序
    async fn update_access_order(&self, key: &CacheKey, accessed: bool) {
        let mut order = self.access_order.write();

        // 移除旧位置
        order.retain(|k| k != key);

        // 如果是访问，添加到末尾
        if accessed {
            order.push(key.clone());
        }
    }

    /// 可能需要清理缓存
    async fn maybe_evict(&self) -> RuntimeResult<()> {
        let current_size = self.data.len();

        if current_size >= self.config.max_entries {
            self.evict_entries(current_size - self.config.max_entries + 1).await?;
        }

        Ok(())
    }

    /// 清理缓存条目
    async fn evict_entries(&self, count: usize) -> RuntimeResult<()> {
        match self.config.eviction_policy {
            EvictionPolicy::LRU => self.evict_lru(count).await,
            EvictionPolicy::LFU => self.evict_lfu(count).await,
            EvictionPolicy::FIFO => self.evict_fifo(count).await,
            EvictionPolicy::TTL => self.evict_expired().await,
            EvictionPolicy::Random => self.evict_random(count).await,
        }
    }

    /// LRU 清理
    async fn evict_lru(&self, count: usize) -> RuntimeResult<()> {
        let mut order = self.access_order.write();
        let keys_to_remove: Vec<CacheKey> = order.drain(..count.min(order.len())).collect();
        drop(order);

        for key in keys_to_remove {
            self.data.remove(&key);
        }

        Ok(())
    }

    /// LFU 清理
    async fn evict_lfu(&self, count: usize) -> RuntimeResult<()> {
        let mut entries: Vec<(CacheKey, u64)> =
            self.data.iter().map(|entry| (entry.key().clone(), entry.value().access_count)).collect();

        entries.sort_by_key(|(_, access_count)| *access_count);

        for (key, _) in entries.into_iter().take(count) {
            self.data.remove(&key);
            self.update_access_order(&key, false).await;
        }

        Ok(())
    }

    /// FIFO 清理
    async fn evict_fifo(&self, count: usize) -> RuntimeResult<()> {
        let mut entries: Vec<(CacheKey, SystemTime)> =
            self.data.iter().map(|entry| (entry.key().clone(), entry.value().created_at)).collect();

        entries.sort_by_key(|(_, created_at)| *created_at);

        for (key, _) in entries.into_iter().take(count) {
            self.data.remove(&key);
            self.update_access_order(&key, false).await;
        }

        Ok(())
    }

    /// 清理过期条目
    async fn evict_expired(&self) -> RuntimeResult<()> {
        let expired_keys: Vec<CacheKey> =
            self.data.iter().filter(|entry| entry.value().is_expired()).map(|entry| entry.key().clone()).collect();

        for key in expired_keys {
            self.data.remove(&key);
            self.update_access_order(&key, false).await;
        }

        Ok(())
    }

    /// 随机清理
    async fn evict_random(&self, count: usize) -> RuntimeResult<()> {
        use rand::seq::SliceRandom;

        let keys: Vec<CacheKey> = self.data.iter().map(|entry| entry.key().clone()).collect();
        let mut rng = rand::thread_rng();
        let keys_to_remove: Vec<CacheKey> = keys.choose_multiple(&mut rng, count).cloned().collect();

        for key in keys_to_remove {
            self.data.remove(&key);
            self.update_access_order(&key, false).await;
        }

        Ok(())
    }
}

/// 磁盘缓存
pub struct DiskCache {
    /// 配置
    config: CacheConfig,
    /// 缓存目录
    cache_dir: PathBuf,
    /// 内存索引
    index: Arc<AsyncRwLock<HashMap<CacheKey, CacheMetadata>>>,
}

/// 缓存元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    /// 文件路径
    file_path: PathBuf,
    /// 创建时间
    created_at: SystemTime,
    /// 大小
    size: u64,
    /// 过期时间
    expires_at: Option<SystemTime>,
}

impl DiskCache {
    /// 创建新的磁盘缓存
    pub async fn new(config: CacheConfig, cache_dir: PathBuf) -> RuntimeResult<Self> {
        // 确保缓存目录存在
        fs::create_dir_all(&cache_dir)
            .await
            .map_err(|e| CacheError::IoError { message: format!("Failed to create cache directory: {}", e) })?;

        let cache = Self { config, cache_dir, index: Arc::new(AsyncRwLock::new(HashMap::new())) };

        // 加载现有索引
        cache.load_index().await?;

        Ok(cache)
    }

    /// 获取缓存项
    pub async fn get<T>(&self, key: &CacheKey) -> RuntimeResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let index = self.index.read().await;

        if let Some(metadata) = index.get(key) {
            // 检查是否过期
            if let Some(expires_at) = metadata.expires_at {
                if SystemTime::now() > expires_at {
                    drop(index);
                    self.remove(key).await?;
                    return Ok(None);
                }
            }

            // 读取文件
            let data = fs::read(&metadata.file_path)
                .await
                .map_err(|e| CacheError::IoError { message: format!("Failed to read cache file: {}", e) })?;

            // 反序列化
            let result = bincode::deserialize(&data)
                .map_err(|e| CacheError::SerializationError { message: format!("Failed to deserialize cache entry: {}", e) })?;

            Ok(Some(result))
        }
        else {
            Ok(None)
        }
    }

    /// 设置缓存项
    pub async fn set<T>(&self, key: CacheKey, value: T, ttl: Option<Duration>) -> RuntimeResult<()>
    where
        T: Serialize,
    {
        // 序列化数据
        let data = bincode::serialize(&value)
            .map_err(|e| CacheError::SerializationError { message: format!("Failed to serialize cache entry: {}", e) })?;

        // 生成文件路径
        let file_name = format!("{}.cache", self.hash_key(&key));
        let file_path = self.cache_dir.join(file_name);

        // 写入文件
        fs::write(&file_path, &data)
            .await
            .map_err(|e| CacheError::IoError { message: format!("Failed to write cache file: {}", e) })?;

        // 更新索引
        let metadata = CacheMetadata {
            file_path,
            created_at: SystemTime::now(),
            size: data.len() as u64,
            expires_at: ttl.map(|ttl| SystemTime::now() + ttl),
        };

        let mut index = self.index.write().await;
        index.insert(key, metadata);

        Ok(())
    }

    /// 删除缓存项
    pub async fn remove(&self, key: &CacheKey) -> RuntimeResult<bool> {
        let mut index = self.index.write().await;

        if let Some(metadata) = index.remove(key) {
            // 删除文件
            if metadata.file_path.exists() {
                fs::remove_file(&metadata.file_path)
                    .await
                    .map_err(|e| CacheError::IoError { message: format!("Failed to remove cache file: {}", e) })?;
            }
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    /// 清空缓存
    pub async fn clear(&self) -> RuntimeResult<()> {
        let mut index = self.index.write().await;

        // 删除所有文件
        for metadata in index.values() {
            if metadata.file_path.exists() {
                let _ = fs::remove_file(&metadata.file_path).await;
            }
        }

        index.clear();
        Ok(())
    }

    // 私有辅助方法

    /// 加载索引
    async fn load_index(&self) -> RuntimeResult<()> {
        let index_file = self.cache_dir.join("index.json");

        if index_file.exists() {
            let data = fs::read_to_string(&index_file)
                .await
                .map_err(|e| CacheError::IoError { message: format!("Failed to read index file: {}", e) })?;

            let loaded_index: HashMap<CacheKey, CacheMetadata> = serde_json::from_str(&data)
                .map_err(|e| CacheError::SerializationError { message: format!("Failed to deserialize index: {}", e) })?;

            let mut index = self.index.write().await;
            *index = loaded_index;
        }

        Ok(())
    }

    /// 保存索引
    pub async fn save_index(&self) -> RuntimeResult<()> {
        let index = self.index.read().await;
        let index_file = self.cache_dir.join("index.json");

        let data = serde_json::to_string_pretty(&*index)
            .map_err(|e| CacheError::SerializationError { message: format!("Failed to serialize index: {}", e) })?;

        fs::write(&index_file, data)
            .await
            .map_err(|e| CacheError::IoError { message: format!("Failed to write index file: {}", e) })?;

        Ok(())
    }

    /// 计算键的哈希值
    fn hash_key(&self, key: &CacheKey) -> String {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// 多层缓存管理器
pub struct CacheManager {
    /// 内存缓存
    memory_cache: MemoryCache,
    /// 磁盘缓存
    disk_cache: Option<DiskCache>,
    /// 配置
    config: CacheConfig,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub async fn new(config: CacheConfig) -> RuntimeResult<Self> {
        let memory_cache = MemoryCache::new(config.clone());

        let disk_cache = if config.enable_disk_cache {
            let cache_dir = PathBuf::from(&config.disk_cache_dir);
            Some(DiskCache::new(config.clone(), cache_dir).await?)
        }
        else {
            None
        };

        Ok(Self { memory_cache, disk_cache, config })
    }

    /// 获取缓存项
    pub async fn get<T>(&self, key: &CacheKey) -> RuntimeResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Serialize + Clone,
    {
        // 首先尝试内存缓存
        if let Some(value) = self.memory_cache.get(key).await? {
            return Ok(Some(value));
        }

        // 然后尝试磁盘缓存
        if let Some(disk_cache) = &self.disk_cache {
            if let Some(value) = disk_cache.get(key).await? {
                // 将数据提升到内存缓存
                let _ = self.memory_cache.set(key.clone(), value.clone(), None).await;
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// 设置缓存项
    pub async fn set<T>(&self, key: CacheKey, value: T, ttl: Option<Duration>) -> RuntimeResult<()>
    where
        T: Serialize + Clone,
    {
        // 设置内存缓存
        self.memory_cache.set(key.clone(), value.clone(), ttl).await?;

        // 设置磁盘缓存
        if let Some(disk_cache) = &self.disk_cache {
            disk_cache.set(key, value, ttl).await?;
        }

        Ok(())
    }

    /// 删除缓存项
    pub async fn remove(&self, key: &CacheKey) -> RuntimeResult<bool> {
        let mut removed = false;

        // 从内存缓存删除
        if self.memory_cache.remove(key).await? {
            removed = true;
        }

        // 从磁盘缓存删除
        if let Some(disk_cache) = &self.disk_cache {
            if disk_cache.remove(key).await? {
                removed = true;
            }
        }

        Ok(removed)
    }

    /// 清空所有缓存
    pub async fn clear(&self) -> RuntimeResult<()> {
        self.memory_cache.clear().await?;

        if let Some(disk_cache) = &self.disk_cache {
            disk_cache.clear().await?;
        }

        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        self.memory_cache.get_stats()
    }

    /// 保存磁盘缓存索引
    pub async fn save_disk_index(&self) -> RuntimeResult<()> {
        if let Some(disk_cache) = &self.disk_cache {
            disk_cache.save_index().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_cache() {
        let config = CacheConfig::default();
        let cache = MemoryCache::new(config);

        let key = CacheKey { key_type: CacheKeyType::Ast, identifier: "test".to_string(), version: "1.0".to_string() };

        let value = "test_value".to_string();

        // 测试设置和获取
        cache.set(key.clone(), value.clone(), None).await.unwrap();
        let retrieved: Option<String> = cache.get(&key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // 测试删除
        let removed = cache.remove(&key).await.unwrap();
        assert!(removed);

        let retrieved: Option<String> = cache.get(&key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_disk_cache() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();
        let cache = DiskCache::new(config, temp_dir.path().to_path_buf()).await.unwrap();

        let key = CacheKey { key_type: CacheKeyType::Hir, identifier: "test".to_string(), version: "1.0".to_string() };

        let value = vec![1, 2, 3, 4, 5];

        // 测试设置和获取
        cache.set(key.clone(), value.clone(), None).await.unwrap();
        let retrieved: Option<Vec<i32>> = cache.get(&key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // 测试保存和加载索引
        cache.save_index().await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = CacheConfig::default();
        config.enable_disk_cache = true;
        config.disk_cache_dir = temp_dir.path().to_string_lossy().to_string();

        let manager = CacheManager::new(config).await.unwrap();

        let key =
            CacheKey { key_type: CacheKeyType::CompilationResult, identifier: "test".to_string(), version: "1.0".to_string() };

        let value = HashMap::from([("key".to_string(), "value".to_string())]);

        // 测试多层缓存
        manager.set(key.clone(), value.clone(), None).await.unwrap();
        let retrieved: Option<HashMap<String, String>> = manager.get(&key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // 测试统计
        let stats = manager.get_stats();
        assert!(stats.entry_count > 0);
    }

    #[test]
    fn test_cache_entry() {
        let data = "test_data".to_string();
        let ttl = Some(Duration::from_secs(60));
        let entry = CacheEntry::new(data.clone(), ttl);

        assert_eq!(entry.data, data);
        assert!(!entry.is_expired());
        assert_eq!(entry.access_count, 0);

        let mut entry = entry;
        entry.touch();
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();

        stats.hits = 80;
        stats.misses = 20;

        assert_eq!(stats.hit_rate(), 0.8);

        stats.hits = 0;
        stats.misses = 0;
        assert_eq!(stats.hit_rate(), 0.0);
    }
}
