# Widget Type (Widget)

The Widget type is a special class type in Valkyrie specifically designed for UI development. It provides high-level abstractions for building modern user interfaces, supporting responsive design, state management, and event handling.

## Basic Widget Definition (Basic Widget Definition)

### Simple Widget (Simple Widget)

```valkyrie
# Basic button widget
widget Button {
    # Widget properties
    text: utf8,
    enabled: bool,
    style: ButtonStyle,
    
    # Event handlers
    on_click: Option⟨micro() -> ()⟩,
    
    # Constructor
    micro constructor(self, text: utf8) {
        self.text = text
        self.enabled = true
        self.style = ButtonStyle::default()
        self.on_click = None
    }
    
    # Render method
    micro render(self) -> Element {
        Element::button()
            .text(self.text)
            .enabled(self.enabled)
            .style(self.style)
            .on_click(self.on_click)
    }
    
    # Set click event
    micro on_click(mut self, handler: micro() -> ()) -> Self {
        self.on_click = handler
        self
    }
    
    # Set style
    micro with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style
        self
    }
}
```

### Text Input Widget (Text Input Widget)

```valkyrie
widget TextInput {
    value: utf8,
    placeholder: utf8,
    max_length: Option⟨usize⟩,
    readonly: bool,
    
    # Event handlers
    on_change: Option⟨micro(utf8) -> ()⟩,
    on_focus: Option⟨micro() -> ()⟩,
    on_blur: Option⟨micro() -> ()⟩,
    
    micro constructor(self, placeholder: utf8 = "") {
        self.value = ""
        self.placeholder = placeholder
        self.max_length = None
        self.readonly = false
        self.on_change = None
        self.on_focus = None
        self.on_blur = None
    }
    
    micro render(self) -> Element {
        Element::input()
            .value(self.value)
            .placeholder(self.placeholder)
            .max_length(self.max_length)
            .readonly(self.readonly)
            .on_change(self.on_change)
            .on_focus(self.on_focus)
            .on_blur(self.on_blur)
    }
    
    # Set value
    micro set_value(mut self, value: utf8) {
        match self.max_length {
            case max_len:
                if value.length > max_len {
                    return
                }
            case None: {}
        }
        
        self.value = value
        
        match self.on_change {
            case handler: handler(self.value.clone())
            case None: {}
        }
    }
}
```

## Layout Widgets

### Container Widget

```valkyrie
widget Container {
    children: [Box⟨Widget⟩],
    padding: Padding,
    margin: Margin,
    background: Option⟨Color⟩,
    border: Option⟨Border⟩,
    
    micro new() -> Self {
        Container {
            children: [],
            padding: Padding::zero(),
            margin: Margin::zero(),
            background: None,
            border: None,
        }
    }
    
    micro render(self) -> Element {
        let mut element = Element::div()
            .padding(self.padding)
            .margin(self.margin)
        
        if let Some(bg) = self.background {
            element = element.background(bg)
        }
        
        if let Some(border) = self.border {
            element = element.border(border)
        }
        
        for child in self.children {
            element = element.child(child.render())
        }
        
        element
    }
    
    # Add child widget
    micro add_child(mut self, child: Box⟨Widget⟩) -> Self {
        self.children.push(child)
        self
    }
    
    # Batch add child widgets
    micro add_children(mut self, children: [Box⟨Widget⟩]) -> Self {
        self.children.extend(children)
        self
    }
    
    # Set styles
    micro padding(mut self, padding: Padding) -> Self {
        self.padding = padding
        self
    }
    
    micro background(mut self, color: Color) -> Self {
        self.background = Some(color)
        self
    }
}
```

### Flex Layout

```valkyrie
widget FlexBox {
    children: [FlexChild],
    direction: FlexDirection,
    justify_content: JustifyContent,
    align_items: AlignItems,
    wrap: FlexWrap,
    gap: f32,
    
    micro new(direction: FlexDirection = FlexDirection::Row) -> Self {
        FlexBox {
            children: [],
            direction: direction,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            wrap: FlexWrap::NoWrap,
            gap: 0.0,
        }
    }
    
    micro render(self) -> Element {
        let mut element = Element::div()
            .display(Display::Flex)
            .flex_direction(self.direction)
            .justify_content(self.justify_content)
            .align_items(self.align_items)
            .flex_wrap(self.wrap)
            .gap(self.gap)
        
        for child in self.children {
            let child_element = child.widget.render()
                .flex_grow(child.grow)
                .flex_shrink(child.shrink)
                .flex_basis(child.basis)
            
            element = element.child(child_element)
        }
        
        element
    }
    
    # Add flex child
    micro add_flex_child(mut self, widget: Box⟨Widget⟩, grow: f32 = 0.0, shrink: f32 = 1.0, basis: FlexBasis = FlexBasis::Auto) -> Self {
        self.children.push(FlexChild {
            widget,
            grow,
            shrink,
            basis,
        })
        self
    }
    
    # Set layout properties
    micro justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify
        self
    }
    
    micro align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align
        self
    }
}
```

### Grid Layout

```valkyrie
widget Grid {
    children: [GridChild],
    template_columns: [GridTrack],
    template_rows: [GridTrack],
    gap: GridGap,
    
    micro new(columns: [GridTrack], rows: [GridTrack]) -> Self {
        Grid {
            children: [],
            template_columns: columns,
            template_rows: rows,
            gap: GridGap::zero(),
        }
    }
    
    micro render(self) -> Element {
        let mut element = Element::div()
            .display(Display::Grid)
            .grid_template_columns(self.template_columns)
            .grid_template_rows(self.template_rows)
            .gap(self.gap)
        
        for child in self.children {
            let child_element = child.widget.render()
                .grid_column(child.column)
                .grid_row(child.row)
            
            element = element.child(child_element)
        }
        
        element
    }
    
    # Add grid child
    micro add_grid_child(mut self, widget: Box⟨Widget⟩, column: GridPosition, row: GridPosition) -> Self {
        self.children.push(GridChild {
            widget,
            column,
            row,
        })
        self
    }
}
```

## State Management Widgets

### Stateful Widget

```valkyrie
widget Counter {
    count: i32,
    step: i32,
    min_value: Option⟨i32⟩,
    max_value: Option⟨i32⟩,
    
    # Event handler
    on_change: Option⟨micro(i32) -> ()⟩,
    
    micro new(initial_count: i32 = 0, step: i32 = 1) -> Self {
        Counter {
            count: initial_count,
            step: step,
            min_value: None,
            max_value: None,
            on_change: None,
        }
    }
    
    micro render(self) -> Element {
        FlexBox::new(FlexDirection::Row)
            .add_flex_child(
                Box::new(Button::new("-")
                    .on_click { self.decrement() }),
                0.0, 1.0, FlexBasis::Auto
            )
            .add_flex_child(
                Box::new(Text::new("{self.count}"))
                    .align(TextAlign::Center)),
                1.0, 1.0, FlexBasis::Auto
            )
            .add_flex_child(
                Box::new(Button::new("+")
                    .on_click { self.increment() }),
                0.0, 1.0, FlexBasis::Auto
            )
            .render()
    }
    
    # Increment counter
    micro increment(mut self) {
        let new_count = self.count + self.step
        
        if let Some(max) = self.max_value {
            if new_count > max {
                return
            }
        }
        
        self.count = new_count
        self.notify_change()
    }
    
    # Decrement counter
    micro decrement(mut self) {
        let new_count = self.count - self.step
        
        if let Some(min) = self.min_value {
            if new_count < min {
                return
            }
        }
        
        self.count = new_count
        self.notify_change()
    }
    
    # Notify changes
    micro notify_change(self) {
        if let Some(handler) = self.on_change {
            handler(self.count)
        }
    }
    
    # Set range
    micro range(mut self, min: i32, max: i32) -> Self {
        self.min_value = Some(min)
        self.max_value = Some(max)
        self
    }
}
```

### Form Widget

```valkyrie
widget Form⟨T⟩ {
    fields: {utf8: Box⟨Widget⟩},
    validators: {utf8: [Validator]},
    data: T,
    errors: {utf8: [utf8]},
    
    # Event handlers
    on_submit: Option⟨micro(T) -> ()⟩,
    on_validate: Option⟨micro(ValidationResult) -> ()⟩,
    
    micro new(initial_data: T) -> Self {
        Form {
            fields: {},
            validators: {},
            data: initial_data,
            errors: {},
            on_submit: None,
            on_validate: None,
        }
    }
    
    micro render(self) -> Element {
        let mut form_element = Element::form()
            .on_submit {
                $e.prevent_default()
                self.handle_submit()
            }
        
        # Render fields
        for (name, field) in self.fields {
            let field_container = Container::new()
                .add_child(field)
            
            # Add error messages
            if let Some(field_errors) = self.errors.get(name) {
                for error in field_errors {
                    field_container = field_container.add_child(
                        Box::new(Text::new(error)
                            .color(Color::Red)
                            .size(TextSize::Small))
                    )
                }
            }
            
            form_element = form_element.child(field_container.render())
        }
        
        # Submit button
        form_element = form_element.child(
            Button::new("Submit")
                .type_(ButtonType::Submit)
                .render()
        )
        
        form_element
    }
    
    # Add field
    micro add_field(mut self, name: utf8, field: Box⟨Widget⟩) -> Self {
        self.fields.insert(name, field)
        self
    }
    
    # Add validator
    micro add_validator(mut self, field_name: utf8, validator: Validator) -> Self {
        if !self.validators.contains_key(field_name) {
            self.validators.insert(field_name.clone(), [])
        }
        self.validators[field_name].push(validator)
        self
    }
    
    # Validate form
    micro validate(mut self) -> ValidationResult {
        self.errors.clear()
        let mut is_valid = true
        
        for (field_name, validators) in self.validators {
            let field_value = self.get_field_value(field_name)
            
            for validator in validators {
                if let Err(error) = validator.validate(field_value) {
                    if !self.errors.contains_key(field_name) {
                        self.errors.insert(field_name.clone(), [])
                    }
                    self.errors[field_name].push(error)
                    is_valid = false
                }
            }
        }
        
        let result = ValidationResult { is_valid, errors: self.errors.clone() }
        
        if let Some(handler) = self.on_validate {
            handler(result.clone())
        }
        
        result
    }
    
    # Handle submit
    micro handle_submit(mut self) {
        let validation_result = self.validate()
        
        if validation_result.is_valid {
            if let Some(handler) = self.on_submit {
                handler(self.data.clone())
            }
        }
    }
}
```

## Advanced Widgets

### Virtual Scrolling List

```valkyrie
widget VirtualList⟨T⟩ {
    items: [T],
    item_height: f32,
    container_height: f32,
    scroll_top: f32,
    
    # Render function
    render_item: micro(T, usize) -> Box⟨Widget⟩,
    
    micro new(items: [T], item_height: f32, container_height: f32) -> Self {
        VirtualList {
            items: items,
            item_height: item_height,
            container_height: container_height,
            scroll_top: 0.0,
            render_item: micro(item, index) { 
                Box::new(Text::new(f"Item {index}")))
            }
        }
    }
    
    micro render(self) -> Element {
        let visible_start = (self.scroll_top / self.item_height).floor() as usize
        let visible_count = (self.container_height / self.item_height).ceil() as usize + 1
        let visible_end = (visible_start + visible_count).min(self.items.length)
        
        let mut container = Container::new()
            .height(self.container_height)
            .overflow_y(Overflow::Scroll)
            .on_scroll { $e -> self.handle_scroll($e.scroll_top) }
        
        # Top spacer
        if visible_start > 0 {
            let spacer_height = visible_start as f32 * self.item_height
            container = container.add_child(
                Box::new(Spacer::new().height(spacer_height))
            )
        }
        
        # Visible items
        for i in visible_start..visible_end {
            let item = &self.items[i]
            let item_widget = (self.render_item)(item, i)
            container = container.add_child(item_widget)
        }
        
        # Bottom spacer
        if visible_end < self.items.length {
            let spacer_height = (self.items.length - visible_end) as f32 * self.item_height
            container = container.add_child(
                Box::new(Spacer::new().height(spacer_height))
            )
        }
        
        container.render()
    }
    
    # Handle scroll
    micro handle_scroll(mut self, scroll_top: f32) {
        self.scroll_top = scroll_top
        # Trigger re-render
        self.request_update()
    }
    
    # Set item renderer
    micro item_renderer(mut self, renderer: micro(T, usize) -> Box⟨Widget⟩) -> Self {
        self.render_item = renderer
        self
    }
}
```

### Modal Dialog

```valkyrie
widget Modal {
    visible: bool,
    title: utf8,
    content: Box⟨Widget⟩,
    closable: bool,
    
    # Event handlers
    on_close: Option⟨micro() -> ()⟩,
    on_confirm: Option⟨micro() -> ()⟩,
    
    micro new(title: utf8, content: Box⟨Widget⟩) -> Self {
        Modal {
            visible: false,
            title: title,
            content: content,
            closable: true,
            on_close: None,
            on_confirm: None,
        }
    }
    
    micro render(self) -> Element {
        if !self.visible {
            return Element::empty()
        }
        
        # Overlay
        let overlay = Element::div()
            .position(Position::Fixed)
            .top(0)
            .left(0)
            .width("100%")
            .height("100%")
            .background(Color::rgba(0, 0, 0, 0.5))
            .z_index(1000)
            .on_click { 
                if self.closable {
                    self.close()
                }
            }
        
        # Dialog content
        let dialog = Container::new()
            .background(Color::White)
            .border_radius(8.0)
            .padding(Padding::all(20.0))
            .max_width(500.0)
            .position(Position::Relative)
            .on_click { $e -> $e.stop_propagation() }
        
        # Header
        let header = FlexBox::new(FlexDirection::Row)
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
            .add_flex_child(
                Box::new(Text::new(self.title)
                    .size(TextSize::Large)
                    .weight(FontWeight::Bold)),
                1.0, 1.0, FlexBasis::Auto
            )
        
        if self.closable {
            header = header.add_flex_child(
                Box::new(Button::new("×")
                    .variant(ButtonVariant::Ghost)
                    .on_click { self.close() }),
                0.0, 1.0, FlexBasis::Auto
            )
        }
        
        dialog = dialog
            .add_child(Box::new(header))
            .add_child(self.content.clone())
        
        # Centering
        let centered = FlexBox::new(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .width("100%")
            .height("100%")
            .add_flex_child(Box::new(dialog), 0.0, 1.0, FlexBasis::Auto)
        
        overlay.child(centered.render())
    }
    
    # Show dialog
    micro show(mut self) {
        self.visible = true
        self.request_update()
    }
    
    # Close dialog
    micro close(mut self) {
        self.visible = false
        
        if let Some(handler) = self.on_close {
            handler()
        }
        
        self.request_update()
    }
}
```

## Responsive Design

### Media Query Widget

```valkyrie
widget Responsive {
    breakpoints: {utf8: f32},
    current_breakpoint: utf8,
    children: {utf8: Box⟨Widget⟩},
    
    micro new() -> Self {
        Responsive {
            breakpoints: {
                "mobile": 768.0,
                "tablet": 1024.0,
                "desktop": 1200.0,
            },
            current_breakpoint: "desktop",
            children: {},
        }
        # setup_resize_listener() should be called here in practice
    }
    
    micro render(self) -> Element {
        if let Some(child) = self.children.get(self.current_breakpoint) {
            child.render()
        } else {
            Element::empty()
        }
    }
    
    # Set widget for different breakpoints
    micro for_breakpoint(mut self, breakpoint: utf8, widget: Box⟨Widget⟩) -> Self {
        self.children.insert(breakpoint, widget)
        self
    }
    
    # Set breakpoints
    micro breakpoint(mut self, name: utf8, width: f32) -> Self {
        self.breakpoints.insert(name, width)
        self
    }
    
    # Update current breakpoint
    micro update_breakpoint(mut self, window_width: f32) {
        let mut current = "mobile"
        
        for (name, width) in self.breakpoints {
            if window_width >= width {
                current = name
            }
        }
        
        if current != self.current_breakpoint {
            self.current_breakpoint = current
            self.request_update()
        }
    }
}
```

## Animation and Transitions

### Animation Widget

```valkyrie
widget Animated {
    child: Box⟨Widget⟩,
    animation: Animation,
    duration: Duration,
    easing: EasingFunction,
    
    micro new(child: Box⟨Widget⟩, animation: Animation) -> Self {
        Animated {
            child: child,
            animation: animation,
            duration: Duration::milliseconds(300),
            easing: EasingFunction::EaseInOut,
        }
    }
    
    micro render(self) -> Element {
        self.child.render()
            .animate(self.animation)
            .duration(self.duration)
            .easing(self.easing)
    }
    
    # Set animation properties
    micro duration(mut self, duration: Duration) -> Self {
        self.duration = duration
        self
    }
    
    micro easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing
        self
    }
}

# Predefined animations
enum Animation {
    FadeIn,
    FadeOut,
    SlideInLeft,
    SlideInRight,
    SlideInUp,
    SlideInDown,
    ScaleIn,
    ScaleOut,
    RotateIn,
    Bounce
}
```

## Best Practices

### 1. Widget Composition

```valkyrie
# Composite widget example
widget UserCard {
    user: User,
    show_actions: bool,
    
    micro new(user: User, show_actions: bool = true) -> Self {
        UserCard { user, show_actions }
    }
    
    micro render(self) -> Element {
        let mut card = Container::new()
            .padding(Padding::all(16.0))
            .border(Border::all(1.0, Color::Gray))
            .border_radius(8.0)
            .background(Color::White)
        
        # User avatar and info
        let user_info = FlexBox::new(FlexDirection::Row)
            .gap(12.0)
            .add_flex_child(
                Box::new(Avatar::new(self.user.avatar_url)
                    .size(48.0)),
                0.0, 1.0, FlexBasis::Auto
            )
            .add_flex_child(
                Box::new(FlexBox::new(FlexDirection::Column)
                    .add_flex_child(
                        Box::new(Text::new(self.user.name)
                            .size(TextSize::Large)
                            .weight(FontWeight::Bold)),
                        0.0, 1.0, FlexBasis::Auto
                    )
                    .add_flex_child(
                        Box::new(Text::new(self.user.email)
                            .color(Color::Gray)),
                        0.0, 1.0, FlexBasis::Auto
                    )),
                1.0, 1.0, FlexBasis::Auto
            )
        
        card = card.add_child(Box::new(user_info))
        
        # Action buttons
        if self.show_actions {
            let actions = FlexBox::new(FlexDirection::Row)
                .gap(8.0)
                .justify_content(JustifyContent::End)
                .add_flex_child(
                    Box::new(Button::new("Edit")
                        .variant(ButtonVariant::Outline)),
                    0.0, 1.0, FlexBasis::Auto
                )
                .add_flex_child(
                    Box::new(Button::new("Delete")
                        .variant(ButtonVariant::Danger)),
                    0.0, 1.0, FlexBasis::Auto
                )
            
            card = card.add_child(Box::new(actions))
        }
        
        card.render()
    }
}
```

### 2. State Management

```valkyrie
# Widget using state management
widget TodoApp {
    todos: [Todo],
    filter: TodoFilter,
    new_todo_text: utf8,
    
    micro new() -> Self {
        TodoApp {
            todos: [],
            filter: TodoFilter::All,
            new_todo_text: "",
        }
    }
    
    micro render(self) -> Element {
        Container::new()
            .padding(Padding::all(20.0))
            .add_child(Box::new(self.render_header()))
            .add_child(Box::new(self.render_todo_list()))
            .add_child(Box::new(self.render_footer()))
            .render()
    }
    
    micro render_header(self) -> impl Widget {
        FlexBox::new(FlexDirection::Row)
            .gap(10.0)
            .add_flex_child(
                Box::new(TextInput::new("Add new task...")
                    .value(self.new_todo_text)
                    .on_change { self.new_todo_text = $text }
                    .on_enter { self.add_todo() }),
                1.0, 1.0, FlexBasis::Auto
            )
            .add_flex_child(
                Box::new(Button::new("Add")
                    .on_click { self.add_todo() }),
                0.0, 1.0, FlexBasis::Auto
            )
    }
    
    micro add_todo(mut self) {
        if !self.new_todo_text.is_empty() {
            self.todos.push(Todo {
                id: generate_id(),
                text: self.new_todo_text.clone(),
                completed: false,
            })
            self.new_todo_text = ""
            self.request_update()
        }
    }
}
```

The Widget type provides Valkyrie with powerful UI development capabilities, making it simple and efficient to build modern user interfaces through a declarative component model and responsive state management.
