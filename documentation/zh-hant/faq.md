# 常見問題 (FAQ)

本頁面收集了 Valkyrie 開發過程中的常見問題和解答。

## 語言基礎

### Q: Valkyrie 與其他函數式語言有什麼區別？

A: Valkyrie 的獨特之處在於：
- **代數效應系統**：原生支援代數效應，優雅處理副作用
- **多目標編譯**：可編譯到 WebAssembly、JavaScript 和原生程式碼
- **現代語法**：結合函數式特性與現代語法設計
- **漸進式採用**：可與現有 JavaScript/TypeScript 專案無縫整合
- **強型別推導**：先進的型別系統，減少顯式型別註解

### Q: 什麼是代數效應？為什麼重要？

A: 代數效應是一種處理副作用的抽象機制：
- **統一抽象**：將異常、非同步、狀態管理等副作用統一處理
- **可組合性**：效應可以自由組合和嵌套
- **控制反轉**：呼叫者決定如何處理效應，而不是被呼叫者
- **型別安全**：效應在型別系統中得到體現

```valkyrie
class State<T> {
    get(): T
    set(value: T): void
}

micro counter() -> i32 {
    let current = @State::get()
    @State::set(current + 1)
    current + 1
}
```

### Q: Valkyrie 支援哪些資料型別？

A: Valkyrie 支援豐富的型別系統：
- **基礎型別**：i32, f32, utf8, bool, void
- **容器型別**：List⟨T⟩, Array⟨T⟩, Map⟨K, V⟩, Set⟨T⟩
- **可選型別**：Option⟨T⟩ (Some { value: T } | None)
- **結果型別**：Result⟨T, E⟩ (Fine { value: T } | Fail { error: E })
- **函數型別**：(A, B) -> C
- **代數資料型別**：自定義的 sum 與 product 型別
- **效應型別**：帶有效應標註的函數型別

## 語法與特性

### Q: 如何定義與使用代數資料型別？

A: 使用顯式 `tag` 的 `unite` 定義，標準寫法是 `[tag(XXXKind)] unite XXX { }`：

```valkyrie
// Sum 型別（顯式 tag 的封閉變體族）
[tag(ResultKind)]
unite Result⟨T, E⟩ {
    Fine { value: T };
    Fail { error: E };
}

// Product 型別（結構體）
struct User {
    id: i32;
    name: utf8;
    email: utf8?;
}

// 模式匹配
match result {
    case Fine { value }: print("成功: {value}");
    case Fail { error }: print("錯誤: {error}");
};
```

### Q: 如何處理非同步操作？

A: Valkyrie 支援原生的 `async/await` 語法：

```valkyrie
micro fetch_user_data(id: i32) -> User {
    let response = fetch("/api/users/{id}").await;
    parse_json(response);
}

// 頂層也可以使用 await
let user = fetch_user_data(42).await;
print("User: {user.name}");
```

### Q: 如何進行錯誤處理？

A: Valkyrie 提供多種錯誤處理方式：

```valkyrie
// 1. 使用 Result 型別
micro divide(a: f64, b: f64) -> Result⟨f64, utf8⟩ {
    if b == 0.0 {
        Fail { error: "除零錯誤" };
    } else {
        Fine { value: a / b };
    };
}

// 2. 使用異常效應
class Exception {
    throw(message: utf8): Never
}

micro safe_divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        Exception::throw("除零錯誤");
    } else {
        a / b
    }
}
```

## 編譯與部署

### Q: Valkyrie 如何編譯到不同目標？

A: Valkyrie 支援多目標編譯：

```bash
# 編譯到 WebAssembly
legion build --target wasm

# 編譯到 JavaScript
legion build --target js

# 編譯到原生程式碼
legion build --target native

# 編譯到 TypeScript 定義
legion build --target ts-defs
```

### Q: 如何與現有 JavaScript 專案整合？

A: Valkyrie 提供無縫整合：

```valkyrie
// 匯入外部模組
using hxo::std::fetch;
using hxo::std::console;

// 公有函數定義
@export(js)
micro greet(name: utf8) -> utf8 {
    "Hello, {name}!"
}

// 使用 JavaScript 物件
micro process_data(data: JSObject) -> JSObject {
    // 處理邏輯
    data
}
```

### Q: 效能如何？有哪些最佳化？

A: Valkyrie 提供多種效能最佳化：
- **尾呼叫最佳化**：自動最佳化尾遞迴
- **內聯最佳化**：小函數自動內聯
- **死程式碼消除**：移除未使用的程式碼
- **效應最佳化**：編譯時最佳化效應處理
- **記憶體管理**：智慧的垃圾回收與記憶體複用

## 工具與生態

### Q: 有哪些開發工具支援？

A: Valkyrie 提供完整的工具鏈：
- **編譯器**：`legion` CLI 工具
- **套件管理器**：內建的依賴管理
- **格式化工具**：`legion fmt` 程式碼格式化
- **語言伺服器**：VS Code、Vim 等編輯器支援
- **除錯器**：原始碼級除錯支援
- **測試框架**：內建單元測試與整合測試

### Q: 設定檔格式是什麼？

A: Valkyrie 使用 `voc.config.von` 作為設定檔：

```von
name: "my-project"
version: "0.1.0"
dependencies: {
    "std": "0.1.0"
}
```

### Q: 如何管理多套件工作區？

A: 使用 `legions.von` 管理工作區：

```von
workspace: {
    members: [
        "packages/*"
    ]
}
```

### Q: 如何編寫和執行測試？

A: 使用內建測試框架：

```valkyrie
// 單元測試
#test
micro test_addition() {
    @assert_eq(add(2, 3), 5)
    @assert_eq(add(-1, 1), 0)
}

// 屬性測試
#test
micro test_addition_commutative() {
    forall (a: i32, b: i32) {
        @assert_eq(add(a, b), add(b, a))
    }
}

// 效應測試
#test
micro test_state_effect() {
    let result = try {
        counter()
    } catch State::get || {
        resume 0;
    } catch State::set |value| {
        resume ();
    }
    @assert_eq(result, 1);
}
```

### Q: 如何管理專案依賴？

A: 使用 `voc.config.von` 設定檔：

```von
{
    name: "my-project",
    version: "0.1.0",
    authors: ["Your Name <your.email@example.com>"],
    dependencies: {
        std: "1.0",
        http: "0.3",
        json: "0.2"
    },
    build: {
        targets: ["js", "wasm"],
        optimization: "release"
    }
}
```

## 學習和社群

### Q: 如何學習 Valkyrie？

A: 推薦的學習路徑：
1. **基礎語法**：從函數式程式設計概念開始
2. **型別系統**：理解代數資料型別和模式匹配
3. **代數效應**：掌握效應的定義和處理
4. **實踐專案**：構建小型應用程式
5. **進階特性**：學習效能最佳化和工具使用

### Q: 有哪些學習資源？

A: 可用的學習資源：
- **官方教程**：[快速開始指南](/guide/getting-started)
- **範例專案**：[程式碼範例](/examples/)
- **API 文件**：完整的標準庫文件
- **社群論壇**：GitHub Discussions
- **影片教程**：YouTube 頻道

### Q: 如何貢獻到 Valkyrie 專案？

A: 貢獻方式：
1. **報告問題**：提交 bug 報告和功能請求
2. **改進文件**：完善文件和範例
3. **編寫程式碼**：實現新功能或修復問題
4. **測試回饋**：使用預發布版本並提供回饋
5. **社群支援**：幫助其他使用者解決問題

### Q: Valkyrie 的發展路線圖是什麼？

A: 主要發展方向：
- **語言特性**：模組系統、巨集系統、並行原語
- **工具改進**：更好的錯誤訊息、除錯體驗、IDE 支援
- **效能最佳化**：編譯速度、執行時效能、記憶體使用
- **生態建設**：標準庫擴展、第三方套件、框架支援
- **平台支援**：更多編譯目標、行動平台、嵌入式系統

---

如果您的問題沒有在這裡找到答案，請：
- 查看 [官方文件](/guide/)
- 提交 [GitHub Issue](https://github.com/valkyrie-lang/valkyrie/issues)
- 參與 [社群討論](https://github.com/valkyrie-lang/valkyrie/discussions)
- 加入 [Discord 社群](https://discord.gg/valkyrie-lang)
