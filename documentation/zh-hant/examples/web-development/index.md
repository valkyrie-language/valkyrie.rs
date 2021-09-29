# 網頁開發

Valkyrie 提供了現代化的網頁開發框架，支持服務端渲染、客戶端應用、WebAssembly 集成、實時通信等功能，為構建高性能 Web 應用提供完整的解決案。

## Web 伺服器框架

### HTTP 伺服器

```valkyrie
# HTTP 伺服器核心
class WebServer {
    router: Router,
    middleware_stack: [Box<Middleware>],
    config: ServerConfig,
    thread_pool: ThreadPool,
}

class ServerConfig {
    host: utf8,
    port: u16,
    max_connections: usize,
    request_timeout: Duration,
    static_files_dir: utf8?,
    enable_compression: bool,
}

imply WebServer {
    micro new(config: ServerConfig) -> Self {
        WebServer {
            router: Router::new(),
            middleware_stack: [],
            config,
            thread_pool: ThreadPool::new(num_cpus::get()),
        }
    }
    
    micro route(mut self, method: HttpMethod, path: utf8, handler: imply Handler) {
        self.router.add_route(method, path, Box::new(handler))
    }
    
    micro get(mut self, path: utf8, handler: imply Handler) {
        self.route(HttpMethod::GET, path, handler)
    }
    
    micro post(mut self, path: utf8, handler: imply Handler) {
        self.route(HttpMethod::POST, path, handler)
    }
    
    micro put(mut self, path: utf8, handler: imply Handler) {
        self.route(HttpMethod::PUT, path, handler)
    }
    
    micro delete(mut self, path: utf8, handler: imply Handler) {
        self.route(HttpMethod::DELETE, path, handler)
    }
    
    micro use_middleware(mut self, middleware: imply Middleware) {
        self.middleware_stack.push(Box::new(middleware))
    }
    
    micro serve_static(mut self, path: utf8, dir: utf8) {
        let static_handler = StaticFileHandler::new(dir)
        self.get(format("{}/*", path), static_handler)
    }
    
    async micro listen(self) -> Result<(), ServerError> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
        let listener = TcpListener::bind(addr).await?
        
        print("Server listening on http://{}", addr)
        
        loop {
            let (stream, _) = listener.accept().await?
            let router = self.router.clone()
            let middleware = self.middleware_stack.clone()
            
            self.thread_pool.spawn(async move {
                if let Err(e) = handle_connection(stream, router, middleware).await {
                    print("Error handling connection: {}", e)
                }
            })
        }
    }
}

# 請求處理
async micro handle_connection(
    mut stream: TcpStream,
    router: Router,
    middleware: [Box<Middleware>]
) -> Result<(), Box<std::error::Error>> {
    let mut buffer: array<u8, 1024> = array<u8, 1024>::new(0)
    stream.read(mut buffer).await?
    
    let request = HttpRequest::parse(buffer)?
    let mut context = RequestContext::new(request)
    
    # 執行中間件鏈
    loop middleware in middleware {
        middleware.process(mut context).await?
        if context.response.is_some() {
            break
        }
    }
    
    # 如果中間件沒有生成響應，則路由到處理器
    if context.response.is_none() {
        if let Some(handler) = router.find_handler(context.request) {
            let response = handler.handle(context.request).await?
            context.response = Some(response)
        } else {
            context.response = Some(HttpResponse::not_found())
        }
    }
    
    # 發送響應
    if let Some(response) = context.response {
        let response_bytes = response.to_bytes()
        stream.write_all(response_bytes).await?
    }
    
    Ok(())
}
```

### 路由系統

```valkyrie
# 路由器
class Router {
    routes: [Route],
    groups: [RouteGroup],
}

class Route {
    method: HttpMethod,
    pattern: PathPattern,
    handler: Box<Handler>,
    middleware: [Box<Middleware>],
}

class RouteGroup {
    prefix: utf8,
    routes: [Route],
    middleware: [Box<Middleware>],
}

enums PathPattern {
    Static(utf8),
    Dynamic(utf8, [utf8]),  # 模式和參數名
    Wildcard(utf8),
}

imply Router {
    micro new() -> Self {
        Router {
            routes: [],
            groups: [],
        }
    }
    
    micro add_route(mut self, method: HttpMethod, path: utf8, handler: Box<Handler>) {
        let pattern = PathPattern::parse(path)
        let route = Route {
            method,
            pattern,
            handler,
            middleware: [],
        }
        self.routes.push(route)
    }
    
    micro group(mut self, prefix: utf8) -> RouteGroupBuilder {
        RouteGroupBuilder::new(prefix, self)
    }
    
    micro find_handler(self, request: HttpRequest) -> (imply Handler)? {
        # 首先檢查路由組
        loop group in self.groups {
            if request.path.starts_with(group.prefix) {
                let sub_path = request.path[group.prefix.length..]
                loop route in group.routes {
                    if route.method == request.method && route.pattern.matches(sub_path) {
                        return Some(route.handler.as_ref())
                    }
                }
            }
        }
        
        # 然後檢查全局路由
        loop route in self.routes {
            if route.method == request.method && route.pattern.matches(request.path) {
                return Some(route.handler.as_ref())
            }
        }
        
        None
    }
}

imply PathPattern {
    micro parse(path: utf8) -> Self {
        if path.contains(':') {
            let mut params = []
            let pattern = path.split('/').map {
                if $.starts_with(':') {
                    params.push($[1..].to_string())
                    "([^/]+)".to_string()
                } else {
                    regex::escape($)
                }
            }.collect::<[_]>().join("/")
            
            PathPattern::Dynamic(pattern, params)
        } else if path.ends_with("*") {
            PathPattern::Wildcard(path[..path.length-1].to_string())
        } else {
            PathPattern::Static(path.to_string())
        }
    }
    
    micro matches(self, path: utf8) -> bool {
        match self {
            PathPattern::Static(pattern) => pattern == path,
            PathPattern::Dynamic(pattern, _) => {
                let regex = Regex::new(pattern).unwrap()
                regex.is_match(path)
            },
            PathPattern::Wildcard(prefix) => path.starts_with(prefix),
        }
    }
    
    micro extract_params(self, path: utf8) -> HashMap<utf8, utf8> {
        let mut params = HashMap::new()
        
        if let PathPattern::Dynamic(pattern, param_names) = self {
            let regex = Regex::new(pattern).unwrap()
            if let Some(captures) = regex.captures(path) {
                for (i, name) in param_names.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        params.insert(name.clone(), value.as_str().to_string())
                    }
                }
            }
        }
        
        params
    }
}
```

### 請求和響應處理

```valkyrie
# HTTP 請求
class HttpRequest {
    method: HttpMethod
    path: utf8
    query_params: HashMap<utf8, utf8>
    headers: HashMap<utf8, utf8>
    body: [u8]
    params: HashMap<utf8, utf8>  # 路由參數
}

#[derive(Clone, PartialEq)]
union HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

imply HttpRequest {
    micro parse(buffer: [u8]) -> Result<Self, ParseError> {
        let request_str = utf8::from_utf8_lossy(buffer)
        let lines: [utf8] = request_str.lines().collect()
        
        if lines.is_empty() {
            return Fail { error: ParseError::InvalidRequest }
        }
        
        # 解析請求行
        let request_line_parts: [utf8] = lines[0].split_whitespace().collect()
        if request_line_parts.length < 3 {
            return Fail { error: ParseError::InvalidRequestLine }
        }
        
        let method = HttpMethod::from_str(request_line_parts[0])?
        let url_parts: [utf8] = request_line_parts[1].splitn(2, '?').collect()
        let path = url_parts[0].to_string()
        
        # 解析查詢參數
        let query_params = if url_parts.length > 1 {
            parse_query_string(url_parts[1])
        } else {
            HashMap::new()
        }
        
        # 解析頭部
        let mut headers = HashMap::new()
        let mut body_start = 1
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1
                break
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase()
                let value = line[colon_pos + 1..].trim().to_string()
                headers.insert(key, value)
            }
        }
        
        # 解析請求體
        let body = if body_start < lines.length {
            lines[body_start..].join("\n").into_bytes()
        } else {
            []
        }
        
        Ok(HttpRequest {
            method,
            path,
            query_params,
            headers,
            body,
            params: HashMap::new(),
        })
    }
    
    micro get_header(self, name: utf8) -> utf8? {
        self.headers.get(name.to_lowercase())
    }
    
    micro get_param(self, name: utf8) -> utf8? {
        self.params.get(name)
    }
    
    micro get_query(self, name: utf8) -> utf8? {
        self.query_params.get(name)
    }
    
    micro json<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body)
    }
    
    micro form_data(self) -> HashMap<utf8, utf8> {
        if let Some(content_type) = self.get_header("content-type") {
            if content_type.contains("application/x-www-form-urlencoded") {
                let body_str = utf8::from_utf8_lossy(self.body)
                return parse_query_string(body_str)
            }
        }
        HashMap::new()
    }
}

# HTTP 響應
class HttpResponse {
    status_code: u16
    status_text: utf8
    headers: HashMap<utf8, utf8>
    body: [u8]
}

imply HttpResponse {
    micro new(status_code: u16) -> Self {
        let status_text = match status_code {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }.to_string()
        
        HttpResponse {
            status_code,
            status_text,
            headers: HashMap::new(),
            body: [],
        }
    }
    
    micro ok() -> Self {
        Self::new(200)
    }
    
    micro created() -> Self {
        Self::new(201)
    }
    
    micro bad_request() -> Self {
        Self::new(400)
    }
    
    micro not_found() -> Self {
        Self::new(404)
    }
    
    micro internal_error() -> Self {
        Self::new(500)
    }
    
    micro with_header(mut self, name: utf8, value: utf8) -> Self {
        self.headers.insert(name.to_string(), value.to_string())
        self
    }
    
    micro with_json<T: Serialize>(mut self, data: T) -> Result<Self, serde_json::Error> {
        let json_str = serde_json::to_string(data)?
        self.body = json_str.into_bytes()
        self.headers.insert("Content-Type".to_string(), "application/json".to_string())
        Ok(self)
    }
    
    micro with_html(mut self, html: utf8) -> Self {
        self.body = html.as_bytes().to_vec()
        self.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string())
        self
    }
    
    micro with_text(mut self, text: utf8) -> Self {
        self.body = text.as_bytes().to_vec()
        self.headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())
        self
    }
    
    micro redirect(location: utf8) -> Self {
        Self::new(302).with_header("Location", location)
    }
    
    micro to_bytes(self) -> [u8] {
        let mut response = format(
            "HTTP/1.1 {} {}\r\n",
            self.status_code,
            self.status_text
        )
        
        # 添加頭部
        for (key, value) in self.headers {
            response.push_str(format("{}:{}\r\n", key, value))
        }
        
        # 添加 Content-Length
        response.push_str(format("Content-Length: {}\r\n", self.body.length))
        response.push_str("\r\n")
        
        let mut bytes = response.into_bytes()
        bytes.extend_from_slice(self.body)
        bytes
    }
}
```

## 中間件系統

### 中間件接口

```valkyrie
# 中間件特徵
trait Middleware: Send + Sync {
    async micro process(self, mut context: RequestContext) -> Result<(), MiddlewareError>
}

class RequestContext {
    request: HttpRequest
    response: HttpResponse?
    data: HashMap<utf8, Box<Any + Send + Sync>>
}

imply RequestContext {
    micro new(request: HttpRequest) -> Self {
        RequestContext {
            request,
            response: None,
            data: HashMap::new(),
        }
    }
    
    micro set_data<T: Any + Send + Sync>(mut self, key: utf8, value: T) {
        self.data.insert(key.to_string(), Box::new(value))
    }
    
    micro get_data<T: Any + Send + Sync>(self, key: utf8) -> T? {
        self.data.get(key)?.downcast_ref::<T>()
    }
}

# 日誌中間件
class LoggingMiddleware {
    format: utf8
}

imply LoggingMiddleware {
    micro new() -> Self {
        LoggingMiddleware {
            format: "{method} {path} - {status} ({duration}ms)".to_string(),
        }
    }
}

imply LoggingMiddleware: Middleware {
    async micro process(self, mut context: RequestContext) -> Result<(), MiddlewareError> {
        let start_time = std::time::Instant::now()
        
        # 記錄請求開始
        print("[{}] {} {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            context.request.method.as_str(),
            context.request.path
        )
        
        # 在響應數據中存儲開始時間
        context.set_data("start_time", start_time)
        
        Ok(())
    }
}

# CORS 中間件
class CorsMiddleware {
    allowed_origins: [utf8]
    allowed_methods: [HttpMethod]
    allowed_headers: [utf8]
    max_age: u32
}

imply CorsMiddleware {
    micro new() -> Self {
        CorsMiddleware {
            allowed_origins: ["*".to_string()],
            allowed_methods: [HttpMethod::GET, HttpMethod::POST, HttpMethod::PUT, HttpMethod::DELETE],
            allowed_headers: ["Content-Type".to_string(), "Authorization".to_string()],
            max_age: 86400,
        }
    }
    
    micro allow_origin(mut self, origin: utf8) -> Self {
        self.allowed_origins = [origin.to_string()]
        self
    }
    
    micro allow_methods(mut self, methods: [HttpMethod]) -> Self {
        self.allowed_methods = methods
        self
    }
}

imply CorsMiddleware: Middleware {
    async micro process(self, mut context: RequestContext) -> Result<(), MiddlewareError> {
        # 處理預檢請求
        if context.request.method == HttpMethod::OPTIONS {
            let response = HttpResponse::ok()
                .with_header("Access-Control-Allow-Origin", self.allowed_origins.join(", "))
                .with_header("Access-Control-Allow-Methods", 
                    self.allowed_methods.iter().map { $.as_str() }.collect::<[_]>().join(", "))
                .with_header("Access-Control-Allow-Headers", self.allowed_headers.join(", "))
                .with_header("Access-Control-Max-Age", self.max_age.to_string())
            
            context.response = Some(response)
            return Ok(())
        }
        
        # 為其他請求添加 CORS 頭
        if let Some(mut response) = context.response {
            response.headers.insert("Access-Control-Allow-Origin".to_string(), 
                self.allowed_origins.join(", "))
        }
        
        Ok(())
    }
}

# 认證中間件
class AuthMiddleware {
    secret_key: utf8,
    protected_paths: [utf8],
}

imply AuthMiddleware {
    micro new(secret_key: utf8) -> Self {
        AuthMiddleware {
            secret_key: secret_key.to_string(),
            protected_paths: [],
        }
    }
    
    micro protect_path(mut self, path: utf8) -> Self {
        self.protected_paths.push(path.to_string())
        self
    }
    
    micro verify_token(self, token: utf8) -> Result<Claims, AuthError> {
        # JWT 令牌驗證邏輯
        jwt::decode::<Claims>(
            token,
            DecodingKey::from_secret(self.secret_key.as_ref()),
            Validation::default()
        ).map { $claims }
        .map_err { AuthError::InvalidToken }
    }
}

imply AuthMiddleware: Middleware {
    async micro process(self, mut context: RequestContext) -> Result<(), MiddlewareError> {
        # 檢查路徑是否需要保護
        let needs_auth = self.protected_paths.iter()
            .any { context.request.path.starts_with($) }
        
        if !needs_auth {
            return Ok(())
        }
        
        # 提取认證令牌
        let token = context.request.get_header("authorization")
            .and_then { $.strip_prefix("Bearer ") }
            .ok_or(MiddlewareError::Unauthorized)?;
        
        # 驗證令牌
        match self.verify_token(token) {
            Ok(claims) => {
                context.set_data("user_claims", claims)
                Ok(())
            },
            Err(_) => {
                context.response = Some(HttpResponse::new(401).with_text("Unauthorized"))
                Ok(())
            }
        }
    }
}
```

## 模板引擎

### HTML 模板系統

```valkyrie
# 模板引擎
class TemplateEngine {
    templates: HashMap<utf8, Template>,
    template_dir: utf8,
    cache_enabled: bool,
}

class Template {
    name: utf8,
    content: utf8,
    compiled: CompiledTemplate,
}

class CompiledTemplate {
    blocks: [TemplateBlock],
}

enums TemplateBlock {
    Text(utf8),
    Variable(utf8),
    Loop {
        variable: utf8,
        items: utf8,
        body: [TemplateBlock],
    },
    Condition {
        expression: utf8,
        then_body: [TemplateBlock],
        else_body: [TemplateBlock]?,
    },
    Include(utf8),
}

imply TemplateEngine {
    micro new(template_dir: utf8) -> Self {
        TemplateEngine {
            templates: HashMap::new(),
            template_dir: template_dir.to_string(),
            cache_enabled: true,
        }
    }
    
    micro load_template(mut self, name: utf8) -> Result<(), TemplateError> {
        let path = format("{}/{}.html", self.template_dir, name)
        let content = std::fs::read_to_string(path)
            .map_err { TemplateError::TemplateNotFound(name.to_string()) }?;
        
        let compiled = self.compile_template(content)?;
        
        let template = Template {
            name: name.to_string(),
            content,
            compiled,
        }
        
        self.templates.insert(name.to_string(), template)
        Ok(())
    }
    
    micro render(mut self, name: utf8, context: TemplateContext) -> Result<utf8, TemplateError> {
        if !self.templates.contains_key(name) {
            self.load_template(name)?
        }
        
        let template = self.templates.get(name).unwrap()
        self.render_blocks(template.compiled.blocks, context)
    }
    
    micro compile_template(self, content: utf8) -> Result<CompiledTemplate, TemplateError> {
        let mut blocks = []
        let mut chars = content.chars().peekable()
        let mut current_text = utf8::new()
        
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next()  # 消費第二個 '{'
                
                # 保存當前文本塊
                if !current_text.is_empty() {
                    blocks.push(TemplateBlock::Text(current_text))
                    current_text = utf8::new()
                }
                
                # 解析變量
                let mut var_name = utf8::new()
                while let Some(v_ch) = chars.next() {
                    if v_ch == '}' && chars.peek() == Some(&'}') {
                        chars.next()
                        break
                    }
                    var_name.push(v_ch)
                }
                blocks.push(TemplateBlock::Variable(var_name.trim().to_string()))
            } else {
                current_text.push(ch)
            }
        }
        
        if !current_text.is_empty() {
            blocks.push(TemplateBlock::Text(current_text))
        }
        
        Ok(CompiledTemplate { blocks })
    }
    
    micro render_blocks(self, blocks: [TemplateBlock], context: TemplateContext) -> Result<string, TemplateError> {
        let mut result = String::new()
        loop block in blocks {
            match block {
                TemplateBlock::Text(text) => result.push_str(text),
                TemplateBlock::Variable(name) => {
                    if let Some(value) = context.get(name) {
                        result.push_str(value)
                    }
                }
                _ => {} # 其他塊的渲染邏輯
            }
        }
        Ok(result)
    }
}
```
