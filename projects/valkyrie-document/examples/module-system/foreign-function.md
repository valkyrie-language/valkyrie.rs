# 外部函数接口 (Foreign Function Interface)

Valkyrie 提供了强大的 FFI 系统，支持与 C、C++、Rust、Python、JavaScript 等多种语言的互操作，实现高效的跨语言调用和数据交换。

## C/C++ 互操作

### 基本 C 函数调用

```valkyrie
# 声明外部 C 函数
extern "C" {
    fn malloc(size: usize) -> *mut u8
    fn free(ptr: *mut u8)
    fn strlen(s: *const i8) -> usize
    fn printf(format: *const i8, ...) -> i32
    fn sin(x: f64) -> f64
    fn cos(x: f64) -> f64
}

# 使用 C 函数
fn use_c_functions() {
    unsafe {
        let ptr = malloc(1024)
        if !ptr.is_null() {
            # 使用内存
            free(ptr)
        }
        
        let result = sin(3.14159 / 2.0)
        println!("sin(π/2) = {}", result)
    }
}
```

### C 结构体互操作

```valkyrie
# C 兼容的结构体
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

# 声明使用结构体的 C 函数
extern "C" {
    fn calculate_distance(p1: *const Point, p2: *const Point) -> f64
    fn rectangle_area(rect: *const Rectangle) -> f64
    fn create_point(x: f64, y: f64) -> Point
}

# 使用结构体与 C 交互
fn geometry_calculations() {
    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }
    
    unsafe {
        let distance = calculate_distance(&p1, &p2)
        println!("Distance: {}", distance)
        
        let rect = Rectangle {
            top_left: Point { x: 0.0, y: 10.0 },
            bottom_right: Point { x: 5.0, y: 0.0 }
        }
        let area = rectangle_area(&rect)
        println!("Area: {}", area)
    }
}
```

### C++ 类互操作

```valkyrie
# C++ 类的 C 包装器声明
extern "C" {
    # Vector3D 类的 C 接口
    fn vector3d_new(x: f64, y: f64, z: f64) -> *mut c_void
    fn vector3d_delete(ptr: *mut c_void)
    fn vector3d_magnitude(ptr: *const c_void) -> f64
    fn vector3d_normalize(ptr: *mut c_void)
    fn vector3d_dot(ptr1: *const c_void, ptr2: *const c_void) -> f64
    fn vector3d_cross(ptr1: *const c_void, ptr2: *const c_void) -> *mut c_void
}

# Valkyrie 包装器
struct Vector3D {
    ptr: *mut c_void,
}

impl Vector3D {
    fn new(x: f64, y: f64, z: f64) -> Self {
        unsafe {
            Self {
                ptr: vector3d_new(x, y, z)
            }
        }
    }
    
    fn magnitude(&self) -> f64 {
        unsafe { vector3d_magnitude(self.ptr) }
    }
    
    fn normalize(&mut self) {
        unsafe { vector3d_normalize(self.ptr) }
    }
    
    fn dot(&self, other: &Vector3D) -> f64 {
        unsafe { vector3d_dot(self.ptr, other.ptr) }
    }
    
    fn cross(&self, other: &Vector3D) -> Vector3D {
        unsafe {
            Vector3D {
                ptr: vector3d_cross(self.ptr, other.ptr)
            }
        }
    }
}

impl Drop for Vector3D {
    fn drop(&mut self) {
        unsafe {
            vector3d_delete(self.ptr)
        }
    }
}
```

## Rust 互操作

### 调用 Rust 库

```valkyrie
# 链接 Rust 静态库
#[link(name = "myrust_lib", kind = "static")]
extern "Rust" {
    fn rust_fibonacci(n: u32) -> u64
    fn rust_sort_array(arr: *mut i32, len: usize)
    fn rust_json_parse(json_str: *const i8) -> *mut c_void
    fn rust_json_free(ptr: *mut c_void)
}

# 使用 Rust 函数
fn use_rust_library() {
    unsafe {
        let fib_10 = rust_fibonacci(10)
        println!("Fibonacci(10) = {}", fib_10)
        
        let mut numbers = [5, 2, 8, 1, 9, 3]
        rust_sort_array(numbers.as_mut_ptr(), numbers.len())
        println!("Sorted: {:?}", numbers)
    }
}
```

### 导出函数给其他语言

```valkyrie
# 导出 Valkyrie 函数给 C/C++
#[no_mangle]
extern "C" fn valkyrie_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
extern "C" fn valkyrie_process_array(arr: *mut f64, len: usize) {
    if arr.is_null() { return }
    
    unsafe {
        let slice = std::slice::from_raw_parts_mut(arr, len)
        for item in slice {
            *item = item.sqrt()  # 计算平方根
        }
    }
}

#[no_mangle]
extern "C" fn valkyrie_create_string(s: *const i8) -> *mut i8 {
    if s.is_null() { return std::ptr::null_mut() }
    
    unsafe {
        let c_str = CStr::from_ptr(s)
        let rust_str = c_str.to_str().unwrap_or("")
        let processed = format!("Processed: {}", rust_str)
        
        let c_string = CString::new(processed).unwrap()
        c_string.into_raw()
    }
}

#[no_mangle]
extern "C" fn valkyrie_free_string(s: *mut i8) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s)
        }
    }
}
```

## Python 互操作

### 嵌入 Python 解释器

```valkyrie
# Python C API 绑定
extern "C" {
    fn Py_Initialize()
    fn Py_Finalize()
    fn PyRun_SimpleString(command: *const i8) -> i32
    fn PyImport_ImportModule(name: *const i8) -> *mut c_void
    fn PyObject_CallMethod(obj: *mut c_void, method: *const i8, format: *const i8, ...) -> *mut c_void
    fn PyLong_AsLong(obj: *mut c_void) -> i64
    fn PyFloat_AsDouble(obj: *mut c_void) -> f64
    fn Py_DecRef(obj: *mut c_void)
}

# Python 解释器包装器
struct PythonInterpreter {
    initialized: bool,
}

impl PythonInterpreter {
    fn new() -> Self {
        unsafe {
            Py_Initialize()
        }
        Self { initialized: true }
    }
    
    fn run_string(&self, code: &str) -> Result<(), String> {
        let c_code = CString::new(code).map_err(|e| e.to_string())?;
        unsafe {
            let result = PyRun_SimpleString(c_code.as_ptr())
            if result == 0 {
                Ok(())
            } else {
                Err("Python execution failed".to_string())
            }
        }
    }
    
    fn call_function(&self, module_name: &str, function_name: &str, args: &[f64]) -> Result<f64, String> {
        let c_module = CString::new(module_name).map_err(|e| e.to_string())?;
        let c_function = CString::new(function_name).map_err(|e| e.to_string())?;
        
        unsafe {
            let module = PyImport_ImportModule(c_module.as_ptr());
            if module.is_null() {
                return Err("Failed to import module".to_string())
            }
            
            # 构建参数格式字符串
            let format = "(" + &"d".repeat(args.len()) + ")"
            let c_format = CString::new(format).unwrap();
            
            let result = match args.len() {
                1 => PyObject_CallMethod(module, c_function.as_ptr(), c_format.as_ptr(), args[0]),
                2 => PyObject_CallMethod(module, c_function.as_ptr(), c_format.as_ptr(), args[0], args[1]),
                _ => return Err("Too many arguments".to_string()),
            }
            
            if result.is_null() {
                Err("Function call failed".to_string())
            } else {
                let value = PyFloat_AsDouble(result)
                Py_DecRef(result)
                Ok(value)
            }
        }
    }
}

impl Drop for PythonInterpreter {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                Py_Finalize()
            }
        }
    }
}

# 使用 Python 解释器
fn use_python() {
    let py = PythonInterpreter::new()
    
    # 执行 Python 代码
    py.run_string("import math").unwrap()
    py.run_string("print('Hello from Python!')").unwrap()
    
    # 调用 Python 函数
    let result = py.call_function("math", "sin", &[3.14159 / 2.0]).unwrap()
    println!("Python sin(π/2) = {}", result)
}
```

### Python 扩展模块

```valkyrie
# 创建 Python 扩展模块
use pyo3::prelude::*;

#[pyfunction]
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[pyfunction]
fn matrix_multiply(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
    let rows_a = a.len()
    let cols_a = a[0].len()
    let cols_b = b[0].len()
    
    if cols_a != b.len() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Matrix dimensions don't match"
        ))
    }
    
    let mut result = vec![vec![0.0; cols_b]; rows_a]
    
    for i in 0..rows_a {
        for j in 0..cols_b {
            for k in 0..cols_a {
                result[i][j] += a[i][k] * b[k][j]
            }
        }
    }
    
    Ok(result)
}

#[pyclass]
struct Calculator {
    #[pyo3(get, set)]
    value: f64,
}

#[pymethods]
impl Calculator {
    #[new]
    fn new(initial_value: f64) -> Self {
        Calculator { value: initial_value }
    }
    
    fn add(&mut self, other: f64) -> f64 {
        self.value += other
        self.value
    }
    
    fn multiply(&mut self, other: f64) -> f64 {
        self.value *= other
        self.value
    }
    
    fn reset(&mut self) {
        self.value = 0.0
    }
}

#[pymodule]
fn valkyrie_math(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    m.add_function(wrap_pyfunction!(matrix_multiply, m)?)?;
    m.add_class::<Calculator>()?;
    Ok(())
}
```

## JavaScript 互操作

### WebAssembly 导出

```valkyrie
# 编译到 WebAssembly
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    # 导入 JavaScript 函数
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str)
    
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64
    
    fn alert(s: &str)
}

# 定义宏简化日志
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct GameEngine {
    width: u32,
    height: u32,
    entities: Vec<Entity>,
}

#[wasm_bindgen]
impl GameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> GameEngine {
        console_log!("Creating game engine {}x{}", width, height)
        GameEngine {
            width,
            height,
            entities: Vec::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn add_entity(&mut self, x: f64, y: f64) -> usize {
        let entity = Entity { x, y, vx: 0.0, vy: 0.0 }
        self.entities.push(entity)
        self.entities.len() - 1
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, dt: f64) {
        for entity in &mut self.entities {
            entity.x += entity.vx * dt
            entity.y += entity.vy * dt
            
            # 边界检查
            if entity.x < 0.0 || entity.x > self.width as f64 {
                entity.vx = -entity.vx
            }
            if entity.y < 0.0 || entity.y > self.height as f64 {
                entity.vy = -entity.vy
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn get_entity_positions(&self) -> Vec<f64> {
        let mut positions = Vec::new()
        for entity in &self.entities {
            positions.push(entity.x)
            positions.push(entity.y)
        }
        positions
    }
}

struct Entity {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

# 导出数学函数
#[wasm_bindgen]
pub fn fast_inverse_sqrt(x: f32) -> f32 {
    # Quake III 快速平方根倒数算法
    let i = x.to_bits()
    let i = 0x5f3759df - (i >> 1)
    let y = f32::from_bits(i)
    y * (1.5 - 0.5 * x * y * y)
}

#[wasm_bindgen]
pub fn mandelbrot(cx: f64, cy: f64, max_iter: u32) -> u32 {
    let mut x = 0.0
    let mut y = 0.0
    let mut iter = 0
    
    while x * x + y * y <= 4.0 && iter < max_iter {
        let temp = x * x - y * y + cx
        y = 2.0 * x * y + cy
        x = temp
        iter += 1
    }
    
    iter
}
```

### Node.js 原生模块

```valkyrie
# Node.js N-API 绑定
use napi::bindgen_prelude::*;

#[napi]
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[napi]
fn process_image(data: Buffer, width: u32, height: u32) -> Result<Buffer> {
    let mut pixels = data.to_vec()
    
    # 简单的图像处理：反转颜色
    for i in (0..pixels.len()).step_by(4) {
        pixels[i] = 255 - pixels[i]      # R
        pixels[i + 1] = 255 - pixels[i + 1]  # G
        pixels[i + 2] = 255 - pixels[i + 2]  # B
        # Alpha 通道保持不变
    }
    
    Ok(Buffer::from(pixels))
}

#[napi]
struct FileProcessor {
    buffer_size: u32,
}

#[napi]
impl FileProcessor {
    #[napi(constructor)]
    pub fn new(buffer_size: u32) -> Self {
        FileProcessor { buffer_size }
    }
    
    #[napi]
    pub fn process_file(&self, path: String) -> Result<String> {
        # 文件处理逻辑
        let content = std::fs::read_to_string(&path)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read file: {}", e)))?;
        
        # 简单处理：统计行数和字符数
        let lines = content.lines().count()
        let chars = content.chars().count()
        
        Ok(format!("File: {}, Lines: {}, Characters: {}", path, lines, chars))
    }
    
    #[napi]
    pub async fn process_file_async(&self, path: String) -> Result<String> {
        # 异步文件处理
        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read file: {}", e)))?;
        
        # 模拟耗时处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let processed = content.to_uppercase()
        Ok(processed)
    }
}
```

## 动态库加载

### 运行时动态加载

```valkyrie
use libloading::{Library, Symbol};

# 动态库管理器
struct DynamicLibrary {
    lib: Library,
}

impl DynamicLibrary {
    fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let lib = Library::new(path)?;
            Ok(DynamicLibrary { lib })
        }
    }
    
    fn get_function<T>(&self, name: &[u8]) -> Result<Symbol<T>, Box<dyn std::error::Error>> {
        unsafe {
            let symbol = self.lib.get(name)?;
            Ok(symbol)
        }
    }
}

# 使用动态库
fn use_dynamic_library() -> Result<(), Box<dyn std::error::Error>> {
    let lib = DynamicLibrary::load("./libmath.so")?;
    
    # 获取函数指针
    let add_func: Symbol<unsafe extern fn(i32, i32) -> i32> = lib.get_function(b"add")?;
    let multiply_func: Symbol<unsafe extern fn(f64, f64) -> f64> = lib.get_function(b"multiply")?;
    
    # 调用动态加载的函数
    unsafe {
        let sum = add_func(5, 3)
        let product = multiply_func(2.5, 4.0)
        
        println!("5 + 3 = {}", sum)
        println!("2.5 * 4.0 = {}", product)
    }
    
    Ok(())
}
```

### 插件系统

```valkyrie
# 插件接口定义
trait Plugin {
    fn name(&self) -> &str
    fn version(&self) -> &str
    fn initialize(&mut self) -> Result<(), String>
    fn execute(&self, input: &str) -> Result<String, String>
    fn cleanup(&mut self)
}

# 插件管理器
struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libraries: Vec<Library>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
            libraries: Vec::new(),
        }
    }
    
    fn load_plugin(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let lib = Library::new(path)?;
            
            # 获取插件创建函数
            let create_plugin: Symbol<unsafe extern fn() -> *mut dyn Plugin> = 
                lib.get(b"create_plugin")?;
            
            let plugin_ptr = create_plugin()
            let plugin = Box::from_raw(plugin_ptr)
            
            let name = plugin.name().to_string()
            self.plugins.insert(name, plugin)
            self.libraries.push(lib)
            
            Ok(())
        }
    }
    
    fn execute_plugin(&self, name: &str, input: &str) -> Result<String, String> {
        match self.plugins.get(name) {
            Some(plugin) => plugin.execute(input),
            None => Err(format!("Plugin '{}' not found", name))
        }
    }
    
    fn list_plugins(&self) -> Vec<(String, String)> {
        self.plugins.iter()
            .map(|(name, plugin)| (name.clone(), plugin.version().to_string()))
            .collect()
    }
}

# 插件导出宏
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_plugin() -> *mut dyn Plugin {
            let plugin = Box::new(<$plugin_type>::new());
            Box::into_raw(plugin)
        }
        
        #[no_mangle]
        pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
            if !plugin.is_null() {
                unsafe {
                    let _ = Box::from_raw(plugin);
                }
            }
        }
    }
}
```

## 内存管理和安全性

### 安全的 FFI 包装器

```valkyrie
# 安全的 C 字符串处理
struct SafeCString {
    ptr: *mut i8,
}

impl SafeCString {
    fn new(s: &str) -> Result<Self, std::ffi::NulError> {
        let c_string = CString::new(s)?;
        Ok(SafeCString {
            ptr: c_string.into_raw()
        })
    }
    
    fn as_ptr(&self) -> *const i8 {
        self.ptr
    }
    
    fn to_string(&self) -> Result<String, std::str::Utf8Error> {
        unsafe {
            let c_str = CStr::from_ptr(self.ptr)
            Ok(c_str.to_str()?.to_string())
        }
    }
}

impl Drop for SafeCString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(self.ptr)
            }
        }
    }
}

# 安全的内存管理
struct ManagedBuffer {
    ptr: *mut u8,
    size: usize,
    capacity: usize,
}

impl ManagedBuffer {
    fn new(capacity: usize) -> Self {
        unsafe {
            let ptr = malloc(capacity)
            if ptr.is_null() {
                panic!("Failed to allocate memory")
            }
            ManagedBuffer {
                ptr,
                size: 0,
                capacity,
            }
        }
    }
    
    fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.size)
        }
    }
    
    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.size)
        }
    }
    
    fn resize(&mut self, new_size: usize) -> Result<(), String> {
        if new_size > self.capacity {
            return Err("Size exceeds capacity".to_string())
        }
        self.size = new_size
        Ok(())
    }
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                free(self.ptr)
            }
        }
    }
}
```

### 错误处理

```valkyrie
# FFI 错误类型
#[derive(Debug)]
enum FFIError {
    NullPointer,
    InvalidUtf8(std::str::Utf8Error),
    LibraryLoadError(String),
    SymbolNotFound(String),
    FunctionCallFailed(i32),
    MemoryAllocationFailed,
}

impl std::fmt::Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FFIError::NullPointer => write!(f, "Null pointer encountered"),
            FFIError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
            FFIError::LibraryLoadError(msg) => write!(f, "Library load error: {}", msg),
            FFIError::SymbolNotFound(name) => write!(f, "Symbol not found: {}", name),
            FFIError::FunctionCallFailed(code) => write!(f, "Function call failed with code: {}", code),
            FFIError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
        }
    }
}

impl std::error::Error for FFIError {}

# 安全的 FFI 调用包装器
fn safe_ffi_call<F, R>(f: F) -> Result<R, FFIError>
where
    F: FnOnce() -> R,
{
    # 设置错误处理
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .map_err(|_| FFIError::FunctionCallFailed(-1))
}
```

## 最佳实践

### 1. 类型安全

```valkyrie
# 使用 newtype 模式增强类型安全
struct FileHandle(*mut c_void);
struct DatabaseConnection(*mut c_void);

# 避免混淆不同类型的指针
impl FileHandle {
    fn new(path: &str) -> Result<Self, FFIError> {
        let c_path = CString::new(path).map_err(|_| FFIError::InvalidUtf8)?;
        unsafe {
            let handle = fopen(c_path.as_ptr(), b"r\0".as_ptr() as *const i8);
            if handle.is_null() {
                Err(FFIError::NullPointer)
            } else {
                Ok(FileHandle(handle))
            }
        }
    }
}
```

### 2. 资源管理

```valkyrie
# RAII 模式确保资源释放
struct ResourceGuard<T> {
    resource: Option<T>,
    cleanup: fn(T),
}

impl<T> ResourceGuard<T> {
    fn new(resource: T, cleanup: fn(T)) -> Self {
        ResourceGuard {
            resource: Some(resource),
            cleanup,
        }
    }
    
    fn take(&mut self) -> Option<T> {
        self.resource.take()
    }
}

impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            (self.cleanup)(resource)
        }
    }
}
```

### 3. 版本兼容性

```valkyrie
# 版本检查
struct LibraryVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

fn check_library_compatibility(required: LibraryVersion, actual: LibraryVersion) -> bool {
    # 主版本必须匹配
    if required.major != actual.major {
        return false
    }
    
    # 次版本向后兼容
    if actual.minor < required.minor {
        return false
    }
    
    # 补丁版本不影响兼容性
    true
}
```

Valkyrie 的 FFI 系统提供了安全、高效的跨语言互操作能力，支持与主流编程语言和运行时环境的集成，为构建复杂的多语言应用提供了强大的基础设施。