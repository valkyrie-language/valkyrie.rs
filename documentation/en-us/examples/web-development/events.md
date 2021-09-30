# Event Handling

Valkyrie provides a unified event handling system that works across platforms, from web browsers to native applications.

## Basic Event Handling

### Click Events

```valkyrie
Button("Click me") {
    on_click {
        print("Button clicked!")
    }
}
```

### Event Object

```valkyrie
Button("Click me") {
    on_click { event |
        print("Clicked at: ({event.x}, {event.y})")
        event.prevent_default()
    }
}
```

## Event Types

### Mouse Events

```valkyrie
div {
    on_mouse_enter { print("Mouse entered") }
    on_mouse_leave { print("Mouse left") }
    on_mouse_move { event |
        update_position(event.x, event.y)
    }
    on_mouse_down { event |
        if event.button == MouseButton.left {
            start_drag(event)
        }
    }
    on_mouse_up { event |
        end_drag(event)
    }
}
```

### Keyboard Events

```valkyrie
input {
    on_key_down { event |
        match event.key {
            case "Enter" => submit_form()
            case "Escape" => cancel_edit()
            case _ => handle_input(event.key)
        }
    }
    on_key_up { event |
        # Handle key release
    }
}
```

### Form Events

```valkyrie
form {
    on_submit { event |
        event.prevent_default()
        submit_data()
    }
    
    input {
        on_change { event |
            validate_field(event.value)
        }
        on_input { event |
            # Real-time input handling
            update_preview(event.value)
        }
    }
}
```

## Event Modifiers

Valkkyrie supports event modifiers for common patterns:

```valkyrie
Button("Submit") {
    # Prevent default behavior
    on_click.prevent { 
        submit_async()
    }
    
    # Stop event propagation
    on_click.stop {
        # Event won't bubble to parent handlers
    }
    
    # Only trigger once
    on_click.once {
        show_welcome_message()
    }
    
    # Only when condition is true
    on_click.when(is_enabled) {
        perform_action()
    }
    
    # Combine modifiers
    on_click.prevent.stop.once {
        first_time_action()
    }
}
```

## Custom Events

### Define Custom Events

```valkyrie
class CounterChanged {
    old_value: i32
    new_value: i32
}

widget Counter {
    count: Signal⟨i32⟩ = Signal.new(0)
    
    micro increment(mut self) {
        let old = self.count.get()
        self.count.update { $ + 1 }
        self.emit(CounterChanged {
            old_value: old,
            new_value: self.count.get()
        })
    }
}
```

### Listen to Custom Events

```valkyrie
Counter {
    on_counter_changed { event |
        print("Count changed from {event.old_value} to {event.new_value}")
    }
}
```

## Event Delegation

For efficient handling of many similar elements:

```valkyrie
ul {
    on_click { event |
        # Check if a list item was clicked
        if let li = event.target.closest("li") {
            let id = li.data("id")
            select_item(id)
        }
    }
    
    for item in items {
        li { data_id: item.id; item.name }
    }
}
```

## Async Event Handlers

Event handlers can be asynchronous:

```valkyrie
Button("Load Data") {
    on_click async {
        let data = fetch_data().await?
        display_data(data)
    }
}
```

## Event Streams

Convert events to streams for reactive programming:

```valkyrie
using std::reactive

# Create observable from events
let clicks = Observable.from_event(button, "click")

# Process event stream
clicks
    .debounce(300ms)
    .filter { $.target.matches(".valid") }
    .subscribe { event |
        process_click(event)
    }
```

## Focus Events

```valkyrie
input {
    on_focus {
        input.select_all()
    }
    on_blur {
        validate_input()
    }
}
```

## Touch Events

```valkyrie
div {
    on_touch_start { event |
        start_gesture(event.touches[0])
    }
    on_touch_move { event |
        update_gesture(event.touches[0])
    }
    on_touch_end { event |
        complete_gesture()
    }
}
```

## Drag and Drop

```valkyrie
div {
    draggable = true
    
    on_drag_start { event |
        event.data_transfer.set_data("text/plain", item_id)
    }
    on_drag_over { event |
        event.prevent_default()  # Allow drop
    }
    on_drop { event |
        event.prevent_default()
        let id = event.data_transfer.get_data("text/plain")
        move_item(id, drop_target)
    }
}
```

## Best Practices

1. **Use modifiers** for common patterns like prevent_default
2. **Delegate events** when handling many similar elements
3. **Debounce** rapid-fire events like mouse_move
4. **Clean up** event listeners when components unmount
5. **Handle errors** in async event handlers

---
**Related Sections**:
- [Widget](./widget.md) - Component system
- [Observable](../../language/reactive-programming/observable.md) - Event streams
