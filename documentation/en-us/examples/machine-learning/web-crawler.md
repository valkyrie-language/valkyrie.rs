# Web Crawler

Valkyrie provides powerful tools for building web crawlers and scrapers with built-in support for asynchronous operations, rate limiting, and content extraction.

## Basic HTTP Requests

### Simple GET Request

```valkyrie
using std::http
using std::async

micro fetch_page(url: string) -> Result⟨string, Error⟩ {
    let response = http::get(url).await?
    
    if response.status == 200 {
        Fine(response.body)
    } else {
        Fail(Error::HttpError(response.status))
    }
}
```

### POST Request with Data

```valkyrie
micro submit_form(url: string, data: {string: string}) -> Result⟨string, Error⟩ {
    let response = http::post(url)
        .json(data)
        .send().await?
    
    Fine(response.body)
}
```

### Custom Headers

```valkyrie
micro fetch_with_headers(url: string) -> Result⟨string, Error⟩ {
    let response = http::get(url)
        .header("User-Agent", "ValkyrieBot/1.0")
        .header("Accept", "text/html")
        .header("Cookie", "session=abc123")
        .send().await?
    
    Fine(response.body)
}
```

## HTML Parsing

### Basic Parsing

```valkyrie
using std::html

micro extract_links(html: string) -> [string] {
    let doc = html::parse(html)
    
    doc.select("a[href]")
        .iter()
        .map { $.attr("href") }
        .filter { $.starts_with("http") }
        .collect()
}
```

### CSS Selectors

```valkyrie
micro scrape_article(html: string) -> Article {
    let doc = html::parse(html)
    
    Article {
        title: doc.select("h1.title").first()?.text(),
        author: doc.select(".author").first()?.text(),
        content: doc.select("article .content").text(),
        date: doc.select("time").first()?.attr("datetime"),
    }
}
```

### XPath Support

```valkyrie
micro scrape_with_xpath(html: string) -> [string] {
    let doc = html::parse(html)
    
    doc.xpath("//div[@class='item']/h2/text()")
        .iter()
        .map { $.to_string() }
        .collect()
}
```

## Rate Limiting

### Basic Rate Limiter

```valkyrie
using std::rate_limit

class Crawler {
    limiter: rate_limit::TokenBucket
    
    micro new(requests_per_second: f64) -> Self {
        Self {
            limiter: rate_limit::TokenBucket::new(
                capacity: 10,
                refill_rate: requests_per_second
            )
        }
    }
    
    async micro fetch(mut self, url: string) -> Result⟨string, Error⟩ {
        # Wait for token
        self.limiter.acquire().await
        
        http::get(url).send().await?.body
    }
}
```

### Polite Crawling

```valkyrie
class PoliteCrawler {
    delay: Duration
    last_request: HashMap⟨string, Instant⟩
    
    micro new(min_delay: Duration) -> Self {
        Self {
            delay: min_delay,
            last_request: HashMap::new(),
        }
    }
    
    async micro fetch(mut self, url: string) -> Result⟨string, Error⟩ {
        let domain = parse_domain(url)
        
        # Check last request time for this domain
        if let Some(last) = self.last_request.get(domain) {
            let elapsed = Instant::now() - last
            if elapsed < self.delay {
                sleep(self.delay - elapsed).await
            }
        }
        
        self.last_request.insert(domain, Instant::now())
        http::get(url).send().await?.body
    }
}
```

## Concurrent Crawling

### Parallel Requests

```valkyrie
micro crawl_parallel(urls: [string]) -> [Result⟨string, Error⟩] {
    let futures = urls.map { url |
        async { fetch_page(url).await }
    }
    
    Future.join_all(futures).await
}
```

### Bounded Concurrency

```valkyrie
using std::async::Semaphore

micro crawl_bounded(urls: [string], max_concurrent: usize) -> [Result⟨string, Error⟩] {
    let semaphore = Semaphore::new(max_concurrent)
    
    let futures = urls.map { url |
        async {
            let permit = semaphore.acquire().await
            let result = fetch_page(url).await
            permit.release()
            result
        }
    }
    
    Future.join_all(futures).await
}
```

## URL Management

### URL Queue

```valkyrie
class UrlQueue {
    pending: VecDeque⟨string⟩
    visited: HashSet⟨string⟩
    
    micro new() -> Self {
        Self {
            pending: VecDeque::new(),
            visited: HashSet::new(),
        }
    }
    
    micro push(mut self, url: string) {
        let normalized = normalize_url(url)
        if !self.visited.contains(normalized) {
            self.visited.insert(normalized)
            self.pending.push_back(url)
        }
    }
    
    micro pop(mut self) -> Option⟨string⟩ {
        self.pending.pop_front()
    }
}
```

### URL Normalization

```valkyrie
micro normalize_url(url: string) -> string {
    let parsed = Url::parse(url)
    
    # Remove fragment
    parsed.set_fragment(None)
    
    # Remove trailing slash
    let path = parsed.path().trim_end_matches("/")
    parsed.set_path(path)
    
    # Sort query parameters
    let query = parsed.query_pairs()
        .sorted_by { |a, b| a.key.cmp(b.key) }
        .collect()
    parsed.set_query(query)
    
    parsed.to_string()
}
```

## Robots.txt Compliance

```valkyrie
using std::robots

class RobotsAwareCrawler {
    user_agent: string
    rules: HashMap⟨string, robots::Rules⟩
    
    micro new(user_agent: string) -> Self {
        Self {
            user_agent,
            rules: HashMap::new(),
        }
    }
    
    async micro can_fetch(mut self, url: string) -> bool {
        let domain = parse_domain(url)
        
        if !self.rules.contains(domain) {
            let robots_url = "https://${domain}/robots.txt"
            let rules = robots::fetch(robots_url).await
            self.rules.insert(domain, rules)
        }
        
        self.rules.get(domain)?.allowed(url, self.user_agent)
    }
}
```

## Data Extraction Patterns

### Structured Extraction

```valkyrie
micro extract_products(html: string) -> [Product] {
    let doc = html::parse(html)
    
    doc.select(".product-item")
        .iter()
        .map { item |
            Product {
                name: item.select(".name").text(),
                price: item.select(".price").text().parse_currency(),
                image: item.select("img").attr("src"),
                url: item.select("a").attr("href"),
            }
        }
        .collect()
}
```

### Pagination Handling

```valkyrie
async micro crawl_paginated(base_url: string) -> [Article] {
    let mut all_articles = []
    let mut page = 1
    let mut has_more = true
    
    while has_more {
        let url = "${base_url}?page=${page}"
        let html = fetch_page(url).await?
        let articles = extract_articles(html)
        
        if articles.is_empty() {
            has_more = false
        } else {
            all_articles.extend(articles)
            page += 1
        }
        
        # Be polite
        sleep(Duration::seconds(1)).await
    }
    
    all_articles
}
```

## Storage

### Save to File

```valkyrie
micro save_results(data: [Article], path: string) -> Result⟨(), Error⟩ {
    let json = serde::to_json(data)?
    std::fs::write(path, json)
}
```

### Database Storage

```valkyrie
using std::database

class CrawlerStorage {
    db: database::Connection
    
    async micro save_article(mut self, article: Article) -> Result⟨(), Error⟩ {
        self.db.execute(
            "INSERT INTO articles (title, url, content) VALUES (?, ?, ?)",
            (article.title, article.url, article.content)
        ).await
    }
    
    async micro article_exists(self, url: string) -> bool {
        self.db.query_one(
            "SELECT 1 FROM articles WHERE url = ?",
            (url,)
        ).await?.is_some()
    }
}
```

## Error Handling

```valkyrie
micro robust_fetch(url: string, max_retries: i32) -> Result⟨string, Error⟩ {
    let mut attempts = 0
    
    loop {
        match http::get(url).timeout(Duration::seconds(30)).send().await {
            Fine(response) if response.status == 200 => {
                return Fine(response.body)
            }
            Fine(response) if response.status == 404 => {
                return Fail(Error::NotFound(url))
            }
            Fine(response) if response.status == 429 => {
                # Rate limited, wait and retry
                let retry_after = response.header("Retry-After")
                    .and_then { $.parse() }
                    .unwrap_or(60)
                sleep(Duration::seconds(retry_after)).await
            }
            Fail(e) if attempts < max_retries => {
                attempts += 1
                sleep(Duration::seconds(2 ^ attempts)).await
            }
            Fail(e) => {
                return Fail(Error::NetworkError(e))
            }
        }
    }
}
```

## Best Practices

1. **Respect robots.txt**: Always check crawling permissions
2. **Use rate limiting**: Don't overwhelm servers
3. **Handle errors gracefully**: Implement retries with backoff
4. **Use proper User-Agent**: Identify your crawler
5. **Cache results**: Avoid re-fetching unchanged content
6. **Be mindful of resources**: Use bounded concurrency
