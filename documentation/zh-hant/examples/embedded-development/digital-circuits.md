# 數字電路與硬件描述 (HDL)

Valkyrie 不仅可以编寫传统的軟件，还可以通過特定的後端和庫，作為**硬件描述語言 (HDL)** 使用，類似于 Chisel 或 SpinalHDL。

## 核心概念

在硬件模式下，Valkyrie 的代碼会被編譯為网表（Netlist），随後可以轉換為 Verilog 或直接通過 Gaia 後端发射為 FPGA 位流。

### 模組 (Module) 與 端口 (Bundle)

硬件設計的基本單元是 `Module`。端口定義使用 `structure` 並配合 `@derive(Bundle)`。

```valkyrie
@derive(Bundle)
structure CounterIO {
    @input  enable: u1,
    @input  reset: u1,
    @output count: u32,
    @output overflow: u1
}

structure Counter {
    max_count: u64
}

imply Counter: Module {
    type IO = CounterIO
    
    micro io(self) -> CounterIO {
        # 由 @derive(Bundle) 自動生成的構造函數，處理端口方向
        CounterIO::default()
    }
    
    micro elaborate(self) {
        let io = self.io()
        
        # 推荐寫法：函數 + 尾随閉包 (Trailing Closure)
        # 自動為作用域內的寄存器绑定時钟和复位
        with_clock_reset(Clock::global(), io.reset) {
            let count_reg = reg32::new(0_u32)
            
            let next_count = mux(
                io.enable,
                mux(
                    count_reg === self.max_count.as_u32(),
                    0_u32,
                    count_reg + 1_u32
                ),
                count_reg
            )
            
            connect(count_reg, next_count)
            connect(io.count, count_reg)
            connect(io.overflow, io.enable && (count_reg === self.max_count.as_u32()))
        }
    }
}
```

> **提示**: 除了尾随閉包，Valkyrie 还支持 `let local` 語法，變量会在作用域结束時自動釋放：
> ```valkyrie
> let local domain = ClockDomain::new(clock, reset)
> let count_reg = reg32::new(0_u32) # 自動关联當前的 local domain
> ```

## 組合邏輯

### 基本操作符

```valkyrie
# 邏輯操作
micro and<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W> {
    a & b
}

micro or<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W> {
    a | b
}

micro xor<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W> {
    a ^ b
}

micro not<const W: usize>(a: HardwareType<u64, W>) -> HardwareType<u64, W> {
    !a
}

# 算術操作
micro add<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W+1> {
    a + b
}

micro sub<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W+1> {
    a - b
}

micro mul<const W1: usize, const W2: usize>(a: HardwareType<u64, W1>, b: HardwareType<u64, W2>) -> HardwareType<u64, W1+W2> {
    a * b
}

# 比较操作
micro eq<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> u1 {
    a === b
}

micro lt<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> u1 {
    a < b
}

micro gt<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> u1 {
    a > b
}
```

### 位寬自動推導與安全檢查

Valkyrie 的編譯器能夠自動推導硬件信號的位寬，並在編譯阶段進行严格的安全檢查：

- **位寬傳播**：加法操作 `a + b` 会自動推導出比輸入位寬多 1 位的輸出位寬，以防止溢出。
- **類型安全**：不允許在不同位寬或不同類型的信號之間進行隐式連接，必须显式轉換（如使用 `.as_u32()`）。
- **静態檢查**：在生成 Verilog 之前，Valkyrie 会檢查所有的端口連接是否完整，是否存在未驅動的輸入或多個驅動源的輸出。

```valkyrie
micro width_inference_example(a: u8, b: u8) {
    # c 的位寬被推導為 u9，以容纳可能的进位
    let c = a + b 
    
    # 如果嘗試將 u9 連接到 u8 端口，編譯器会报错
    # let d: u8 = c # 編譯錯誤
    
    # 必须显式截斷或處理
    let d: u8 = c[7:0]
}
```

## ALU 設計

利用多路選擇器和基本操作符，可以輕鬆實現複雜的 ALU 模組。

```valkyrie
# ALU 操作碼
enum AluOp {
    Add = 0b0000,
    Sub = 0b0001,
    And = 0b0010,
    Or  = 0b0011,
    Xor = 0b0100,
    Sll = 0b0101,
    Srl = 0b0110,
    Sra = 0b0111
}

@derive(Bundle)
structure AluIO {
    @input  a: u32,
    @input  b: u32,
    @input  op: u4,
    @output result: u32,
    @output zero: u1,
    @output overflow: u1
}

structure Alu {
    width: usize
}

imply Alu: Module {
    type IO = AluIO
    
    micro io(self) -> AluIO {
        AluIO::default()
    }
    
    micro elaborate(self) {
        let io = self.io()
        
        let add_result = io.a + io.b
        let sub_result = io.a - io.b
        let and_result = io.a & io.b
        let or_result  = io.a | io.b
        let xor_result = io.a ^ io.b
        
        # 移位操作，仅取低 5 位
        let shamt = io.b[4:0]
        let sll_result = io.a << shamt
        let srl_result = io.a >> shamt
        
        # 使用 mux_lookup 進行操作碼匹配
        let result = mux_lookup(io.op, [
            add_result[31:0],
            sub_result[31:0],
            and_result,
            or_result,
            xor_result,
            sll_result,
            srl_result,
            0_u32 # 占位
        ])
        
        connect(io.result, result)
        connect(io.zero, result === 0_u32)
        
        # 溢出檢測（簡化範例）
        let is_add = io.op === 0_u4
        let overflow = is_add && add_result[32]
        connect(io.overflow, overflow)
    }
}
```

## 內存系統

Valkyrie 支持同步和異步內存模型。

### 單端口 RAM

```valkyrie
@derive(Bundle)
structure RamIO<const WIDTH: usize, const DEPTH: usize> {
    @input  addr: UInt<{log2_ceil(DEPTH)}>,
    @input  data_in: UInt<WIDTH>,
    @output data_out: UInt<WIDTH>,
    @input  we: u1, # 寫使能
    @input  en: u1  # 芯片使能
}

structure SinglePortRam<const WIDTH: usize, const DEPTH: usize> {}

imply<const WIDTH: usize, const DEPTH: usize> SinglePortRam<WIDTH, DEPTH>: Module {
    type IO = RamIO<WIDTH, DEPTH>
    
    micro io(self) -> RamIO<WIDTH, DEPTH> {
        RamIO::default()
    }
    
    micro elaborate(self) {
        let io = self.io()
        let mem = Mem::<UInt<WIDTH>, DEPTH>::new()
        
        # 寫操作
        when(io.en && io.we) {
            mem.write(io.addr, io.data_in)
        }
        
        # 讀操作
        let out = mux(io.en, mem.read(io.addr), 0_u(WIDTH))
        connect(io.data_out, out)
    }
}
```

## 處理器設計

Valkyrie 的元編程能力使其非常适合编寫複雜的處理器內核。

### 簡單的流水線連接

在多級流水線設計中，可以使用 `connect` 函數方便地將不同阶段的寄存器連接起來。

```valkyrie
micro pipeline_example() {
    let fetch_reg = reg32::new(0_u32)
    let decode_reg = reg32::new(0_u32)
    let execute_reg = reg32::new(0_u32)
    
    # 流水線逐級传遞
    connect(decode_reg, fetch_reg)
    connect(execute_reg, decode_reg)
}
```

通過這種方式，Valkyrie 允許開發者以高度結構化和類型安全的方式描述複雜的數字邏輯系統。
