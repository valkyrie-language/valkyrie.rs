# Web Development

Valkyrie provides comprehensive support for web development, from server-side rendering to interactive client applications.

## Key Features

### Full-Stack Development
- Write once, run on server and client
- Server-side rendering (SSR) support
- Progressive enhancement

### Modern UI Framework
- Declarative component model
- Reactive state management
- Efficient DOM updates

### Type Safety
- End-to-end type checking
- API contract enforcement
- Compile-time error detection

## Quick Start

### Hello World

```valkyrie
micro main() {
    html {
        head {
            title { "Valkyrie App" }
        }
        body {
            h1 { "Hello, Valkyrie!" }
            p { "Welcome to web development with Valkyrie." }
        }
    }
}
```

### Interactive Component

```valkyrie
widget Counter {
    count: Signal⟨i32⟩ = Signal.new(0)
    
    div {
        class = "counter"
        
        button {
            on_click { count.update { $ - 1 } }
            "-"
        }
        
        span { "Count: ${count.get()}" }
        
        button {
            on_click { count.update { $ + 1 } }
            "+"
        }
    }
}
```

## Project Structure

```
my-web-app/
├── legion.json
├── src/
│   ├── main.vk           # Entry point
│   ├── app.vk            # Main application
│   ├── components/       # Reusable components
│   │   ├── header.vk
│   │   └── footer.vk
│   ├── pages/            # Page components
│   │   ├── home.vk
│   │   └── about.vk
│   └── styles/           # CSS modules
│       └── main.css
└── public/               # Static assets
    └── images/
```

## Routing

```valkyrie
using std::web::router

micro main() {
    router {
        route(path = "/", component = HomePage)
        route(path = "/about", component = AboutPage)
        route(path = "/users/:id", component = UserPage)
        route(path = "*", component = NotFoundPage)
    }
}

widget UserPage {
    id: string
    
    micro on_mount() {
        let user = fetch_user(id).await?
        # Update state with user data
    }
    
    div {
        h1 { "User: ${id}" }
        # User details
    }
}
```

## Data Fetching

```valkyrie
widget UserProfile {
    user: Signal⟨Option⟨User⟩⟩ = Signal.new(None)
    loading: Signal⟨bool⟩ = Signal.new(false)
    error: Signal⟨Option⟨string⟩⟩ = Signal.new(None)
    
    micro on_mount() {
        self.load_user()
    }
    
    async micro load_user() {
        loading.set(true)
        error.set(None)
        
        try {
            let response = fetch("/api/user").await?
            let data = response.json().await?
            user.set(Some(data))
        }
        .catch {
            case e:
                error.set(Some(e.message))
        }
        
        loading.set(false)
    }
    
    div {
        if loading.get() {
            Spinner { }
        } else if let Some(err) = error.get() {
            ErrorMessage { message: err }
        } else if let Some(u) = user.get() {
            UserCard { user: u }
        }
    }
}
```

## Styling

### CSS Modules

```valkyrie
using styles from "./button.module.css"

widget StyledButton {
    text: string
    
    button {
        class = styles.primary
        on_click { /* ... */ }
        text
    }
}
```

### Inline Styles

```valkyrie
div {
    style = {
        "background-color": "blue",
        "padding": "10px",
        "border-radius": "5px"
    }
    "Styled content"
}
```

### Dynamic Styles

```valkyrie
widget ThemeAware {
    is_dark: Signal⟨bool⟩ = Signal.new(false)
    
    div {
        class = if is_dark.get() { "dark-theme" } else { "light-theme" }
        
        button {
            on_click { is_dark.update { !$ } }
            "Toggle Theme"
        }
    }
}
```

## Chapter Contents

- **[Widget](./widget.md)**: Component system and lifecycle
- **[Events](./events.md)**: Event handling and user interaction

## Deployment

### Static Site Generation

```bash
valkyrie build --target static
```

### Server-Side Rendering

```bash
valkyrie build --target ssr
```

### Progressive Web App

```bash
valkyrie build --target pwa
```
