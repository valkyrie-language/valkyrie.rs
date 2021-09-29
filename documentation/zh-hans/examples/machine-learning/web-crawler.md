# 网络爬虫

Valkyrie 提供了强大的网络爬虫框架，支持异步并发、智能调度、数据提取和存储等功能。通过类型安全的 API 和零成本抽象，为大规模数据采集提供高效解决方案。

## 核心特性

### 异步 HTTP 客户端

```valkyrie
use std::http::{HttpClient, Request, Response, Method}
use std::async::{Future, Stream}
use std::time::Duration

# HTTP 客户端配置
class CrawlerConfig {
    max_concurrent: usize      # 最大并发数 ∥
    request_delay: Duration    # 请求间隔 Δt
    timeout: Duration          # 超时时间 τ
    user_agent: utf8           # 用户代理
    max_retries: u32           # 最大重试次数
}

# 异步 HTTP 客户端
class AsyncHttpClient {
    config: CrawlerConfig,
    client: HttpClient
    
    async micro new(config: CrawlerConfig) -> AsyncHttpClient {
        let client = HttpClient::builder()
            .timeout(config.timeout)
            .user_agent(config.user_agent.clone())
            .build()
            
        AsyncHttpClient { config, client }
    }
    
    async micro get(self, url: utf8) -> Result⟨Response, CrawlerError⟩ {
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .build()?
            
        let mut retry_count = 0  # 重试计数器
        
        loop {
            match self.client.send(request.clone()).await {
                Fine { value: response } => {
                    if response.status().is_success() {
                        return Ok(response)  # 成功响应
                    } else if response.status().is_server_error() && retry_count < self.config.max_retries {
                        retry_count += 1
                        std::time::sleep(Duration::from_secs(2_u64 ^ retry_count)).await  # 指数退避
                        continue
                    } else {
                        return Err(CrawlerError::HttpError(response.status()))  # HTTP错误
                    }
                },
                Fail { error: e } if retry_count < self.config.max_retries => {
                    retry_count += 1
                    std::time::sleep(Duration::from_secs(2_u64 ^ retry_count)).await  # 指数退避
                    continue
                },
                Fail { error: e } => return Err(CrawlerError::NetworkError(e))  # 网络错误
            }
        }
    }
    
    async micro post(self, url: utf8, body: [u8]) -> Result⟨Response, CrawlerError⟩ {
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(body.to_vec())
            .build()?
            
        self.send_with_retry(request).await
    }
}

# 错误处理
unite CrawlerError {
    HttpError { status: StatusCode },
    NetworkError { error: HttpError },
    ParseError { message: utf8 },
    RateLimitExceeded,
    Timeout,
    UnsupportedContentType { content_type: utf8 }
}
```

### URL 管理和调度

```valkyrie
use std::collections::{HashSet, VecDeque}
use std::sync::{Arc, Mutex}
use std::hash::{Hash, Hasher}

# URL 优先级队列
class UrlQueue {
    high_priority: VecDeque⟨CrawlTask⟩
    normal_priority: VecDeque⟨CrawlTask⟩
    low_priority: VecDeque⟨CrawlTask⟩
    visited: HashSet⟨utf8⟩
    in_progress: HashSet⟨utf8⟩
}

class CrawlTask {
    url: utf8
    depth: u32
    priority: Priority
    metadata: TaskMetadata
}

enums Priority {
    High = 3,
    Normal = 2,
    Low = 1
}

class TaskMetadata {
    referrer: Option⟨utf8⟩
    created_at: SystemTime
    retry_count: u32
    custom_headers: HashMap⟨utf8, utf8⟩
}

imply UrlQueue {
    micro new() -> UrlQueue {
        UrlQueue {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            visited: HashSet::new(),
            in_progress: HashSet::new()
        }
    }
    
    micro add_url(mut self, url: utf8, priority: Priority, depth: u32) -> bool {
        if self.visited.contains(url) || self.in_progress.contains(url) {
            return false  # URL已存在
        }
        
        let task = CrawlTask {
            url: url.clone(),
            depth,  # 爬取深度
            priority,  # 优先级
            metadata: TaskMetadata {
                referrer: None,
                created_at: SystemTime::now(),  # 创建时间
                retry_count: 0,  # 重试次数
                custom_headers: HashMap::new()
            }
        }
        
        match priority {
            Priority::High => self.high_priority.push_back(task),    # 高优先级队列
            Priority::Normal => self.normal_priority.push_back(task), # 普通优先级队列
            Priority::Low => self.low_priority.push_back(task)       # 低优先级队列
        }
        
        true  # 添加成功
    }
    
    micro next_task(mut self) -> Option⟨CrawlTask⟩ {
        if let Some(task) = self.high_priority.pop_front() {
            self.in_progress.insert(task.url.clone())
            return Some(task)
        }
        
        if let Some(task) = self.normal_priority.pop_front() {
            self.in_progress.insert(task.url.clone())
            return Some(task)
        }
        
        if let Some(task) = self.low_priority.pop_front() {
            self.in_progress.insert(task.url.clone())
            return Some(task)
        }
        
        None
    }
    
    micro mark_completed(mut self, url: utf8) {
        self.in_progress.remove(url)
        self.visited.insert(url.to_string())
    }
    
    micro mark_failed(mut self, url: utf8) {
        self.in_progress.remove(url)
    }
    
    micro is_empty(self) -> bool {
        self.high_priority.is_empty() && 
        self.normal_priority.is_empty() && 
        self.low_priority.is_empty()
    }
}

# 智能调度器
class CrawlerScheduler {
    queue: Arc⟨Mutex⟨UrlQueue⟩⟩,
    rate_limiter: RateLimiter,
    domain_delays: HashMap⟨utf8, SystemTime⟩
    
    micro new(requests_per_second: f64) -> CrawlerScheduler {
        CrawlerScheduler {
            queue: Arc::new(Mutex::new(UrlQueue::new())),
            rate_limiter: RateLimiter::new(requests_per_second),
            domain_delays: HashMap::new()
        }
    }
    
    async micro schedule_request(mut self) -> Option⟨CrawlTask⟩ {
        # 等待速率限制
        self.rate_limiter.wait_if_needed().await
        
        let mut queue = self.queue.lock().unwrap()
        
        # 寻找可以立即处理的任务
        while let Some(task) = queue.next_task() {
            let domain = extract_domain(task.url)?  # 提取域名
            
            # 检查域名延迟
            if let Some(t_last) = self.domain_delays.get(domain) {
                let elapsed = SystemTime::now().duration_since(*t_last).unwrap_or_default()
                if elapsed < Duration::from_secs(1) {
                    # 重新加入队列，降低优先级
                    queue.add_url(task.url, Priority::Low, task.depth)
                    continue
                }
            }
            
            self.domain_delays.insert(domain, SystemTime::now())  # 更新时间戳
            return Some(task)  # 返回任务
        }
        
        None
    }
}

# 速率限制器
class RateLimiter {
    interval: Duration
    last_request: Option⟨SystemTime⟩
}

imply RateLimiter {
    micro new(requests_per_second: f64) -> RateLimiter {
        RateLimiter {
            interval: Duration::from_secs_f64(1.0 / requests_per_second),
            last_request: None
        }
    }
    
    async micro wait_if_needed(mut self) {
        if let Some(last) = self.last_request {
            let elapsed = SystemTime::now().duration_since(last).unwrap_or(Duration::ZERO)
            if elapsed < self.interval {
                std::time::sleep(self.interval - elapsed).await
            }
        }
        self.last_request = Some(SystemTime::now())
    }
}
```

### HTML 解析和数据提取

```valkyrie
use std::html::{Document, Element, Selector}
use std::regex::Regex

# HTML 解析器
class HtmlParser {
    base_url: utf8
}

imply HtmlParser {
    micro new(base_url: utf8) -> HtmlParser {
        HtmlParser { base_url }
    }
    
    micro extract_links(self, html: utf8) -> [utf8] {
        # 使用 CSS 选择器提取链接
        let selector = Selector::parse("a[href]").unwrap()
        let document = Html::parse_document(html)
        
        document.select(selector)
            .filter_map { %.value().attr("href") }
            .map { self.resolve_url(%).unwrap_or(%.to_string()) }
            .collect()
    }
    
    micro extract_text(self, doc: Document, selector: utf8) -> [utf8] {
        let css_selector = Selector::parse(selector).unwrap()
        let mut texts = []
        
        loop element in doc.select(css_selector) {
            let text = element.text().collect::<[utf8]>().join(" ").trim().to_string()
            if !text.is_empty() {
                texts.push(text)
            }
        }
        
        texts
    }
    
    micro extract_structured_data(self, doc: Document) -> StructuredData {
        let mut data = StructuredData::new()
        
        # 提取标题
        if let Some(title) = doc.select(Selector::parse("title").unwrap()).next() {
            data.title = Some(title.text().collect::<utf8>())
        }
        
        # 提取元数据
        loop meta in doc.select(Selector::parse("meta").unwrap()) {
            if let (Some(name), Some(content)) = (meta.value().attr("name"), meta.value().attr("content")) {
                data.meta.insert(name.to_string(), content.to_string())
            }
        }
        
        # 提取 JSON-LD 结构化数据
        loop script in doc.select(Selector::parse("script[type='application/ld+json']").unwrap()) {
            let json_text = script.text().collect::<utf8>()
            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(json_text) {
                data.json_ld.push(json_data)
            }
        }
        
        data
    }
    
    micro resolve_url(self, href: utf8) -> Option⟨utf8⟩ {
        if href.starts_with("http://") || href.starts_with("https://") {
            Some(href.to_string())
        } else if href.starts_with("/") {
            Some(format!("{}{}", self.base_url, href))
        } else if href.starts_with("./") {
            Some(format!("{}/{}", self.base_url, href[2..]))
        } else if !href.starts_with("#") && !href.starts_with("mailto:") && !href.starts_with("javascript:") {
            Some(format!("{}/{}", self.base_url, href))
        } else {
            None
        }
    }
}

class StructuredData {
    title: Option⟨utf8⟩
    meta: HashMap⟨utf8, utf8⟩
    json_ld: [serde_json::Value]
    custom_fields: HashMap⟨utf8, utf8⟩
}

imply StructuredData {
    micro new() -> StructuredData {
        StructuredData {
            title: None,
            meta: HashMap::new(),
            json_ld: [],
            custom_fields: HashMap::new()
        }
    }
}
```

### 数据存储和管道

```valkyrie
use std::database::{Database, Connection, Transaction}
use std::fs::{File, OpenOptions}
use std::io::{Write, BufWriter}

# 数据存储接口
trait DataStore {
    async micro save_page(mut self, url: utf8, data: CrawledData) -> Result⟨(), StoreError⟩
    async micro save_links(mut self, parent_url: utf8, links: [utf8]) -> Result⟨(), StoreError⟩
    async micro get_crawled_urls(self) -> Result⟨HashSet⟨utf8⟩, StoreError⟩
}

# 抓取结果
class CrawledData {
    url: utf8
    title: Option⟨utf8⟩
    content: utf8
    links: [utf8]
    metadata: StructuredData
    crawled_at: SystemTime
    response_time: Duration
    status_code: u16
}

# 数据库存储实现
class DatabaseStore {
    connection: Connection
}

imply DatabaseStore: DataStore {
    async micro save_page(mut self, url: utf8, data: CrawledData) -> Result⟨(), StoreError⟩ {
        let mut tx = self.connection.begin().await?
        
        # 保存页面数据
        sqlx::query!(
            "INSERT INTO pages (url, title, content, status_code, crawled_at, response_time) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             ON CONFLICT (url) DO UPDATE SET 
             title = EXCLUDED.title, 
             content = EXCLUDED.content, 
             status_code = EXCLUDED.status_code, 
             crawled_at = EXCLUDED.crawled_at, 
             response_time = EXCLUDED.response_time",
            url,
            data.title,
            data.content,
            data.status_code as i32,
            data.crawled_at,
            data.response_time.as_millis() as i64
        ).execute(mut tx).await?
        
        # 保存元数据
        loop (key, value) in data.metadata.meta {
            sqlx::query!(
                "INSERT INTO page_metadata (url, key, value) VALUES ($1, $2, $3) 
                 ON CONFLICT (url, key) DO UPDATE SET value = EXCLUDED.value",
                data.url, key, value
            ).execute(mut tx).await?
        }
        
        tx.commit().await?
        Ok(())
    }
    
    async micro save_links(mut self, parent_url: utf8, links: [utf8]) -> Result⟨(), StoreError⟩ {
        let mut tx = self.connection.begin().await?
        
        loop link in links {
            sqlx::query!(
                "INSERT INTO links (parent_url, target_url, discovered_at) 
                 VALUES ($1, $2, $3) 
                 ON CONFLICT (parent_url, target_url) DO NOTHING",
                parent_url, link, SystemTime::now()
            ).execute(mut tx).await?
        }
        
        tx.commit().await?
        Ok(())
    }
    
    async micro get_crawled_urls(self) -> Result⟨HashSet⟨utf8⟩, StoreError⟩ {
        let rows = sqlx::query!("SELECT url FROM pages")
            .fetch_all(self.connection).await?
        
        Ok(rows.into_iter().map { %.url }.collect())
    }
}

# JSON 文件存储实现
class JsonFileStore {
    file_path: utf8
    writer: BufWriter⟨File⟩
}

imply JsonFileStore {
    micro new(file_path: utf8) -> Result⟨JsonFileStore, StoreError⟩ {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?
        
        Ok(JsonFileStore {
            file_path,
            writer: BufWriter::new(file)
        })
    }
}

imply JsonFileStore: DataStore {
    async micro save_page(mut self, url: utf8, data: CrawledData) -> Result⟨(), StoreError⟩ {
        let json_data = serde_json::to_string(data)?
        writeln!(self.writer, "{}", json_data)?
        self.writer.flush()?
        Ok(())
    }
    
    async micro save_links(mut self, parent_url: utf8, links: [utf8]) -> Result⟨(), StoreError⟩ {
        let link_data = serde_json::json!({
            "parent_url": parent_url,
            "links": links,
            "discovered_at": SystemTime::now()
        })
        
        writeln!(self.writer, "{}", link_data)?
        self.writer.flush()?
        Ok(())
    }
    
    async micro get_crawled_urls(self) -> Result⟨HashSet⟨utf8⟩, StoreError⟩ {
        # 从文件读取已爬取的 URL
        let content = std::fs::read_to_string(self.file_path)?
        let mut urls = HashSet::new()
        
        loop line in content.lines() {
            if let Ok(data) = serde_json::from_str::<CrawledData>(line) {
                urls.insert(data.url)
            }
        }
        
        Ok(urls)
    }
}

unite StoreError {
    DatabaseError { error: sqlx::Error },
    IoError { error: std::io::Error },
    SerializationError { error: serde_json::Error }
}
```

### 完整的爬虫引擎

```valkyrie
use std::sync::Arc
use std::async::Semaphore

# 主爬虫引擎
class WebCrawler⟨S: DataStore⟩ {
    client: AsyncHttpClient,
    scheduler: CrawlerScheduler,
    parser: HtmlParser,
    store: S,
    semaphore: Arc⟨Semaphore⟩,
    config: CrawlerEngineConfig
}

class CrawlerEngineConfig {
    max_depth: u32
    max_pages: Option⟨usize⟩
    allowed_domains: Option⟨HashSet⟨utf8⟩⟩
    url_filters: [Regex]
    content_filters: [ContentFilter]
}

class ContentFilter {
    selector: utf8
    min_length: Option⟨usize⟩
    required_keywords: [utf8]
}

imply⟨S: DataStore⟩ WebCrawler⟨S⟩ {
    async micro new(config: CrawlerConfig, store: S) -> WebCrawler⟨S⟩ {
        let client = AsyncHttpClient::new(config.clone()).await
        let scheduler = CrawlerScheduler::new(config.requests_per_second)
        
        WebCrawler {
            client,
            scheduler,
            parser: HtmlParser::new(utf8::new()),
            store,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config: CrawlerEngineConfig::default()
        }
    }
    
    async micro crawl(mut self, seed_urls: [utf8]) -> Result⟨CrawlStats, CrawlerError⟩ {
        let mut stats = CrawlStats::new()
        
        # 添加种子 URL
        loop url in seed_urls {
            self.scheduler.queue.lock().unwrap().add_url(url, Priority::High, 0)
        }
        
        # 获取已爬取的 URL
        let crawled_urls = self.store.get_crawled_urls().await?
        
        # 主爬取循环
        while let Some(task) = self.scheduler.schedule_request().await {
            if crawled_urls.contains(task.url) {
                continue
            }
            
            if let Some(max_pages) = self.config.max_pages {
                if stats.pages_crawled >= max_pages {
                    break
                }
            }
            
            # 获取并发许可
            let permit = self.semaphore.clone().acquire_owned().await.unwrap()
            
            let client = self.client.clone()
            let store = self.store.clone()
            let parser = self.parser.clone()
            let scheduler = self.scheduler.clone()
            let config = self.config.clone()
            
            # 异步处理单个页面
            spawn(async move {
                let _permit = permit  # 确保许可在任务结束时释放
                
                match Self::crawl_single_page(client, store, parser, task, config).await {
                    Fine { value: page_data } => {
                        # 添加发现的链接到队列
                        loop link in page_data.links {
                            if page_data.depth < config.max_depth {
                                scheduler.queue.lock().unwrap().add_url(
                                    link.clone(), 
                                    Priority::Normal, 
                                    page_data.depth + 1
                                )
                            }
                        }
                        
                        scheduler.queue.lock().unwrap().mark_completed(page_data.url)
                    },
                    Fail { error: e } => {
                        print("Failed to crawl {}: {:?}", task.url, e)
                        scheduler.queue.lock().unwrap().mark_failed(task.url)
                    }
                }
            })
            
            stats.pages_crawled += 1  # 页面计数
        }
        
        Ok(stats)
    }
    
    async micro crawl_single_page(
        client: AsyncHttpClient,
        mut store: S,
        parser: HtmlParser,
        task: CrawlTask,
        config: CrawlerEngineConfig
    ) -> Result⟨CrawledPageData, CrawlerError⟩ {
        let start_time = SystemTime::now()
        
        # 发送 HTTP 请求
        let response = client.get(task.url).await?
        let response_time = SystemTime::now().duration_since(start_time).unwrap_or_default()
        
        # 检查内容类型
        let content_type = response.headers()
            .get("content-type")
            .and_then { %.to_str().ok() }
            .unwrap_or("")
        
        if !content_type.contains("text/html") {
            return Err(CrawlerError::UnsupportedContentType(content_type.to_string()))
        }
        
        # 解析 HTML
        let html = response.text().await?
        let doc = parser.parse(html)?
        
        # 提取数据
        let links = parser.extract_links(doc)
        # ... 更多逻辑
    }
}
```
```
