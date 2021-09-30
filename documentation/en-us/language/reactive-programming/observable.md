# Observable

`Observable` is the core of reactive programming, representing a sequence of values produced over time.

## Core Concept

Unlike `Future` which only returns a single value, `Observable` can:
- Produce 0 or more values.
- End at any time.
- Also produce errors and terminate.

## Basic Definition

```valkyrie
trait Observable⟨T⟩ {
    # Subscribe to this observable
    micro subscribe(self, observer: Observer⟨T⟩) -> Subscription
}
```

## Creating Observables

You can create observables from various sources:

```valkyrie
# Create from array
let obs1 = Observable.from([1, 2, 3])

# Create from timer
let obs2 = Observable.interval(Duration.seconds(1))

# Create from event
let obs3 = Observable.from_event(button, "click")
```

## Reactive Transformations

Valkyrie supports fluent operators to process these values:

```valkyrie
let processed = obs1
    .filter { $ % 2 == 0 }
    .map { value -> "Value: {value}" }
    .debounce(Duration.ms(300))
```

## Subscription and Resource Management

When you no longer need to listen, you can explicitly unsubscribe:

```valkyrie
let sub = obs.subscribe { value ->
    print("Received: {value}")
}

# Unsubscribe later
sub.unsubscribe()
```

## Difference from Signal: Events vs State

This is the core distinction in Valkyrie's reactive architecture:

| Feature | Observable (Event) | Signal (State) |
| :--- | :--- | :--- |
| **Represents** | "What happened" (action sequence) | "What it is" (current value) |
| **Timeliness** | Instantaneous, discrete | Continuous, persistent |
| **Execution Nature** | Lazy: No work without subscribers | Eager: Always holds a value |
| **Update Mechanism** | Asynchronous push | Synchronous tracking (Pull-Push hybrid) |
| **Use Cases** | Click events, sockets, timers | UI binding, configuration, business state |

---
**Related Sections**:
- [Signal](./signal.md) - Accessor / Settler abstraction representing current state
- [Stream](./stream.md) - Asynchronous iterator
