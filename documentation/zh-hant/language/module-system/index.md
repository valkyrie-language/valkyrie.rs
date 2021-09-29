# 模塊系統

Valkyrie 採用基於命名空間（namespace）和匯入（using）的模塊系統，提供靈活的程式碼組織和依賴管理方式。

## 命名空間宣告 (namespace)

Valkyrie 使用 `namespace!` 關鍵字在檔案頂部宣告命名空間。

```valkyrie
namespace! math.geometry;

class Point {
    x: f64
    y: f64
}

imply Point {
    micro distance(self, other: Point) -> f64 {
        let dx = self.x - other.x
        let dy = self.y - other.y
        (dx * dx + dy * dy).sqrt()
    }
}
```

## 匯入系統 (using)

使用 `using` 關鍵字匯入其他命名空間的成員。

```valkyrie
using math.geometry;
using math.geometry.Point;
```

## 可見性控制

### 存取修飾詞

```valkyrie
namespace database {
    # 公開結構體
    pub class Connection {
        # 私有欄位
        host: string,
        port: u16,
        # 公開欄位
        pub timeout: Duration,
    }
    
    # 包內可見
    class InternalConfig {
        secret_key: string,
    }
    
    # 私有函式
    micro validate_connection(conn: Connection) -> bool {
        # 內部驗證邏輯
        true
    }
    
    # 公開函式
    pub micro connect(host: string, port: u16) -> Result⟨Connection, Error⟩ {
        let conn = Connection(host, port, timeout: Duration::seconds(30))
        if validate_connection(conn) {
            Fine(conn)
        } else {
            Fail(Error::InvalidConnection)
        }
    }
}
```

### 重新匯出

```valkyrie
namespace api {
    # 重新匯出其他模塊的型別
    pub using database.{Connection, Error}
    pub using auth.{User, Session}
    
    # 提供統一的 API 介面
    pub micro create_authenticated_connection(credentials: Credentials) -> Result⟨(Connection, Session), Error⟩ {
        let session = auth::login(credentials)?
        let connection = database::connect("localhost", 5432)?
        Fine(connection, session)
    }
}
```

## 外部函式介面 (FFI)

Valkyrie 支援與 C、C++、Rust、Python 和 JavaScript 的互操作。

詳細內容請參考：[外部函式介面](foreign-function.md)

## 檔案路徑無關的模塊

### 邏輯模塊組織

Valkyrie 的模塊系統不依賴檔案路徑，而是基於邏輯命名空間：

```valkyrie
# 檔案: src/geometry.val
namespace math.geometry {
    pub class Point { x: f64, y: f64 }
}

# 檔案: src/algebra.val  
namespace math.algebra {
    pub class Matrix { data: [[f64]] }
}

# 檔案: src/utils.val
namespace math.geometry {  # 擴充已存在的命名空間
    pub micro origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
}
```

### 模塊宣告檔案

```valkyrie
# 檔案: math.module.val
# 宣告模塊的公開介面
module math {
    pub namespace geometry {
        pub class Point
        pub micro distance(Point, Point) -> f64
        pub micro origin() -> Point
    }
    
    pub namespace algebra {
        pub class Matrix
        pub micro multiply(Matrix, Matrix) -> Matrix
    }
}
```

## 條件編譯和特性

### 特性閘控

```valkyrie
namespace network {
    # 基礎網路功能
    pub class TcpStream { /* ... */ }
    
    # 非同步功能（需要 async 特性）
    @cfg(feature = "async")
    pub namespace async {
        pub class AsyncTcpStream { /* ... */ }
        
        pub micro connect_async(addr: SocketAddr) -> Future<Result<AsyncTcpStream, Error>> {
            # 非同步連接實作
        }
    }
    
    # TLS 支援（需要 tls 特性）
    @cfg(feature = "tls")
    pub namespace tls {
        pub class TlsStream { /* ... */ }
        
        pub micro wrap_tls(stream: TcpStream, config: TlsConfig) -> Result<TlsStream, TlsError> {
            # TLS 包裝實作
        }
    }
}
```

### 平台特定程式碼

```valkyrie
namespace platform {
    # 通用介面
    pub trait FileSystem {
        micro read_file(path: String) -> Result<String, IoError>
        micro write_file(path: String, content: String) -> Result<(), IoError>
    }
    
    # Windows 實作
    @cfg(target_os = "windows")
    pub namespace windows {
        pub class WindowsFileSystem
        
        imply WindowsFileSystem: FileSystem {
            micro read_file(self, path: String) -> Result<String, IoError> {
                # Windows 特有的實作
                Fine("Windows content".to_string())
            }
        }
    }
    
    # Unix 實作
    @cfg(any(target_os = "linux", target_os = "macos"))
    pub namespace unix {
        pub class UnixFileSystem
        
        imply UnixFileSystem: FileSystem {
            micro read_file(self, path: String) -> Result<String, IoError> {
                # Unix 特有的實作
                Fine("Unix content".to_string())
            }
        }
    }
}
```

## 依賴管理

### 外部依賴

```valkyrie
# 專案設定檔: valkyrie.toml
[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
log = "0.4"

[dev-dependencies]
tokio-test = "0.4"

# 在程式碼中使用外部依賴
using serde.{Serialize, Deserialize}
using tokio.runtime.Runtime
using log.{info, warn, error}

@derive(Serialize, Deserialize)
class Config {
    host: String,
    port: u16,
}

micro main() {
    async_main().block
}

micro async_main() {
    info("Starting application")
    # 非同步邏輯
}
```

### 工作空間

```valkyrie
# 工作空間設定: Workspace.toml
[workspace]
members = [
    "core",
    "api",
    "cli",
    "web"
]

# 共用依賴
[workspace.dependencies]
serde = "1.0"
tokio = "1.0"

# 在子專案中引用工作空間依賴
# core/valkyrie.toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }

# 內部依賴
api = { path = "../api" }
```

## 模塊初始化

### 靜態初始化

```valkyrie
namespace config {
    # 靜態配置
    pub static DATABASE_URL: String = env("DATABASE_URL")
    pub static MAX_CONNECTIONS: i32 = 100
    
    # 延遲初始化
    pub static LOGGER: Lazy<Logger> = Lazy::new({
        Logger::new()
            .with_level(LogLevel::Info)
            .with_output(Output::Stdout)
    })
}
```

### 動態初始化

```valkyrie
namespace database {
    static mut CONNECTION_POOL: Option<ConnectionPool> = None
    
    pub micro initialize(config: DatabaseConfig) -> Result<(), Error> {
        unsafe {
            if CONNECTION_POOL.is_some() {
                return Fail(Error::AlreadyInitialized)
            }
            
            let pool = ConnectionPool::new(config)?
            CONNECTION_POOL = Some(pool)
            Fine { value: () }
        }
    }
    
    pub micro get_connection() -> Result<Connection, Error> {
        unsafe {
            CONNECTION_POOL
                .as_ref()
                .ok_or(Error::NotInitialized)?
                .get_connection()
        }
    }
}
```

## 測試模塊

### 單元測試

```valkyrie
namespace math.geometry {
    pub micro distance(p1: Point, p2: Point) -> f64 {
        let dx = p1.x - p2.x
        let dy = p1.y - p2.y
        (dx * dx + dy * dy).sqrt()
    }
    
    # 測試模塊
    @cfg(test)
    namespace tests {
        using super.*
        
        @test
        micro test_distance_same_point() {
            let p = Point { x: 1.0, y: 2.0 }
            let dist = distance(p, p)
            @assert_equal(dist, 0.0)
        }
        
        @test
        micro test_distance_different_points() {
            let p1 = Point { x: 0.0, y: 0.0 }
            let p2 = Point { x: 3.0, y: 4.0 }
            let dist = distance(p1, p2)
            @assert_equal(dist, 5.0)
        }
    }
}
```

### 整合測試

```valkyrie
# 檔案: tests/integration.val
using myapp.api.*
using myapp.database.*

@test
micro test_full_workflow() {
    # 設定測試環境
    let config = TestConfig::default()
    initialize_test_database(config)
    
    # 執行測試
    let user = create_user("alice", "alice@example.com")
    assert_true(user.is_ok())
    
    let found_user = find_user_by_email("alice@example.com")
    assert_true(found_user.is_some())
    
    # 清理
    cleanup_test_database()
}
```

## 最佳實踐

### 模塊設計原則

1. **單一職責**: 每個模塊應該有明確的職責
2. **低耦合**: 模塊間依賴應該最小化
3. **高內聚**: 相關功能應該組織在同一模塊中
4. **介面穩定**: 公開介面應該保持穩定

### 命名約定

```valkyrie
# 好的命名空間組織
namespace myapp {
    namespace core {        # 核心功能
        namespace types     # 基礎型別
        namespace traits    # 特徵定義
        namespace utils     # 工具函式
    }
    
    namespace services {    # 業務服務
        namespace user      # 使用者服務
        namespace auth      # 認證服務
        namespace payment   # 支付服務
    }
    
    namespace adapters {    # 適配器層
        namespace database  # 資料庫適配器
        namespace http      # HTTP 適配器
        namespace cache     # 快取適配器
    }
}
```

### 版本相容性

```valkyrie
namespace api {
    # 版本化 API
    namespace v1 {
        pub class User {
            id: i64,
            name: String,
        }
        
        pub micro get_user(id: i64) -> Option<User> {
            # v1 實作
        }
    }
    
    namespace v2 {
        pub class User {
            id: i64,
            name: String,
            email: String,  # 新增欄位
        }
        
        pub micro get_user(id: i64) -> Option<User> {
            # v2 實作
        }
        
        # 向後相容
        pub micro get_user_v1(id: i64) -> Option<v1::User> {
            get_user(id).map({ v1::User {
                id: u.id,
                name: u.name,
            })
        }
    }
    
    # 當前版本別名
    pub using v2.*
}
```
