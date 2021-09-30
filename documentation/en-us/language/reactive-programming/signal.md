# Signal

`Signal` is the core primitive for state management in Valkyrie. Unlike `Observable` which represents a sequence of events, `Signal` represents **current state** - it always has a value and can be read synchronously.

## Core Concept: Accessor and Settler

`Signal` is built on two fundamental traits:

- **Accessor⟨T⟩**: Provides read access to current state via `.get()`.
- **Settler⟨T⟩**: Provides write access via `.set()` and `.update()`.

A `Signal` combines both traits, making it a complete state container.

## Basic Usage

```valkyrie
# Create a signal with initial value
let count = Signal.new(0)

# Read current value (synchronous)
print(count.get())  # 0

# Set new value
count.set(5)

# Update based on current value
count.update { $ + 1 }  # Now 6
```

## Computed Signals

You can derive new signals that automatically update when dependencies change:

```valkyrie
let first_name = Signal.new("John")
let last_name = Signal.new("Doe")

# Computed signal - automatically recalculates
let full_name = Signal.derive {
    "{first_name.get()} {last_name.get()}"
}

print(full_name.get())  # "John Doe"

first_name.set("Jane")
print(full_name.get())  # "Jane Doe" - automatically updated
```

## Signal in UI Components

Signals integrate seamlessly with Valkyrie's UI system:

```valkyrie
widget Counter {
    # State declared as signal
    count: Signal⟨i32⟩ = Signal.new(0)
    
    Column {
        # UI automatically re-renders when signal changes
        Text("Count: {count.get()}")
        
        Row {
            Button("-") {
                on_click { count.update { $ - 1 } }
            }
            Button("+") {
                on_click { count.update { $ + 1 } }
            }
        }
    }
}
```

## Read-Only Signals

Sometimes you want to expose read access while keeping write access private:

```valkyrie
class Store {
    private state: Signal⟨AppState⟩
    public readonly: Accessor⟨AppState⟩
    
    micro new() {
        let signal = Signal.new(AppState.default())
        Self {
            state: signal,
            readonly: signal.read_only(),
        }
    }
    
    micro update(mut self, action: Action) {
        self.state.update { s | reducer(s, action) }
    }
}
```

## Batch Updates

For performance, you can batch multiple updates:

```valkyrie
Signal.batch {
    first_name.set("Jane")
    last_name.set("Smith")
    # Computed signals only recalculate once at the end
}
```

## Difference from Observable: State vs Events

| Feature | Signal (State) | Observable (Events) |
| :--- | :--- | :--- |
| **Represents** | "What it is" (current value) | "What happened" (action sequence) |
| **Initial Value** | Required | Not required |
| **Read** | Synchronous `.get()` | Subscribe to future values |
| **Update** | `.set()` / `.update()` | Emit new values |
| **Use Cases** | UI state, configuration | Click events, messages, streams |

## Fine-grained Reactivity

Valkyrie's signal system supports fine-grained reactivity, meaning only the parts of the UI that actually use a signal will re-render when it changes:

```valkyrie
widget UserProfile {
    user: Signal⟨User⟩
    
    Column {
        # Only this Text re-renders when name changes
        Text("{user.get().name}")
        
        # Only this Text re-renders when email changes
        Text("{user.get().email}")
        
        # This part is not affected by name/email changes
        StaticFooter { }
    }
}
```

---
**Related Sections**:
- [Observable](./observable.md) - For event streams and sequences
- [Widget](../../examples/web-development/widget.md) - UI component system
