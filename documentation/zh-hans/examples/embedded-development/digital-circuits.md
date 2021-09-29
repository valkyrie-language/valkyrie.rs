# 数字电路与硬件描述 (HDL)

Valkyrie 不仅可以编写传统的软件，还可以通过特定的后端和库，作为**硬件描述语言 (HDL)** 使用，类似于 Chisel 或 SpinalHDL。

## 核心概念

在硬件模式下，Valkyrie 的代码会被编译为网表（Netlist），随后可以转换为 Verilog 或直接通过 Gaia 后端发射为 FPGA 位流。

### 模块 (Module) 与 端口 (Bundle)

硬件设计的基本单元是 `Module`。端口定义使用 `structure` 并配合 `@derive(Bundle)`。

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
        # 由 @derive(Bundle) 自动生成的构造函数，处理端口方向
        CounterIO::default()
    }
    
    micro elaborate(self) {
        let io = self.io()
        
        # 推荐写法：函数 + 尾随闭包 (Trailing Closure)
        # 自动为作用域内的寄存器绑定时钟和复位
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

> **提示**: 除了尾随闭包，Valkyrie 还支持 `let local` 语法，变量会在作用域结束时自动释放：
> ```valkyrie
> let local domain = ClockDomain::new(clock, reset)
> let count_reg = reg32::new(0_u32) # 自动关联当前的 local domain
> ```

## 组合逻辑

### 基本操作符

```valkyrie
# 逻辑操作
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

# 算术操作
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

### 位宽自动推导与安全检查

Valkyrie 的编译器能够自动推导硬件信号的位宽，并在编译阶段进行严格的安全检查：

- **位宽传播**：加法操作 `a + b` 会自动推导出比输入位宽多 1 位的输出位宽，以防止溢出。
- **类型安全**：不允许在不同位宽或不同类型的信号之间进行隐式连接，必须显式转换（如使用 `.as_u32()`）。
- **静态检查**：在生成 Verilog 之前，Valkyrie 会检查所有的端口连接是否完整，是否存在未驱动的输入或多个驱动源的输出。

```valkyrie
micro width_inference_example(a: u8, b: u8) {
    # c 的位宽被推导为 u9，以容纳可能的进位
    let c = a + b 
    
    # 如果尝试将 u9 连接到 u8 端口，编译器会报错
    # let d: u8 = c # 编译错误
    
    # 必须显式截断或处理
    let d: u8 = c[7:0]
}
```

## ALU 设计

利用多路选择器和基本操作符，可以轻松实现复杂的 ALU 模块。

```valkyrie
# ALU 操作码
enums AluOp {
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
        
        # 使用 mux_lookup 进行操作码匹配
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
        
        # 溢出检测（简化示例）
        let is_add = io.op === 0_u4
        let overflow = is_add && add_result[32]
        connect(io.overflow, overflow)
    }
}
```

## 内存系统

Valkyrie 支持同步和异步内存模型。

### 单端口 RAM

```valkyrie
@derive(Bundle)
structure RamIO<const WIDTH: usize, const DEPTH: usize> {
    @input  addr: UInt<{log2_ceil(DEPTH)}>,
    @input  data_in: UInt<WIDTH>,
    @output data_out: UInt<WIDTH>,
    @input  we: u1, # 写使能
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
        
        # 写操作
        when(io.en && io.we) {
            mem.write(io.addr, io.data_in)
        }
        
        # 读操作
        let out = mux(io.en, mem.read(io.addr), 0_u(WIDTH))
        connect(io.data_out, out)
    }
}
```

## 处理器设计

Valkyrie 的元编程能力使其非常适合编写复杂的处理器内核。

### 简单的流水线连接

在多级流水线设计中，可以使用 `connect` 函数方便地将不同阶段的寄存器连接起来。

```valkyrie
micro pipeline_example() {
    let fetch_reg = reg32::new(0_u32)
    let decode_reg = reg32::new(0_u32)
    let execute_reg = reg32::new(0_u32)
    
    # 流水线逐级传递
    connect(decode_reg, fetch_reg)
    connect(execute_reg, decode_reg)
}
```

通过这种方式，Valkyrie 允许开发者以高度结构化和类型安全的方式描述复杂的数字逻辑系统。
