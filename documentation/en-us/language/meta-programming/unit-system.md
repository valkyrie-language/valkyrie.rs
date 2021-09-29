# Unit System

Valkyrie provides a powerful compile-time unit system that ensures correctness of physical quantity calculations through macros and type system, preventing unit mismatch errors.

## Basic Unit Definitions

### SI Base Units

```valkyrie
# Base unit macros
let mass = 1kg        # Kilogram
let length = 1m       # Meter
let time = 1s         # Second
let current = 1A      # Ampere
let temperature = 1K  # Kelvin
let amount = 1mol     # Mole
let luminosity = 1cd  # Candela

# Using base units
let distance: Length = 100m
let duration: Time = 5s
let weight: Mass = 2.5kg
let temp: Temperature = 273.15K
```

### Derived Units

```valkyrie
# Area units
let area1 = 1m²       # Square meter
let area2 = 1m × 1m   # Equivalent notation
let area3 = 1m ^ 2     # Exponent notation

# Volume units (L³)
let volume1 = 1m³     # Cubic meter
let volume2 = 1m × 1m × 1m  # Equivalent notation
let volume3 = 1m ^ 3        # Exponent notation

# Velocity units
let velocity1 = 1m/s  # Meter per second
let velocity2 = 1m / 1s  # Equivalent notation

# Acceleration units
let acceleration = 1m/s²  # Meter per second squared

# Composite units
let force1 = 1N       # Newton
let force2 = 1kg × 1m/s²  # Equivalent definition

# Energy units
let energy1 = 1J      # Joule
let energy2 = 1N × 1m # Equivalent definition
let energy3 = 1kg × 1m²/s²  # Base unit representation

# Power units
let power1 = 1W       # Watt
let power2 = 1J/s     # Equivalent definition
```

## Unit Type System

### Dimension Types

```valkyrie
# Dimension type definitions
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

# Using dimension types
micro calculate_kinetic_energy(mass: Mass, velocity: Velocity) -> Energy {
    0.5 × mass × velocity ^ 2
}

micro calculate_power(energy: Energy, time: Time) -> Power {
    energy / time
}
```

### Unit Conversion

```valkyrie
# Length unit conversion
let meter = 1m
let kilometer = 1km     # 1000m
let centimeter = 1cm    # 0.01m
let millimeter = 1mm    # 0.001m
let inch = 1inch        # 0.0254m
let foot = 1ft          # 0.3048m
let yard = 1yd          # 0.9144m
let mile = 1mile        # 1609.344m

# Automatic conversion
let distance1: Length = 5km
let distance2: Length = 3000m
let total_distance = distance1 + distance2  # 8000m

# Explicit conversion
let km_value = distance1.to(km)  # 5.0
let m_value = distance1.to(m)    # 5000.0
```

### Mass Units

```valkyrie
# Mass units
let kilogram = 1kg
let gram = 1g           # 0.001kg
let ton = 1t            # 1000kg
let pound = 1lb         # 0.453592kg
let ounce = 1oz         # 0.0283495kg

# Mass calculation
let total_mass = 2kg + 500g  # 2.5kg
let density = total_mass / (1m³)  # Density type
```

### Time Units

```valkyrie
# Time units
let second = 1s
let minute = 1min       # 60s
let hour = 1h           # 3600s
let day = 1day          # 86400s
let week = 1week        # 604800s
let year = 1year        # 31557600s (365.25 days)

# Time calculation
let duration = 2h + 30min + 15s  # 9015s
let frequency = 1 / duration      # Frequency type
```

## Composite Unit Calculations

### Physics Formulas

```valkyrie
# Automatic derivation
micro get_force(mass: Mass, acceleration: Acceleration) -> Force {
    mass × acceleration
}

# Kinetic energy formula: E = 1/2 × m × v²
micro get_energy(mass: Mass, velocity: Velocity) -> Energy {
    0.5 × mass × velocity ^ 2
}

# Potential energy formula: Ep = mgh
micro get_potential_energy(mass: Mass, g: Acceleration, height: Length) -> Energy {
    mass × g × height
}

# Power formula: P = W/t
micro power_from_work(work: Energy, time: Time) -> Power{
    work / time
}

# Ohm's law: V = IR
micro get_voltage(current: Current, resistance: Resistance) -> Voltage{
    current × resistance
}
```

### Electrical Units

```valkyrie
# Electrical base units
let current = 1A        # Ampere
let voltage = 1V        # Volt
let resistance = 1Ω     # Ohm
let capacitance = 1F    # Farad
let inductance = 1H     # Henry
let charge = 1C         # Coulomb

# Composite electrical units
let power_electrical = voltage × current  # Electrical power
let energy_stored = 0.5 × capacitance × voltage ^ 2  # Capacitor energy storage
let magnetic_energy = 0.5 × inductance × current ^ 2  # Inductor energy storage
```

### Thermodynamic Units

```valkyrie
# Temperature units
let kelvin = 1K
let celsius = 1°C       # Relative temperature
let fahrenheit = 1°F    # Relative temperature

# Heat units
let joule = 1J
let calorie = 1cal      # 4.184J
let btu = 1BTU          # 1055.06J

# Thermodynamic calculations
micro heat_capacity(heat: Energy, temp_change: Temperature) -> HeatCapacity{
    heat / temp_change
}

micro thermal_conductivity(heat_flow: Power, area: Area, temp_gradient: Temperature, length: Length) -> ThermalConductivity{
    heat_flow × length / (area × temp_gradient)
}
```

## Unit System Verification

### Compile-time Checking

```valkyrie
# Correct unit operations
let distance = 100m
let time = 10s
let speed = distance / time  # 10 m/s, correct type

# Compile error examples
# let invalid = distance + time  # Error: cannot add length and time
# let wrong_speed = distance × time  # Error: result is not velocity type

# Function parameter type checking
micro calculate_work(force: Force, distance: Length) -> Energy{
    force × distance
}

# Type checking during calls
let work = calculate_work(10N, 5m)  # Correct
# let invalid_work = calculate_work(10m, 5N)  # Error: parameter type mismatch
```

### Runtime Unit Conversion

```valkyrie
# Unit conversion function
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

# Using converter
let converter = UnitConverter::default()
let distance_m = converter.convert(5km, km, m)  # 5000m
let mass_kg = converter.convert(10lb, lb, kg)   # 4.53592kg
```

## Custom Unit Systems

### Defining New Units

```valkyrie
# Define new base unit
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

# Use macro to define new unit
define_unit!(Pixel, "px", Length);  # Pixel as length unit
define_unit!(Byte, "B", Information);  # Byte as information unit
define_unit!(Bit, "bit", Information);

# Information unit calculation
let file_size = 1024Byte
let bandwidth = file_size / 1s  # Bytes per second
```

### Composite Unit Macro

```valkyrie
# Macro for defining composite units
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

# Using composite unit macro
compound_unit!(Density = Mass^1 Length^-3);  # kg/m³
compound_unit!(Pressure = Mass^1 Length^-1 Time^-2);  # Pa = kg/(m·s²)
compound_unit!(ElectricField = Mass^1 Length^1 Time^-3 Current^-1);  # V/m
```

## Unit System Application Examples

### Physics Simulation

```valkyrie
# Particle physics simulation
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
        
        # Update velocity and position
        self.velocity += self.acceleration × dt
        self.position += self.velocity × dt
    }
    
    micro kinetic_energy(self) -> Energy {
        0.5 × self.mass × self.velocity.magnitude_squared()
    }
}

# Gravity field simulation
class GravityField {
    g: Acceleration,  # Gravitational acceleration
}

impl GravityField {
    micro force_on(self, particle: Particle) -> Vector3⟨Force⟩ {
        Vector3::new(0, -particle.mass × self.g, 0)
    }
}
```

### Engineering Calculations

```valkyrie
# Structural engineering calculations
class Beam {
    length: Length,
    cross_section_area: Area,
    moment_of_inertia: MomentOfInertia,
    elastic_modulus: Pressure,
}

impl Beam {
    micro max_deflection(self, load: Force) -> Length {
        # Simply supported beam mid-point maximum deflection formula
        load × self.length ^ 3 / (48 × self.elastic_modulus × self.moment_of_inertia)
    }
    
    micro max_stress(self, moment: Moment, distance: Length) -> Pressure {
        moment × distance / self.moment_of_inertia
    }
}

# Fluid dynamics calculations
micro reynolds_number(density: Density, velocity: Velocity, length: Length, viscosity: DynamicViscosity) -> Dimensionless{
    density × velocity × length / viscosity
}

micro drag_force(drag_coefficient: Dimensionless, density: Density, velocity: Velocity, area: Area) -> Force{
    0.5 × drag_coefficient × density × velocity ^ 2 × area
}
```

### Circuit Analysis

```valkyrie
# Circuit components
class Resistor {
    resistance: Resistance,
}

class Capacitor {
    capacitance: Capacitance,
}

class Inductor {
    inductance: Inductance,
}

# Circuit analysis functions
micro rc_time_constant(resistance: Resistance, capacitance: Capacitance) -> Time {
    resistance × capacitance
}

micro lc_resonant_frequency(inductance: Inductance, capacitance: Capacitance) -> Frequency{
    1 / (2 × PI × (inductance × capacitance).sqrt())
}

micro power_dissipation(voltage: Voltage, current: Current) -> Power{
    voltage × current
}
```

## Performance Optimization

### Zero-cost Abstraction

```valkyrie
# Compile-time unit elimination
#[inline(always)]
micro optimized_calculation(distance: Length, time: Time) -> Velocity{
    # After compilation equivalent to: distance_value / time_value
    distance / time
}

# Constant folding
const GRAVITY: Acceleration = 9.81m/s²;
const EARTH_RADIUS: Length = 6.371e6m;

# Compile-time calculation
const ESCAPE_VELOCITY: Velocity = (2 × GRAVITY × EARTH_RADIUS).sqrt();
```

### Batch Calculation Optimization

```valkyrie
# SIMD vectorized unit calculation
class VectorQuantity⟨T: Dimension, const N: usize⟩ {
    values: [f64; N],
    _phantom: PhantomData⟨T⟩,
}

impl⟨T: Dimension, const N: usize⟩ VectorQuantity⟨T, N⟩ {
    micro add(self, other: Self) -> Self {
        let mut result = [f64; N]::new(0.0);
        for i in 0..N {
            result[i] = self.values[i] + other.values[i];
        }
        Self { values: result, _phantom: PhantomData }
+        Self { values: result, _phantom: PhantomData }
    }
    
    micro multiply_scalar(self, scalar: f64) -> Self {
        let mut result = [f64; N]::new(0.0)
        for i in 0..N {
            result[i] = self.values[i] * scalar
        }
        result
    }
}
        Self { values: result, _phantom: PhantomData }
    }
}
```

## Best Practices

### 1. Unit Consistency

```valkyrie
# Always use unit literals
let good_distance = 100m;  # Good practice
# let bad_distance = 100;  # Avoid unitless values

# Function signatures with explicit units
micro calculate_area(width: Length, height: Length) -> Area{
    width × height
}

# Instead of
# micro calculate_area(width: f64, height: f64) -> f64
```

### 2. Unit Conversion Strategy

```valkyrie
# Perform unit conversion at boundaries
micro process_user_input(distance_str: utf8, unit_str: utf8) -> Result⟨Length, Any⟩ {
    let value: f64 = distance_str.parse()?;
    match unit_str {
        "m" => Ok(value × 1m),
        "km" => Ok(value × 1km),
        "ft" => Ok(value × 1ft),
        _ => Err(Any::UnknownUnit),
    }
}

# Internal calculations maintain consistent units
micro internal_calculation(distances: [Length]) -> Length{
    distances.iter().sum()  # Automatically handles units
}
```

### 3. Error Handling

```valkyrie
# Unit mismatch error handling
union UnitError {
    IncompatibleUnits { expected: utf8, found: utf8 },
    ConversionNotFound { from: utf8, to: utf8 },
    InvalidValue { value: f64 }
}

# Safe unit operations
micro safe_divide⟨T: Dimension, U: Dimension⟩(numerator: Quantity⟨T⟩, denominator: Quantity⟨U⟩) -> Result⟨Quantity⟨T::Div⟨U⟩⟩, UnitError⟩ {
    if denominator.value().abs() < f64::EPSILON {
        return Err(UnitError::InvalidValue(denominator.value()))
    }
    Ok(numerator / denominator)
}
```

Valkyrie's unit system provides type-safe physical quantity calculation capabilities through compile-time checking and zero-cost abstraction for scientific computing and engineering applications, effectively preventing calculation issues caused by unit errors.
