# Promise (承諾)

`Promise` 是 `Future` 的標準實作。它不僅代表一個未來的值，還提供了手動控制該值何時完成的能力。

## 基本用法

通常，你可以透過 `async` 區塊自動建立一個 `Promise`：

```valkyrie
let p: Promise⟨i32⟩ = async {
    42
}
```

### 顯式建立與完成

在某些低階場景或與外部程式碼互動時，你可能需要手動控制 `Promise`：

```valkyrie
# 建立一個處於掛起狀態的 Promise 和它的解析器
let (p, resolver) = Promise.pending⟨string⟩()

# 在稍後的某個時刻手動完成它
resolver.resolve("Success!")

# 或者讓它失敗
# resolver.reject(Error("Failed"))
```

## 執行控制

`Promise` 提供了三種主要的執行模式，透過 `.run` 控制器（通常可省略）存取：

### 1. 非同步等待 (.await)
在非同步函式中掛起，不阻塞執行緒。
```valkyrie
let data = fetch_data().await
```

### 2. 同步阻塞 (.block)
在同步環境中阻塞當前執行緒，直到結果返回。
```valkyrie
let data = fetch_data().block
```

### 3. 非同步啟動 (.awake)
啟動任務但不等待其結果（Fire and Forget）。
```valkyrie
fetch_data().awake
```

## 靜態方法

- `Promise.resolve(val)`: 建立一個已經成功的 Promise。
- `Promise.reject(err)`: 建立一個已經失敗的 Promise。
- `Promise.all([p1, p2])`: 等待所有 Promise 完成，如果任一失敗則整體失敗。
- `Promise.any([p1, p2])`: 只要有一個 Promise 成功就返回其結果。
- `Promise.allSettled([p1, p2])`: 等待所有 Promise 完成（無論成功或失敗）。

## 與 JavaScript Promise 的關係

Valkyrie 的 `Promise` 在編譯到 JavaScript 後會直接映射為原生的 `Promise` 物件，確保了零開銷的互通性。

---
**相關章節**:
- [Future](./future.md) - 非同步底層原語
- [執行控制](./index.md#執行控制runawait--runblock--runawake--awake) - 詳細的執行模式說明
