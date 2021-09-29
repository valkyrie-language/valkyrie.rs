# Valkyrie 範例集合

本目錄包含了 Valkyrie 語言在各個領域的完整範例和教程，展示了 Valkyrie 的強大功能和廣泛應用場景。

## 🎮 遊戲開發

[遊戲開發框架](game-development/) - 完整的遊戲開發解決方案

- **核心功能**：遊戲引擎架構、ECS 系統、渲染管線
- **圖形編程**：Shader 開發、GPU 計算、wgpu 集成
- **性能優化**：內存管理、並發處理、資源管理
- **實用工具**：場景管理、資源加載、調試工具

## 🔧 嵌入式開發

[嵌入式開發](embedded-development/) - 嵌入式系統和 WebAssembly 開發

- **WASM 開發**：WebAssembly 模組、WASI 接口、內存管理
- **微控制器編程**：GPIO 控制、中斷處理、通信協議
- **實時系統**：RTOS 開發、任務調度、時序控制
- **傳感器接口**：ADC 採集、I2C/SPI 通信、數據處理
- **電源管理**：低功耗設計、睡眠模式、喚醒機制

## 🔬 芯片設計

[芯片設計](chip-design/) - 硬件描述語言和數字電路設計

- **HDL 基礎**：硬件數據類型、模組定義、時序邏輯
- **數字電路**：組合邏輯、ALU 設計、狀態機
- **處理器設計**：RISC-V 核心、指令解碼、流水線
- **內存系統**：RAM 設計、緩存結構、存儲控制器
- **總線互連**：AXI4 協議、交叉開關、仲裁器
- **驗證方法**：測試平台、形式化驗證、覆蓋率分析
- **綜合實現**：FPGA 開發、ASIC 設計流程、時序約束

## 🌐 網頁開發

[網頁開發](web-development/) - 現代化的 Web 開發框架

- **Web 伺服器**：高性能 HTTP 伺服器、路由系統、中間件
- **界面組件**：基於 Widget 的響應式 UI 開發
- **XML 語法**：類 TSX 的聲明式語法 (X-Grammar)
- **原生語法**：基於 DSL 的函數式語法 (V-Grammar)
- **事件處理**：單點事件與廣播事件機制

## 🌟 特色功能

### 類型安全

Valkyrie 在所有領域都提供編譯時類型檢查，確保代碼的正確性和安全性：

```valkyrie
# 遊戲開發中的類型安全
class Player {
    position: Vec3⟨f32⟩,
    health: Health⟨100⟩,  # 編譯時範圍檢查
    inventory: Inventory⟨32⟩  # 固定大小容器
}

# 嵌入式開發中的硬件抽象
class GpioPin⟨PIN: u8, PORT: char⟩ {
    _phantom: PhantomData⟨(PIN, PORT)⟩
}

# 芯片設計中的位寬檢查
type UInt⟨W: usize⟩ = HardwareType⟨u64, W⟩
let result: UInt⟨33⟩ = add(a: UInt⟨32⟩, b: UInt⟨32⟩)  # 自動推導位寬
```

### 零成本抽象

高級抽象在編譯時完全優化，運行時性能等同於手寫的底層代碼：

```valkyrie
# 高級遊戲邏輯
entities.query⟨(Transform, Velocity)⟩()
    .par_iter()
    .for_each {
        $1.position += $2.delta * dt
    }

# 編譯後等效於優化的循環
# 無運行時開銷，無動態分配
```

### 內存安全

在系統編程中提供內存安全保證，避免常見的內存錯誤：

```valkyrie
# 嵌入式開發中的安全內存操作
micro process_buffer(mut buffer: [u8]) {
    # 編譯時邊界檢查
    loop i in 0..buffer.length {
        buffer[i] = buffer[i].wrapping_add(1)  # 明確的溢出行為
    }
}

# 芯片設計中的安全硬件訪問
micro write_register<const ADDR: u32>(value: u32) {
    # 編譯時地址驗證
    unsafe { *(ADDR as ◆u32) = value }
}
```

### 並發編程

內置的並發原語支持安全的多線程和異步編程：

```valkyrie
# 遊戲中的並行系統
async micro update_physics(world: &World) {
    let (positions, velocities) = world.query_mut::<(Position, Velocity)>()
    
    positions.par_iter_mut()
        .zip(velocities.par_iter())
        .for_each {
            $1.update(*$2)
        }
}

# 嵌入式中的異步 I/O
async micro read_sensor() -> SensorData {
    let data = i2c.read_async(SENSOR_ADDR).await?
    SensorData::parse(data)
}
```

## 🚀 開始使用

1. **選擇領域**：根據你的項目需求選擇相應的範例目錄
2. **閱讀文檔**：每個目錄都包含詳細的說明和教程
3. **運行範例**：所有範例都可以直接編譯和運行
4. **深入學習**：通過修改範例代碼來學習 Valkyrie 的特性

## 📚 相關資源

- [Valkyrie 語言參考](../language/) - 完整的語言規範和特性說明
- [標準庫文檔](../stdlib/) - 標準庫 API 參考
- [工具鏈指南](../toolchain/) - 編譯器、調試器等工具使用說明
- [最佳實踐](../best-practices/) - 代碼風格和設計模式

## 🤝 貢獻指南

歡迎為 Valkyrie 範例集合貢獻代碼和文檔：

1. **報告問題**：發現 bug 或有改進建議請提交 issue
2. **提交代碼**：遵循項目的代碼規範和測試要求
3. **完善文檔**：幫助改進範例的說明和教程
4. **分享經驗**：分享你在使用 Valkyrie 過程中的心得體會

Valkyrie 致力於為現代系統編程提供安全、高效、易用的解決方案。通過這些範例，你可以快速掌握 Valkyrie 在各個領域的應用，並開始構建自己的項目。
