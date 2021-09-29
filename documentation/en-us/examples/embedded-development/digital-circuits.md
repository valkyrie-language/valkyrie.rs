# Digital Circuits and Hardware Description (HDL)

Valkyrie can not only write traditional software but also serve as a **Hardware Description Language (HDL)** through specific backends and libraries, similar to Chisel or SpinalHDL.

## Core Concepts

In hardware mode, Valkyrie code is compiled to netlists, which can then be converted to Verilog or directly emitted as FPGA bitstreams through the Gaia backend.

### Modules and Ports

The basic unit of hardware design is the `Module`. Port definitions use `structure` with `@derive(Bundle)`.

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
        # Auto-generated constructor by @derive(Bundle), handling port directions
        CounterIO::default()
    }
    
    micro elaborate(self) {
        let io = self.io()
        
        # Recommended approach: Function + Trailing Closure
        # Automatically binds clock and reset for registers in scope
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

> **Tip**: Besides trailing closures, Valkyrie also supports `let local` syntax where variables are automatically released when the scope ends:
> ```valkyrie
> let local domain = ClockDomain::new(clock, reset)
> let count_reg = reg32::new(0_u32) # Automatically associates with current local domain
> ```

## Combinational Logic

### Basic Operators

```valkyrie
# Logic operations
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

# Arithmetic operations
micro add<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W+1> {
    a + b
}

micro sub<const W: usize>(a: HardwareType<u64, W>, b: HardwareType<u64, W>) -> HardwareType<u64, W+1> {
    a - b
}

micro mul<const W1: usize, const W2: usize>(a: HardwareType<u64, W1>, b: HardwareType<u64, W2>) -> HardwareType<u64, W1+W2> {
    a * b
}

# Comparison operations
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

### Automatic Bit-width Inference and Safety Checks

Valkyrie's compiler can automatically infer hardware signal bit-widths and perform strict safety checks at compile time:

- **Bit-width Propagation**: Addition operation `a + b` automatically infers an output bit-width one bit larger than the input to prevent overflow.
- **Type Safety**: Implicit connections between signals of different bit-widths or types are not allowed; explicit conversion is required (e.g., using `.as_u32()`).
- **Static Checks**: Before generating Verilog, Valkyrie checks that all port connections are complete and there are no undriven inputs or multiply-driven outputs.

```valkyrie
micro width_inference_example(a: u8, b: u8) {
    # c's bit-width is inferred as u9 to accommodate possible carry
    let c = a + b 
    
    # Attempting to connect u9 to u8 port will cause compiler error
    # let d: u8 = c # Compilation error
    
    # Must explicitly truncate or handle
    let d: u8 = c[7:0]
}
```

## ALU Design

Using multiplexers and basic operators, complex ALU modules can be easily implemented.

```valkyrie
# ALU opcode
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
        
        # Shift operations, only take lower 5 bits
        let shamt = io.b[4:0]
        let sll_result = io.a << shamt
        let srl_result = io.a >> shamt
        
        # Use mux_lookup for opcode matching
        let result = mux_lookup(io.op, [
            add_result[31:0],
            sub_result[31:0],
            and_result,
            or_result,
            xor_result,
            sll_result,
            srl_result,
            0_u32 # placeholder
        ])
        
        connect(io.result, result)
        connect(io.zero, result === 0_u32)
        
        # Overflow detection (simplified example)
        let is_add = io.op === 0_u4
        let overflow = is_add && add_result[32]
        connect(io.overflow, overflow)
    }
}
```

## Memory Systems

Valkyrie supports both synchronous and asynchronous memory models.

### Single-Port RAM

```valkyrie
@derive(Bundle)
structure RamIO<const WIDTH: usize, const DEPTH: usize> {
    @input  addr: UInt<{log2_ceil(DEPTH)}>,
    @input  data_in: UInt<WIDTH>,
    @output data_out: UInt<WIDTH>,
    @input  we: u1, # Write enable
    @input  en: u1  # Chip enable
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
        
        # Write operation
        when(io.en && io.we) {
            mem.write(io.addr, io.data_in)
        }
        
        # Read operation
        let out = mux(io.en, mem.read(io.addr), 0_u(WIDTH))
        connect(io.data_out, out)
    }
}
```

## Processor Design

Valkyrie's metaprogramming capabilities make it well-suited for writing complex processor cores.

### Simple Pipeline Connection

In multi-stage pipeline designs, the `connect` function can conveniently connect registers across different stages.

```valkyrie
micro pipeline_example() {
    let fetch_reg = reg32::new(0_u32)
    let decode_reg = reg32::new(0_u32)
    let execute_reg = reg32::new(0_u32)
    
    # Pipeline stage-by-stage propagation
    connect(decode_reg, fetch_reg)
    connect(execute_reg, decode_reg)
}
```

Through this approach, Valkyrie allows developers to describe complex digital logic systems in a highly structured and type-safe manner.
