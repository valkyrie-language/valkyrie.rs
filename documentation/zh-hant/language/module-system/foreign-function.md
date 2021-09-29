# 外部函式介面 (Foreign Function Interface)

Valkyrie 提供了強大的 FFI 系統，支援與 C、C++、Rust、Python、JavaScript 等多種語言的互操作，實現高效的跨語言呼叫和資料交換。

> **注意**：本頁主要介紹 FFI 的語法和呼叫方式。關於如何安全地管理外部物件的生命週期（如自動釋放記憶體），請參考 [外部物件生命週期管理](../lifetime/foreign-objects.md)。

## C/C++ 互操作

### 基本 C 函式呼叫

```valkyrie
# 聲明外部 C 函式
@import(c, "libc", "malloc")
micro malloc(size: usize) -> ◆u8

@import(c, "libc", "free")
micro free(ptr: ◆u8)

@import(c, "libc", "strlen")
micro strlen(s: ◇i8) -> usize

@import(c, "libc", "printf")
micro printf(format: ◇i8, ...) -> i32

@import(c, "libc", "sin")
micro sin(x: f64) -> f64

@import(c, "libc", "cos")
micro cos(x: f64) -> f64

# 使用 C 函式
micro use_c_functions() {
    unsafe {
        let ptr = malloc(1024)
        if !ptr.is_null() {
            # 使用記憶體
            free(ptr)
        }
        
        let result = sin(3.14159 / 2.0)
        print("sin(π/2) = {}", result)
    }
}
```

### C 結構體互操作

```valkyrie
# C 相容的結構體 (Value Class)
structure Point {
    x: f64
    y: f64
}

structure Rectangle {
    top_left: Point
    bottom_right: Point
}

# 聲明使用結構體的 C 函式
@import(c, "geometry_lib", "calculate_distance")
micro calculate_distance(p1: ◇Point, p2: ◇Point) -> f64

@import(c, "geometry_lib", "rectangle_area")
micro rectangle_area(rect: ◇Rectangle) -> f64

@import(c, "geometry_lib", "create_point")
micro create_point(x: f64, y: f64) -> Point

# 與 C 互動
micro geometry_calculations() {
    let p1 = Point(0.0, 0.0)
    let p2 = Point(3.0, 4.0)
    
    unsafe {
        let distance = calculate_distance(p1, p2)
        print("Distance: {}", distance)
        
        let rect = Rectangle(
            Point(0.0, 10.0),
            Point(5.0, 0.0)
        )
        let area = rectangle_area(rect)
        print("Area: {}", area)
    }
}
```

### C++ 類別互操作

```valkyrie
# C++ 類別的 C 封裝器聲明
# Vector3D 類別的 C 介面
@import(c, "vector_lib", "vector3d_new")
micro vector3d_new(x: f64, y: f64, z: f64) -> ◆u8

@import(c, "vector_lib", "vector3d_delete")
micro vector3d_delete(ptr: ◆u8)

@import(c, "vector_lib", "vector3d_magnitude")
micro vector3d_magnitude(ptr: ◇u8) -> f64

@import(c, "vector_lib", "vector3d_normalize")
micro vector3d_normalize(ptr: ◆u8)

@import(c, "vector_lib", "vector3d_dot")
micro vector3d_dot(ptr1: ◇u8, ptr2: ◇u8) -> f64

@import(c, "vector_lib", "vector3d_cross")
micro vector3d_cross(ptr1: ◇u8, ptr2: ◇u8) -> ◆u8

# Valkyrie 封裝器 (Reference Class)
class Vector3D {
    ptr: ◆u8
}

imply Vector3D {
    micro constructor(mut self, x: f64, y: f64, z: f64) {
        unsafe {
            self.ptr = vector3d_new(x, y, z)
        }
    }
    
    micro magnitude(self) -> f64 {
        unsafe { vector3d_magnitude(self.ptr) }
    }
    
    micro normalize(mut self) {
        unsafe { vector3d_normalize(self.ptr) }
    }
    
    micro dot(self, other: Vector3D) -> f64 {
        unsafe { vector3d_dot(self.ptr, other.ptr) }
    }
    
    micro cross(self, other: Vector3D) -> Vector3D {
        unsafe {
            Vector3D {
                ptr: vector3d_cross(self.ptr, other.ptr)
            }
        }
    }
}

# 階段 3: Finalize
imply Vector3D: Finalize {
    micro finalize(mut self) {
        unsafe {
            vector3d_delete(self.ptr)
        }
    }
}
```

## Rust 互操作

### 呼叫 Rust 函式庫

```valkyrie
# 連結 Rust 靜態庫
@link(name: "myrust_lib", kind: "static")
@import(rust, "myrust_lib", "rust_fibonacci")
micro rust_fibonacci(n: u32) -> u64

@import(rust, "myrust_lib", "rust_sort_array")
micro rust_sort_array(arr: ◆i32, len: usize)

@import(rust, "myrust_lib", "rust_json_parse")
micro rust_json_parse(json_str: ◇u8) -> ◆u8

@import(rust, "myrust_lib", "rust_json_free")
micro rust_json_free(ptr: ◆u8)

# 使用 Rust 函式
class RustJson {
    ptr: ◆u8
}

imply RustJson {
    micro new(json_str: utf8) -> Self {
        unsafe {
            RustJson {
                ptr: rust_json_parse(json_str.as_ptr())
            }
        }
    }
}

# 階段 3: Finalize
imply RustJson: Finalize {
    micro finalize(mut self) {
        unsafe {
            rust_json_free(self.ptr)
        }
    }
}

micro use_rust_library() {
    unsafe {
        let fib_10 = rust_fibonacci(10)
        print("Fibonacci(10) = {}", fib_10)
        
        let mut numbers = [5, 2, 8, 1, 9, 3]
        rust_sort_array(numbers.as_mut_ptr(), numbers.length)
        print("Sorted: {}", numbers)
    }
}
```

### 匯出函式給其他語言

```valkyrie
# 匯出 Valkyrie 函式給 C/C++
@export(c, "valkyrie_add")
micro valkyrie_add(a: i32, b: i32) -> i32 {
    a + b
}

@export(c, "valkyrie_create_string")
micro valkyrie_create_string(s: ◇u8) -> ◆u8 {
    if s.is_null() { return std::ptr::null_mut() }
    
    unsafe {
        let c_str = CStr::from_ptr(s)
        let rust_str = c_str.to_str().default("")
        let processed = "Processed: {}".format(rust_str)
        
        let c_string = CString::new(processed).expect("Invalid string")
        c_string.into_raw()
    }
}

@export(c, "valkyrie_free_string")
micro valkyrie_free_string(s: ◆u8) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s)
        }
    }
}
```

## Python 互操作

Valkyrie 提供了與 Python 的原生互操作支援，可以直接匯入 Python 模組並呼叫其中的函式。

```valkyrie
using std::python

class PythonInterpreter {
    handle: python::Handle
}

imply PythonInterpreter {
    micro new() -> Self {
        Self { handle: python::init() }
    }
    
    micro run_script(self, script: utf8) {
        self.handle.execute(script)
    }
}

# 階段 3: Finalize
imply PythonInterpreter: Finalize {
    micro finalize(mut self) {
        python::finalize(self.handle)
    }
}

micro main() {
    let py = PythonInterpreter::new()
    
    # 匯入 Python 模組
    let math = python::import("math")
    let result = math.sin(3.14159 / 2.0)
    print("Python sin(π/2) = {}", result)
    
    # 執行複雜的 Python 程式碼
    py.run_script(r"
import matplotlib.pyplot as plt
import numpy as np

x = np.linspace(0, 10, 100)
y = np.sin(x)
plt.plot(x, y)
plt.show()
    ")
}
```

### Python 擴充模組

```valkyrie
# 建立 Python 擴充模組
using pyo3::prelude

@pyfunction
micro fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}

@pyfunction
micro matrix_multiply(a: [[f64]], b: [[f64]]) -> Result⟨[[f64]], PyError⟩ {
    let rows_a = a.length
    let cols_a = a[0].length
    let cols_b = b[0].length
    
    if cols_a != b.length {
        return Err(PyValueError::new_err("Incompatible matrix dimensions"))
    }
    
    let mut result: [[f64]] = []
    
    loop i in 0..rows_a {
        loop j in 0..cols_b {
            loop k in 0..cols_a {
                result[i][j] += a[i][k] * b[k][j]
            }
        }
    }
    
    Ok(result)
}

@pyclass
class Calculator {
    @pyo3(get, set)
    value: f64,
}

imply Calculator {
    @new
    micro new(initial_value: f64) -> Self {
        Calculator { value: initial_value }
    }
    
    micro add(mut self, other: f64) -> f64 {
        self.value += other
        self.value
    }
    
    micro multiply(mut self, other: f64) -> f64 {
        self.value ×= other
        self.value
    }
    
    micro reset(mut self) {
        self.value = 0.0
    }
}

@pymodule
micro valkyrie_math(_py: Python, m: PyModule) -> Result⟨(), PyError⟩ {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    m.add_function(wrap_pyfunction!(matrix_multiply, m)?)?;
    m.add_class::<Calculator>()?;
    Ok(())
}
```

## JavaScript 互操作

### WebAssembly 匯出

```valkyrie
# 編譯到 WebAssembly
using wasm_bindgen.prelude

@wasm_bindgen
# 匯入 JavaScript 函式
@import(js, "console", "log")
micro log(s: utf8)

@import(js, "Math", "random")
micro random() -> f64

@import(js, "window", "alert")
micro alert(s: utf8)

# 使用宏簡化日誌
macro console_log(args) {
    log("{}".format(args))
}

@wasm_bindgen
class GameEngine {
    width: u32
    height: u32
    entities: [Entity]
}

@wasm_bindgen
imply GameEngine {
    @wasm_bindgen(constructor)
    micro new(width: u32, height: u32) -> Self {
        console_log("Creating game engine {}x{}".format(width, height))
        GameEngine {
            width,
            height,
            entities: [],
        }
    }
    
    @wasm_bindgen
    micro add_entity(mut self, x: f64, y: f64) -> usize {
        let entity = Entity { x, y, vx: 0.0, vy: 0.0 }
        self.entities.push(entity)
        self.entities.length - 1
    }
    
    @wasm_bindgen
    micro update(mut self, dt: f64) {
        loop entity in mut self.entities {
            entity.x += entity.vx * dt
            entity.y += entity.vy * dt
            
            # 邊界檢查
            if entity.x < 0.0 || entity.x > self.width {
                entity.vx = -entity.vx
            }
            if entity.y < 0.0 || entity.y > self.height {
                entity.vy = -entity.vy
            }
        }
    }
    
    @wasm_bindgen
    micro get_entity_positions(self) -> [f64] {
        let mut positions = []
        loop entity in self.entities {
            positions.push(entity.x)
            positions.push(entity.y)
        }
        positions
    }
}

class Entity {
    x: f64
    y: f64
    vx: f64
    vy: f64
}

# 匯出數學函式
@wasm_bindgen
micro fast_inverse_sqrt(x: f32) -> f32 {
    # Quake III 快速平方根倒數演算法
    let i = x.to_bits()
    let i = 0x5f3759df - (i >> 1)
    let y = f32::from_bits(i)
    y * (1.5 - 0.5 * x * y ^ 2)
}

# Mandelbrot 集合計算
@wasm_bindgen
micro mandelbrot(cx: f64, cy: f64, max_iter: u32) -> u32 {
    let mut x = 0.0
    let mut y = 0.0
    let mut iter = 0
    
    while x ^ 2 + y ^ 2 <= 4.0 && iter < max_iter {
        let temp = x ^ 2 - y ^ 2 + cx
        y = 2.0 * x * y + cy
        x = temp
        iter += 1
    }
    
    iter
}
```

### Node.js 原生模組

```valkyrie
# Node.js N-API 綁定
using napi.bindgen_prelude

@napi
micro fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}

@napi
micro process_image(data: Buffer, width: u32, height: u32) -> Result⟨Buffer, Error⟩ {
    let mut pixels = data.to_vector()
    
    # 簡單的影像處理：反轉顏色
    loop i in (0..pixels.length).step_by(4) {
        pixels[i] = 255 - pixels[i]      # R
        pixels[i + 1] = 255 - pixels[i + 1]  # G
        pixels[i + 2] = 255 - pixels[i + 2]  # B
        # Alpha 色版保持不變
    }
    
    Ok(Buffer::from(pixels))
}

@napi
class FileProcessor {
    buffer_size: u32
}

@napi
imply FileProcessor {
    @napi(constructor)
    micro new(buffer_size: u32) -> Self {
        FileProcessor { buffer_size }
    }
    
    @napi
    micro process_file(self, path: utf8) -> Result⟨utf8, Error⟩ {
        # 檔案處理邏輯
        let content = std::fs::read_to_string(path)
            .map_err { Error::new(Status::GenericFailure, "Failed to read file: {}".format($)) }?
        
        # 簡單處理：統計行數和字元數
        let lines = content.lines().count()
        let chars = content.chars().count()
        
        Ok("File: {}, Lines: {}, Characters: {}".format(path, lines, chars))
    }
    
    @napi
    async micro process_file_async(self, path: utf8) -> Result⟨utf8, Error⟩ {
        # 非同步檔案處理
        let content = std::fs::read_to_string(path).await
            .map_err { Error::new(Status::GenericFailure, "Failed to read file: {}".format($)) }?
        
        # 模擬耗時處理
        std::time::sleep(Duration::from_millis(100)).await
        
        let processed = content.to_uppercase()
        Ok(processed)
    }
}
```

## 動態函式庫載入

### 執行期動態載入

```valkyrie
using libloading.{Library, Symbol}

# 動態函式庫管理器
class DynamicLibrary {
    lib: Library
}

imply DynamicLibrary {
    micro load(path: utf8) -> Result⟨Self, Error⟩ {
        unsafe {
            let lib = Library::new(path)?
            Ok(DynamicLibrary { lib })
        }
    }
    
    micro get_function⟨T⟩(self, name: [u8]) -> Result⟨Symbol⟨T⟩, Error⟩ {
        unsafe {
            let symbol = self.lib.get(name)?
            Ok(symbol)
        }
    }
}

# 使用動態函式庫
micro use_dynamic_library() -> Result⟨(), Error⟩ {
    let lib = DynamicLibrary::load("./libmath.so")?
    
    # 獲取函式指標
    let add_func: Symbol⟨unsafe micro(i32, i32) -> i32⟩ = lib.get_function(b"add")?
    let multiply_func: Symbol⟨unsafe micro(f64, f64) -> f64⟩ = lib.get_function(b"multiply")?
    
    # 呼叫動態載入的函式
    unsafe {
        let sum = add_func(5, 3)
        let product = multiply_func(2.5, 4.0)
        
        print("5 + 3 = {}", sum)
        print("2.5 * 4.0 = {}", product)
    }
    
    Ok(())
}
```

### 外掛系統

```valkyrie
# 外掛介面定義
trait Plugin {
    micro name(self) -> utf8
    micro version(self) -> utf8
    micro initialize(mut self) -> Result⟨(), utf8⟩
    micro execute(self, input: utf8) -> Result⟨utf8, utf8⟩
    micro cleanup(mut self)
}

# 外掛管理器
class PluginManager {
    plugins: HashMap⟨utf8, Plugin⟩
    libraries: [Library]
}

imply PluginManager {
    micro new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
            libraries: [],
        }
    }
    
    micro load_plugin(mut self, path: utf8) -> Result⟨(), Error⟩ {
        unsafe {
            let lib = Library::new(path)?
            
            # 獲取外掛建立函式
            let create_plugin: Symbol⟨unsafe micro() -> ◆Plugin⟩ = 
                lib.get(b"create_plugin")?
            
            let plugin_ptr = create_plugin()
            let plugin = Box::from_raw(plugin_ptr)
            
            let name = plugin.name().to_utf8()
            self.plugins.insert(name, plugin)
            self.libraries.push(lib)
            
            Ok(())
        }
    }
    
    micro execute_plugin(self, name: utf8, input: utf8) -> Result⟨utf8, utf8⟩ {
        match self.plugins.get(name) {
            Some(plugin) => plugin.execute(input),
            None => Fail("Plugin '{}' not found".format(name))
        }
    }
    
    micro list_plugins(self) -> [(utf8, utf8)] {
        self.plugins.iter()
            .map(|(name, plugin)| (name, plugin.version()))
            .collect()
    }
}

# 外掛匯出宏
macro export_plugin(plugin_type) {
    @export(c, "create_plugin")
    micro create_plugin() -> ◆Plugin {
        let plugin = Box::new(plugin_type::new())
        Box::into_raw(plugin)
    }
    
    @export(c, "destroy_plugin")
    micro destroy_plugin(plugin: ◆Plugin) {
        if !plugin.is_null() {
            unsafe {
                let _ = Box::from_raw(plugin)
            }
        }
    }
}
```

## 記憶體管理和安全性

### 安全的 FFI 封裝器

```valkyrie
# 安全的 C 字串處理
class SafeCString {
    ptr: ◆i8
}

imply SafeCString {
    micro new(s: utf8) -> Result⟨Self, Error⟩ {
        let c_string = CString::new(s)?
        Ok(SafeCString {
            ptr: c_string.into_raw()
        })
    }
    
    micro as_ptr(self) -> ◇i8 {
        self.ptr
    }
    
    micro to_utf8(self) -> Result⟨utf8, Error⟩ {
        unsafe {
            let c_str = CStr::from_ptr(self.ptr)
            Ok(c_str.to_str()?.to_utf8())
        }
    }
}

# 階段 3: Finalize
imply SafeCString: Finalize {
    micro finalize(mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(self.ptr)
            }
        }
    }
}

# 安全的記憶體管理
class ManagedBuffer {
    ptr: ◆u8
    size: usize
    capacity: usize
}

imply ManagedBuffer {
    micro new(capacity: usize) -> Self {
        unsafe {
            let ptr = malloc(capacity)
            if ptr.is_null() {
        return Err(FFIError::NullPointer)
    }
            ManagedBuffer {
                ptr,
                size: 0,
                capacity,
            }
        }
    }
    
    micro as_slice(self) -> [u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.size)
        }
    }
    
    micro as_mut_slice(mut self) -> [mut u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.size)
        }
    }
    
    micro resize(mut self, new_size: usize) -> Result⟨(), utf8⟩ {
        if new_size > self.capacity {
            return Fail("Size exceeds capacity")
        }
        self.size = new_size
        Ok(())
    }
}

# 階段 3: Finalize
imply ManagedBuffer: Finalize {
    micro finalize(mut self) {
        if !self.ptr.is_null() {
            unsafe {
                free(self.ptr)
            }
        }
    }
}
```

### 錯誤處理

```valkyrie
# FFI 錯誤型別
@derive(Debug)
enums FFIError {
    NullPointer
    InvalidUtf8(Error)
    LibraryLoadError(utf8)
    SymbolNotFound(utf8)
    FunctionCallFailed(i32)
    MemoryAllocationFailed
}

imply FFIError: Display {
    micro fmt(self, f: Formatter) -> Result {
        match self {
            FFIError::NullPointer => f.write("Null pointer encountered"),
            FFIError::InvalidUtf8(e) => f.write("Invalid UTF-8: {}".format(e)),
            FFIError::LibraryLoadError(msg) => f.write("Library load error: {}".format(msg)),
            FFIError::SymbolNotFound(name) => f.write("Symbol not found: {}".format(name)),
            FFIError::FunctionCallFailed(code) => f.write("Function call failed with code: {}".format(code)),
            FFIError::MemoryAllocationFailed => f.write("Memory allocation failed"),
        }
    }
}

imply FFIError: Error {}

# 安全的 FFI 呼叫封裝器
micro safe_ffi_call⟨F, R⟩(f: F) -> Result⟨R, FFIError⟩
where
    F: micro() -> R,
{
    # 設置錯誤處理
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .map_err { FFIError::FunctionCallFailed(-1) }
}
```

## 最佳實踐

### 1. 型別安全

```valkyrie
# 控制代碼模式 (Handle Pattern)
class FileHandle(◆c_void)
class DatabaseConnection(◆c_void)

imply FileHandle {
    micro open(path: utf8) -> Option⟨FileHandle⟩ {
        let c_path = path.to_c_utf8()
        unsafe {
            let handle = fopen(c_path.as_ptr(), "r".as_ptr())
            if handle.is_null() { None } else { Some(FileHandle(handle)) }
        }
    }
}
```

### 2. RAII 模式 (Resource Acquisition Is Initialization)

透過終結器 (Finalizer) 確保資源在超出作用域時自動釋放。

```valkyrie
class ResourceGuard⟨T⟩ {
    resource: Option⟨T⟩
    cleanup: micro(T)
}

imply ResourceGuard⟨T⟩ {
    micro new(resource: T, cleanup: micro(T)) -> Self {
        ResourceGuard {
            resource: Some(resource),
            cleanup,
        }
    }
    
    micro take(mut self) -> Option⟨T⟩ {
        self.resource.take()
    }
}

# 階段 3: Finalize
imply ResourceGuard⟨T⟩: Finalize {
    micro finalize(mut self) {
        if let item = self.resource.take()? {
            (self.cleanup)(item)
        }
    }
}
```

### 3. 版本相容性

```valkyrie
# 版本檢查
class LibraryVersion {
    major: u32
    minor: u32
    patch: u32
}

micro check_library_compatibility(required: LibraryVersion, actual: LibraryVersion) -> bool {
    # 主版本必須相符
    if required.major != actual.major {
        return false
    }
    
    # 次版本向後相容
    if actual.minor < required.minor {
        return false
    }
    
    # 補丁版本不影響相容性
    true
}
```

Valkyrie 的 FFI 系統提供了安全、高效的跨語言互操作能力，支援與主流程式語言和執行期環境的整合，為建構複雜的多語言應用提供了強大的基礎設施。
