# 單位制系統

Valkyrie 提供了強大的編譯時單位制系統，透過宏和型別系統確保物理量計算的正確性，防止單位不匹配的錯誤。

## 基本單位定義

### SI 基本單位

```valkyrie
# 基本單位宏
let mass = 1kg        # 公斤
let length = 1m       # 公尺
let time = 1s         # 秒
let current = 1A      # 安培
let temperature = 1K  # 開爾文
let amount = 1mol     # 莫耳
let luminosity = 1cd  # 坎德拉

# 使用基本單位
let distance: Length = 100m
let duration: Time = 5s
let weight: Mass = 2.5kg
let temp: Temperature = 273.15K
```

### 導出單位

```valkyrie
# 面積單位
let area1 = 1m²       # 平方公尺
let area2 = 1m × 1m   # 等價寫法
let area3 = 1m ^ 2     # 指數冪寫法

# 體積單位 (L³)
let volume1 = 1m³     # 立方公尺
let volume2 = 1m × 1m × 1m  # 等價寫法
let volume3 = 1m ^ 3        # 指數冪寫法

# 速度單位
let velocity1 = 1m/s  # 公尺每秒
let velocity2 = 1m / 1s  # 等價寫法

# 加速度單位
let acceleration = 1m/s²  # 公尺每秒平方

# 組合單位
let force1 = 1N       # 牛頓
let force2 = 1kg × 1m/s²  # 等價定義

# 能量單位
let energy1 = 1J      # 焦耳
let energy2 = 1N × 1m # 等價定義
let energy3 = 1kg × 1m²/s²  # 基本單位表示

# 功率單位
let power1 = 1W       # 瓦特
let power2 = 1J/s     # 等價定義
```

## 單位型別系統

### 量綱型別

```valkyrie
# 量綱型別定義
type Length = Quantity⟨[1, 0, 0, 0, 0, 0, 0]⟩     # [L, M, T, I, Θ, N, J]
type Mass = Quantity⟨[0, 1, 0, 0, 0, 0, 0]⟩
type Time = Quantity⟨[0, 0, 1, 0, 0, 0, 0]⟩
type Area = Quantity⟨[2, 0, 0, 0, 0, 0, 0]⟩
type Volume = Quantity⟨[3, 0, 0, 0, 0, 0, 0]⟩
type Velocity = Quantity⟨[1, 0, -1, 0, 0, 0, 0]⟩
type Acceleration = Quantity⟨[1, 0, -2, 0, 0, 0, 0]⟩
type Force = Quantity⟨[1, 1, -2, 0, 0, 0, 0]⟩
type Energy = Quantity⟨[2, 1, -2, 0, 0, 0, 0]⟩
type Power = Quantity⟨[2, 1, -3, 0, 0, 0, 0]⟩

# 使用量綱型別
micro calculate_kinetic_energy(mass: Mass, velocity: Velocity) -> Energy {
    0.5 × mass × velocity ^ 2
}

micro calculate_power(energy: Energy, time: Time) -> Power {
    energy / time
}
```

### 單位轉換

```valkyrie
# 長度單位轉換
let meter = 1m
let kilometer = 1km     # 1000m
let centimeter = 1cm    # 0.01m
let millimeter = 1mm    # 0.001m
let inch = 1inch        # 0.0254m
let foot = 1ft          # 0.3048m
let yard = 1yd          # 0.9144m
let mile = 1mile        # 1609.344m

# 自動轉換
let distance1: Length = 5km
let distance2: Length = 3000m
let total_distance = distance1 + distance2  # 8000m

# 顯式轉換
let km_value = distance1.to(km)  # 5.0
let m_value = distance1.to(m)    # 5000.0
```

### 質量單位

```valkyrie
# 質量單位
let kilogram = 1kg
let gram = 1g           # 0.001kg
let ton = 1t            # 1000kg
let pound = 1lb         # 0.453592kg
let ounce = 1oz         # 0.0283495kg

# 質量計算
let total_mass = 2kg + 500g  # 2.5kg
let density = total_mass / (1m³)  # 密度型別
```

### 時間單位

```valkyrie
# 時間單位
let second = 1s
let minute = 1min       # 60s
let hour = 1h           # 3600s
let day = 1day          # 86400s
let week = 1week        # 604800s
let year = 1year        # 31557600s (365.25 days)

# 時間計算
let duration = 2h + 30min + 15s  # 9015s
let frequency = 1 / duration      # 頻率型別
```

## 複合單位計算

### 物理公式

```valkyrie
# 自動推導
micro get_force(mass: Mass, acceleration: Acceleration) -> Force {
    mass × acceleration
}

# 動能公式: E = 1/2 × m × v²
micro get_energy(mass: Mass, velocity: Velocity) -> Energy {
    0.5 × mass × velocity ^ 2
}

# 勢能公式: Ep = mgh
micro get_potential_energy(mass: Mass, g: Acceleration, height: Length) -> Energy {
    mass × g × height
}

# 功率公式: P = W/t
micro power_from_work(work: Energy, time: Time) -> Power {
    work / time
}

# 歐姆定律: V = IR
micro get_voltage(current: Current, resistance: Resistance) -> Voltage {
    current × resistance
}
```

### 電學單位

```valkyrie
# 電學基本單位
let current = 1A        # 安培
let voltage = 1V        # 伏特
let resistance = 1Ω     # 歐姆
let capacitance = 1F    # 法拉
let inductance = 1H     # 亨利
let charge = 1C         # 庫侖

# 組合電學單位
let power_electrical = voltage × current  # 電功率
let energy_stored = 0.5 × capacitance × voltage ^ 2  # 電容儲能
let magnetic_energy = 0.5 × inductance × current ^ 2  # 電感儲能
```

### 熱力學單位

```valkyrie
# 溫度單位
let kelvin = 1K
let celsius = 1°C       # 相對溫度
let fahrenheit = 1°F    # 相對溫度

# 熱量單位
let joule = 1J
let calorie = 1cal      # 4.184J
let btu = 1BTU          # 1055.06J

# 熱力學計算
micro heat_capacity(heat: Energy, temp_change: Temperature) -> HeatCapacity {
    heat / temp_change
}

micro thermal_conductivity(heat_flow: Power, area: Area, temp_gradient: Temperature, length: Length) -> ThermalConductivity {
    heat_flow × length / (area × temp_gradient)
}
```

## 單位制驗證

### 編譯時檢查

```valkyrie
# 正確的單位運算
let distance = 100m
let time = 10s
let speed = distance / time  # 10 m/s，型別正確

# 編譯錯誤範例
# let invalid = distance + time  # 錯誤：不能將長度和時間相加
# let wrong_speed = distance × time  # 錯誤：結果不是速度型別

# 函式參數型別檢查
micro calculate_work(force: Force, distance: Length) -> Energy {
    force × distance
}

# 呼叫時會進行型別檢查
let work = calculate_work(10N, 5m)  # 正確
# let invalid_work = calculate_work(10m, 5N)  # 錯誤：參數型別不匹配
```

### 執行階段單位轉換

```valkyrie
# 單位轉換函式
class UnitConverter {
    conversion_factors: HashMap<(Unit, Unit), f64>,
}

impl UnitConverter {
    micro convert⟨T: Dimension⟩(self, value: Quantity⟨T⟩, from: Unit, to: Unit) -> Quantity⟨T⟩ {
        if from == to {
            return value
        }
        
        let factor = self.conversion_factors.get(&(from, to))
            .or_else { self.conversion_factors.get(&(to, from)).map { 1.0 / $ } }
            .expect("No conversion factor found")
        
        Quantity::new(value.value() × factor)
    }
}

# 使用轉換器
let converter = UnitConverter::default()
let distance_m = converter.convert(5km, km, m)  # 5000m
let mass_kg = converter.convert(10lb, lb, kg)   # 4.53592kg
```

## 自定義單位系統

### 定義新的單位

```valkyrie
# 定義新的基本單位
macro_rules! define_unit {
    ($name:ident, $symbol:literal, $dimension:expr) => {
        pub class $name;
        
        impl Unit for $name {
            const SYMBOL: utf8 = $symbol;
            type Dimension = $dimension;
        }
        
        pub const $name: Quantity⟨$dimension⟩ = Quantity::new(1.0);
    }
}

# 使用宏定義新單位
define_unit!(Pixel, "px", Length);  # 像素作為長度單位
define_unit!(Byte, "B", Information);  # 位元組作為資訊單位
define_unit!(Bit, "bit", Information);

# 資訊單位計算
let file_size = 1024Byte
let bandwidth = file_size / 1s  # 位元組每秒
```

### 複合單位宏

```valkyrie
# 定義複合單位的宏
macro_rules! compound_unit {
    ($name:ident = $($unit:ident)^$power:expr)*) => {
        type $name = Quantity⟨[
            $($unit::Dimension::EXPONENTS[0] × $power +)* 0,
            $($unit::Dimension::EXPONENTS[1] × $power +)* 0,
            $($unit::Dimension::EXPONENTS[2] × $power +)* 0,
            $($unit::Dimension::EXPONENTS[3] × $power +)* 0,
            $($unit::Dimension::EXPONENTS[4] × $power +)* 0,
            $($unit::Dimension::EXPONENTS[5] × $power +)* 0,
            $($unit::Dimension::EXPONENTS[6] × $power +)* 0,
        ]⟩;
    }
}

# 使用複合單位宏
compound_unit!(Density = Mass^1 Length^-3);  # kg/m³
compound_unit!(Pressure = Mass^1 Length^-1 Time^-2);  # Pa = kg/(m·s²)
compound_unit!(ElectricField = Mass^1 Length^1 Time^-3 Current^-1);  # V/m
```

## 單位制應用範例

### 物理模擬

```valkyrie
# 粒子物理模擬
class Particle {
    mass: Mass,
    position: Vector3⟨Length⟩,
    velocity: Vector3⟨Velocity⟩,
    acceleration: Vector3⟨Acceleration⟩,
}

impl Particle {
    micro apply_force(mut self, force: Vector3⟨Force⟩, dt: Time) {
        # F = ma => a = F/m
        self.acceleration = force / self.mass
        
        # 更新速度和位置
        self.velocity += self.acceleration × dt
        self.position += self.velocity × dt
    }
    
    micro kinetic_energy(self) -> Energy {
        0.5 × self.mass × self.velocity.magnitude_squared()
    }
}

# 重力場模擬
class GravityField {
    g: Acceleration,  # 重力加速度
}

impl GravityField {
    micro force_on(self, particle: Particle) -> Vector3⟨Force⟩ {
        Vector3::new(0, -particle.mass × self.g, 0)
    }
}
```

### 工程計算

```valkyrie
# 結構工程計算
class Beam {
    length: Length,
    cross_section_area: Area,
    moment_of_inertia: MomentOfInertia,
    elastic_modulus: Pressure,
}

impl Beam {
    micro max_deflection(self, load: Force) -> Length {
        # 簡支梁中點最大撓度公式
        load × self.length ^ 3 / (48 × self.elastic_modulus × self.moment_of_inertia)
    }
    
    micro max_stress(self, moment: Moment, distance: Length) -> Pressure {
        moment × distance / self.moment_of_inertia
    }
}

# 流體力學計算
micro reynolds_number(density: Density, velocity: Velocity, length: Length, viscosity: DynamicViscosity) -> Dimensionless {
    density × velocity × length / viscosity
}

micro drag_force(drag_coefficient: Dimensionless, density: Density, velocity: Velocity, area: Area) -> Force {
    0.5 × drag_coefficient × density × velocity ^ 2 × area
}
```

### 電路分析

```valkyrie
# 電路元件
class Resistor {
    resistance: Resistance,
}

class Capacitor {
    capacitance: Capacitance,
}

class Inductor {
    inductance: Inductance,
}

# 電路分析函式
micro rc_time_constant(resistance: Resistance, capacitance: Capacitance) -> Time {
    resistance × capacitance
}

micro lc_resonant_frequency(inductance: Inductance, capacitance: Capacitance) -> Frequency {
    1 / (2 × PI × (inductance × capacitance).sqrt())
}

micro power_dissipation(voltage: Voltage, current: Current) -> Power {
    voltage × current
}
```

## 效能優化

### 零成本抽象

```valkyrie
# 編譯時單位消除
#[inline(always)]
micro optimized_calculation(distance: Length, time: Time) -> Velocity {
    # 編譯後等價於: distance_value / time_value
    distance / time
}

# 常數折疊
const GRAVITY: Acceleration = 9.81m/s²;
const EARTH_RADIUS: Length = 6.371e6m;

# 編譯時計算
const ESCAPE_VELOCITY: Velocity = (2 × GRAVITY × EARTH_RADIUS).sqrt();
```

### 批次計算優化

```valkyrie
# SIMD 向量化單位計算
class VectorQuantity⟨T: Dimension, const N: usize⟩ {
    values: [f64; N],
    _phantom: PhantomData⟨T⟩,
}

impl⟨T: Dimension, const N: usize⟩ VectorQuantity⟨T, N⟩ {
    micro add(self, other: Self) -> Self {
        let mut result = [f64; N]::new(0.0);
        loop i in 0..N {
            result[i] = self.values[i] + other.values[i];
        }
        Self { values: result, _phantom: PhantomData }
    }
    
    micro multiply_scalar(self, scalar: f64) -> Self {
        let mut result = [f64; N]::new(0.0);
        loop i in 0..N {
            result[i] = self.values[i] × scalar;
        }
        Self { values: result, _phantom: PhantomData }
    }
}
```

## 最佳實踐

### 1. 單位一致性

```valkyrie
# 始終使用帶單位的字面量
let good_distance = 100m;  # 好的做法
# let bad_distance = 100;  # 避免無單位數值

# 函式簽名明確單位
micro calculate_area(width: Length, height: Length) -> Area {
    width × height
}

# 而不是
# micro calculate_area(width: f64, height: f64) -> f64
```

### 2. 單位轉換策略

```valkyrie
# 在邊界處進行單位轉換
micro process_user_input(distance_str: utf8, unit_str: utf8) -> Result⟨Length, Any⟩ {
    let value: f64 = distance_str.parse()?;
    match unit_str {
        "m" => Ok(value × 1m),
        "km" => Ok(value × 1km),
        "ft" => Ok(value × 1ft),
        _ => Err(Any::UnknownUnit),
    }
}

# 內部計算保持一致單位
micro internal_calculation(distances: [Length]) -> Length {
    distances.iter().sum()  # 自動處理單位
}
```

### 3. 錯誤處理

```valkyrie
# 單位不匹配的錯誤處理
union UnitError {
    IncompatibleUnits { expected: utf8, found: utf8 },
    ConversionNotFound { from: utf8, to: utf8 },
    InvalidValue { value: f64 }
}

# 安全的單位操作
micro safe_divide⟨T: Dimension, U: Dimension⟩(numerator: Quantity⟨T⟩, denominator: Quantity⟨U⟩) -> Result⟨Quantity⟨T::Div⟨U⟩⟩, UnitError⟩ {
    if denominator.value().abs() < f64::EPSILON {
        return Err(UnitError::InvalidValue(denominator.value()))
    }
    Ok(numerator / denominator)
}
```

Valkyrie 的單位制系統透過編譯時檢查和零成本抽象，為科學計算和工程應用提供了型別安全的物理量計算能力，有效防止了單位錯誤導致的計算問題。
