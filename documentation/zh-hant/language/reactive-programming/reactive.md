# 響應式程式設計

響應式程式設計是一種基於資料流和變化傳播的程式設計範式。在 Valkyrie 中，響應式程式設計透過 Observable、Signal 和 Reactive 等抽象提供了強大的資料流處理能力。

## 核心原語：狀態 (Signal) vs 事件 (Observable)

在 Valkyrie 中，響應式程式設計被劃分為兩個並行的核心概念，分別應對不同的場景：

### 1. 信號 (Signal) —— 狀態管理
Signal 代表**當前狀態**。它始終持有一個值，且任何對該值的讀取都會自動建立相依關係。

- **抽象**：透過 `Accessor` (讀) 和 `Settler` (寫) 進行角色化抽象。
- **特性**：同步更新、細粒度追蹤、資料驅動。
- **適用**：UI 資料繫結、配置項、業務狀態。

### 2. 觀察物件 (Observable) —— 事件處理
Observable 代表**事件序列**。它描述了隨時間推移發生的一系列動作。

- **抽象**：透過 `Observable` (源) 和 `Observer` (接收) 進行流式抽象。
- **特性**：非同步傳播、惰性執行（Lazy）、管道運算子豐富。
- **適用**：使用者互動（點擊/捲動）、網路請求流、計時器、WebSocket 資料流。

---

## 核心概念詳解

### Signal (狀態容器)

```valkyrie
# 基礎用法：使用 raise 掛鉤狀態
let count = raise Signal(0)

# 衍生狀態 (Accessor)
let doubled = Memo { count * 2 }

# 副作用
Effect {
    # 編譯器自動追蹤 count 和 doubled，無需 .value
    print("當前值: {count}, 二倍值: {doubled}")
}

count = 5 # 觸發同步更新
```

### Observable (事件流)

```valkyrie
# 從事件源建立
let clicks = Observable.from_events("click")

# 鏈式運算子處理
let debounced_clicks = clicks
    .debounce(300ms)
    .map { "Clicked at: {now()}" }

# 訂閱執行
debounced_clicks.subscribe {
    print($)
}
```

## 基本運算子

### 轉換運算子

```valkyrie
# map - 轉換每個值
let numbers = Observable.from([1, 2, 3, 4, 5])
let squares = numbers.map { $ * $ }

# flat_map - 扁平化映射
let words = Observable.from(["hello", "world"])
let characters = words.flat_map {
    Observable.from($.chars())
}

# scan - 累積操作
let numbers = Observable.from([1, 2, 3, 4, 5])
let running_sum = numbers.scan(0) { $1 + $2 }
# 輸出: 1, 3, 6, 10, 15
```

### 過濾運算子

```valkyrie
# filter - 過濾值
let numbers = Observable.from([1, 2, 3, 4, 5, 6])
let evens = numbers.filter { $ % 2 == 0 }

# take - 取前 N 個值
let first_three = numbers.take(3)

# skip - 跳過前 N 個值
let after_two = numbers.skip(2)

# distinct - 去重
let unique = Observable.from([1, 1, 2, 2, 3, 3]).distinct()
```

### 組合運算子

```valkyrie
# merge - 合併多個流
let stream1 = Observable.from([1, 3, 5])
let stream2 = Observable.from([2, 4, 6])
let merged = stream1.merge(stream2)

# zip - 配對組合
let names = Observable.from(["Alice", "Bob", "Charlie"])
let ages = Observable.from([25, 30, 35])
let people = names.zip(ages).map {
    Person { name: $1, age: $2 }
}

# combine_latest - 最新值組合
let temperature = Signal(20.0)
let humidity = Signal(60.0)
let comfort_index = temperature.combine_latest(humidity).map {
    calculate_comfort($1, $2)
}
```

## 實際應用示例

### 使用者介面響應式更新

```valkyrie
# 響應式 UI 元件
class CounterComponent {
    private count: Signal⟨i32⟩
    private increment_clicks: Observable⟨unit⟩
    private decrement_clicks: Observable⟨unit⟩
    
    micro CounterComponent() -> CounterComponent {
        let count = Signal(0)
        let increment_clicks = Observable.from_events("increment")
        let decrement_clicks = Observable.from_events("decrement")
        
        # 響應點擊事件
        increment_clicks.subscribe {
            count.update { $ + 1 }
        }
        
        decrement_clicks.subscribe {
            count.update { $ - 1 }
        }
        
        CounterComponent {
            count,
            increment_clicks,
            decrement_clicks
        }
    }
    
    micro render(self) -> Widget {
        let count_text = self.count.map { value -> "計數: {value}" }
        
        VStack {
            Text(count_text)
            HStack {
                Button("增加").on_click(self.increment_clicks)
                Button("減少").on_click(self.decrement_clicks)
            }
        }
    }
}
```

### 資料流處理

```valkyrie
# 即時資料處理管道
class DataProcessor {
    micro process_sensor_data(sensor_stream: Observable⟨SensorReading⟩) -> Observable⟨ProcessedData⟩ {
        sensor_stream
            .filter { $.is_valid() }  # 過濾無效資料
            .map { $.normalize() }    # 標準化資料
            .buffer(5s)               # 5秒緩衝視窗
            .map { self.analyze_batch($) } # 批量分析
            .filter { $.confidence > 0.8 } # 過濾低信賴度結果
    }
    
    private micro analyze_batch(batch: [SensorReading]) -> ProcessedData {
        let average = batch.iter().map { $.value }.sum() / batch.length
        let variance = calculate_variance(batch)
        
        ProcessedData {
            timestamp: now(),
            average,
            variance,
            confidence: calculate_confidence(variance)
        }
    }
}

# 使用資料處理器
let processor = DataProcessor()
let sensor_stream = Observable.from_websocket("ws://sensor.example.com")
let processed_stream = processor.process_sensor_data(sensor_stream)
```
