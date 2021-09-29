# 控制反转 (Inversion of Control)

控制反转（IoC）是一种设计原则，通过将对象的创建和依赖关系的管理从对象本身转移到外部容器或框架中，实现了松耦合的设计。在 Valkyrie 中，IoC 通过 Effect 系统实现，提供了强大的依赖注入和服务定位能力。

## 基本概念

### 依赖注入 (Dependency Injection)
依赖注入是 IoC 的一种实现方式，通过外部注入依赖对象，而不是在对象内部创建依赖。

### 服务定位 (Service Locator)
服务定位器是一个中央注册表，用于查找和获取服务实例。

### 生命周期管理
容器负责管理对象的生命周期，包括创建、初始化、销毁等。

## Effect 系统中的 IoC

### 依赖注入请求结构

```valkyrie
structure ServiceRequest {
    service_type: Type
}

structure NamedServiceRequest {
    service_type: Type
    name: utf8
}

structure ServiceRegister {
    service_type: Type
    instance: Any
}

structure ServiceRegisterFactory {
    service_type: Type
    factory: { -> Any }
}

structure ServiceRegisterSingleton {
    service_type: Type
    factory: { -> Any }
}

structure ServiceCreate {
    service_type: Type
}

structure ServiceInitialize {
    instance: Any
}

structure ServiceDispose {
    instance: Any
}
```

### 实现 IoC 容器

```valkyrie
class IoCContainer {
    private mut services: {Type: Any} = {}
    private mut factories: {Type: { -> Any }} = {}
    private mut singletons: {Type: Any} = {}
    private mut named_services: {utf8: {Type: Any}} = {}
    
    micro resolve<T>(self, service_type: Type<T>) -> T {
        try {
            raise ServiceRequest { service_type: service_type }
        }.catch {
            case ServiceRequest { service_type }: {
                match self.singletons.get(service_type) {
                    case instance: resume instance as T
                    case null: {}
                }
                
                match self.factories.get(service_type) {
                    case factory: resume factory() as T
                    case null: {}
                }
                
                match self.services.get(service_type) {
                    case instance: resume instance as T
                    case null: {}
                }
                
                resume self.auto_wire(service_type)
            }
        }
    }
    
    micro resolve_named<T>(self, service_type: Type<T>, name: utf8) -> T {
        try {
            raise NamedServiceRequest { service_type: service_type, name: name }
        }.catch {
            case NamedServiceRequest { service_type, name }: {
                match self.named_services.get(name) {
                    case named_map:
                        match named_map.get(service_type) {
                            case instance: resume instance as T
                            case null: {}
                        }
                    case null: {}
                }
                
                raise ServiceNotFoundError { service_type, name }
            }
        }
    }
    
    micro register<T>(mut self, service_type: Type<T>, instance: T) {
        self.services[service_type] = instance
    }
    
    micro register_factory<T>(mut self, service_type: Type<T>, factory: { -> T }) {
        self.factories[service_type] = factory
    }
    
    micro register_singleton<T>(mut self, service_type: Type<T>, factory: { -> T }) {
        let instance = factory()
        self.singletons[service_type] = instance
    }
    
    micro create<T>(self, service_type: Type<T>) -> T {
        try {
            raise ServiceCreate { service_type: service_type }
        }.catch {
            case ServiceCreate { service_type }: {
                let constructor = service_type.get_constructor()
                let dependencies = constructor.get_parameters().map { 
                    self.resolve(%.type)
                }
                resume constructor.invoke(dependencies)
            }
        }
    }
    
    micro initialize<T>(self, instance: T) -> T {
        try {
            raise ServiceInitialize { instance: instance }
        }.catch {
            case ServiceInitialize { instance }: {
                if instance implements Initializable {
                    instance.initialize()
                }
                resume instance
            }
        }
    }
    
    micro dispose<T>(self, instance: T) {
        try {
            raise ServiceDispose { instance: instance }
        }.catch {
            case ServiceDispose { instance }: {
                if instance implements Disposable {
                    instance.dispose()
                }
            }
        }
    }
    
    private micro auto_wire<T>(self, service_type: Type<T>) -> T {
        let instance = self.create(service_type)
        let initialized = self.initialize(instance)
        self.services[service_type] = initialized
        initialized
    }
}
```

### 使用依赖注入注解

```valkyrie
trait UserRepository {
    micro find_by_id(self, id: utf8) -> User?
    micro save(self, user: User) -> Unit
    micro delete(self, id: utf8) -> Unit
}

trait EmailService {
    micro send_email(self, to: utf8, subject: utf8, body: utf8) -> Unit
}

class DatabaseUserRepository {
    private connection: DatabaseConnection
    
    new(@inject connection: DatabaseConnection) {
        self.connection = connection
    }
    
    impl UserRepository {
        micro find_by_id(self, id) -> User? {
            self.connection.query("SELECT * FROM users WHERE id = ?", [id])
                .map { User::from_row(%) }
        }
        
        micro save(self, user) {
            self.connection.execute(
                "INSERT INTO users (id, name, email) VALUES (?, ?, ?)",
                [user.id, user.name, user.email]
            )
        }
        
        micro delete(self, id) {
            self.connection.execute("DELETE FROM users WHERE id = ?", [id])
        }
    }
}

class SmtpEmailService {
    private config: EmailConfig
    
    new(@inject config: EmailConfig) {
        self.config = config
    }
    
    impl EmailService {
        micro send_email(self, to, subject, body) {
            let smtp_client = SmtpClient::new(self.config)
            smtp_client.send(to, subject, body)
        }
    }
}
```

### 服务类使用依赖注入

```valkyrie
class UserService {
    private user_repository: UserRepository
    private email_service: EmailService
    
    new(
        @inject user_repository: UserRepository,
        @inject email_service: EmailService
    ) {
        self.user_repository = user_repository
        self.email_service = email_service
    }
    
    @transactional
    micro create_user(self, name: utf8, email: utf8) -> User {
        let user = User {
            id: generate_id(),
            name,
            email,
            created_at: now()
        }
        
        self.user_repository.save(user)
        
        self.email_service.send_email(
            user.email,
            "Welcome!",
            f"Welcome {user.name}!"
        )
        
        user
    }
    
    micro get_user(self, id: utf8) -> User? {
        self.user_repository.find_by_id(id)
    }
    
    @authorized("admin")
    micro delete_user(self, id: utf8) -> Unit {
        if let user = self.user_repository.find_by_id(id) {
            self.user_repository.delete(id)
            
            self.email_service.send_email(
                user.email,
                "Account Deleted",
                "Your account has been deleted."
            )
        }
    }
}
```

### 配置和启动

```valkyrie
class ApplicationConfig {
    micro configure_services(self, container: IoCContainer) {
        container.register(EmailConfig, EmailConfig {
            smtp_host: "smtp.example.com",
            smtp_port: 587,
            username: "app@example.com",
            password: "password"
        })
        
        container.register_singleton(DatabaseConnection, {
            DatabaseConnection::new("postgresql://localhost/myapp")
        })
        
        container.register_factory(UserRepository, {
            let connection = container.resolve(DatabaseConnection)
            DatabaseUserRepository::new(connection)
        })
        
        container.register_factory(EmailService, {
            let config = container.resolve(EmailConfig)
            SmtpEmailService::new(config)
        })
        
        container.register_factory(UserService, {
            let user_repo = container.resolve(UserRepository)
            let email_service = container.resolve(EmailService)
            UserService::new(user_repo, email_service)
        })
    }
}

class Application {
    private container: IoCContainer
    
    new() {
        self.container = IoCContainer {}
        let config = ApplicationConfig {}
        config.configure_services(self.container)
    }
    
    micro run(self) {
        with self.container {
            let user_service = self.container.resolve(UserService)
            
            let user = user_service.create_user("Alice", "alice@example.com")
            print(f"Created user: {user.id}")
            
            let found_user = user_service.get_user(user.id)
            print(f"Found user: {found_user}")
        }
    }
}
```

### 作用域和生命周期

```valkyrie
unite ServiceScope {
    Singleton,
    Scoped,
    Transient,
}

class ScopeManager {
    private mut scoped_services: {utf8: {Type: Any}} = {}
    private mut current_scope: utf8? = null
    
    micro begin_scope(mut self, scope_id: utf8) {
        self.current_scope = scope_id
        self.scoped_services[scope_id] = {}
    }
    
    micro end_scope(mut self, scope_id: utf8, container: IoCContainer) {
        if let services = self.scoped_services.remove(scope_id) {
            for (_, service) in services {
                container.dispose(service)
            }
        }
        
        if self.current_scope == scope_id {
            self.current_scope = null
        }
    }
    
    micro get_scoped_service<T>(self, service_type: Type<T>) -> T? {
        if let scope_id = self.current_scope {
            if let services = self.scoped_services.get(scope_id) {
                services.get(service_type).map { %service -> %service as T }
            } else {
                null
            }
        } else {
            null
        }
    }
    
    micro set_scoped_service<T>(mut self, service_type: Type<T>, instance: T) {
        if let scope_id = self.current_scope {
            self.scoped_services.get_mut(scope_id)[service_type] = instance
        }
    }
}
```

### 条件注册

```valkyrie
class ConditionalRegistration {
    micro register_if<T>(
        self,
        container: IoCContainer,
        service_type: Type<T>,
        factory: { -> T },
        condition: { -> bool }
    ) {
        if condition() {
            container.register_factory(service_type, factory)
        }
    }
    
    micro register_profile<T>(
        self,
        container: IoCContainer,
        service_type: Type<T>,
        implementations: {utf8: { -> T }},
        active_profile: utf8
    ) {
        if let factory = implementations.get(active_profile) {
            container.register_factory(service_type, factory)
        }
    }
}

let conditional = ConditionalRegistration {}
let container = IoCContainer {}

conditional.register_profile(
    container,
    EmailService,
    {
        "development": { MockEmailService {} },
        "production": { SmtpEmailService::new(email_config) },
        "testing": { InMemoryEmailService {} }
    },
    get_active_profile()
)
```

### 装饰器模式

```valkyrie
class ServiceDecorator<T> {
    private inner: T
    
    new(inner: T) {
        self.inner = inner
    }
    
    micro get_inner(self) -> T {
        self.inner
    }
}

class CachedUserRepository {
    private inner: UserRepository
    private mut cache: {utf8: User} = {}
    
    new(@inject inner: UserRepository) {
        self.inner = inner
    }
    
    impl UserRepository {
        micro find_by_id(self, id) -> User? {
            if let cached = self.cache.get(id) {
                return cached
            }
            
            let user = self.inner.find_by_id(id)
            if let u = user {
                self.cache[id] = u
            }
            user
        }
        
        micro save(self, user) {
            self.inner.save(user)
            self.cache[user.id] = user
        }
        
        micro delete(self, id) {
            self.inner.delete(id)
            self.cache.remove(id)
        }
    }
}

container.register_factory(UserRepository, {
    let base_repo = DatabaseUserRepository::new(
        container.resolve(DatabaseConnection)
    )
    CachedUserRepository::new(base_repo)
})
```

## 最佳实践

### 1. 接口隔离
定义小而专注的接口，避免大而全的接口。

### 2. 单一职责
每个服务应该只有一个职责，避免服务过于复杂。

### 3. 生命周期管理
合理选择服务的生命周期，避免内存泄漏和资源浪费。

### 4. 循环依赖检测
在容器中实现循环依赖检测，避免无限递归。

```valkyrie
class WebApplication {
    private container: IoCContainer
    private scope_manager: ScopeManager
    
    new() {
        self.container = IoCContainer {}
        self.scope_manager = ScopeManager {}
        self.configure_services()
    }
    
    private micro configure_services(self) {
        self.container.register_singleton(DatabaseConnection, {
            DatabaseConnection::new(get_connection_string())
        })
        
        self.container.register_factory(UserRepository, {
            let conn = self.container.resolve(DatabaseConnection)
            let base_repo = DatabaseUserRepository::new(conn)
            CachedUserRepository::new(base_repo)
        })
        
        self.container.register_scoped(UserService, {
            let user_repo = self.container.resolve(UserRepository)
            let email_service = self.container.resolve(EmailService)
            UserService::new(user_repo, email_service)
        })
        
        self.container.register_scoped(UserController, {
            let user_service = self.container.resolve(UserService)
            UserController::new(user_service)
        })
    }
    
    micro handle_request(self, request: HttpRequest) -> HttpResponse {
        let request_id = generate_request_id()
        
        self.scope_manager.begin_scope(request_id)
        
        try {
            with self.container, self.scope_manager {
                let controller = self.container.resolve(UserController)
                controller.handle(request)
            }
        } finally {
            self.scope_manager.end_scope(request_id, self.container)
        }
    }
}
```

通过 Effect 系统实现的 IoC 提供了类型安全、灵活配置、易于测试的依赖注入能力，使得应用程序的架构更加清晰和可维护。
