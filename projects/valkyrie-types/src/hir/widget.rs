//! Widget, widget lifecycle, and singleton definitions for HIR.

use super::{HirDocumentation, HirExpr, HirField, HirFunction, HirGeneric, HirParent, HirProperty, HirVisibility};
use crate::{Identifier, NamePath};

/// A widget in HIR.
///
/// Widgets are UI components that manage their own state and render
/// themselves to virtual DOM elements. Each widget must implement
/// a `render` method that returns an `Element`.
///
/// # State Management
///
/// Widget fields can be marked as state fields using the `@state` attribute
/// or by naming convention (prefix with `_` or `state_`). State fields
/// trigger re-rendering when modified.
///
/// # Lifecycle
///
/// Widgets support lifecycle methods:
/// - `on_mount`: Called when the widget is first created
/// - `on_unmount`: Called when the widget is destroyed
/// - `on_update`: Called when state changes
///
/// # Example
///
/// ```v
/// widget Counter {
///     _count: i32 = 0
///
///     fn render(self) -> Element {
///         Element("div")
///             .text(self._count.to_string())
///             .on("click", self.increment)
///     }
///
///     fn increment(mut self) {
///         self._count = self._count + 1
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirWidget {
    /// The name of the widget.
    pub name: Identifier,
    /// Documentation for the widget.
    pub doc: HirDocumentation,
    /// Generic parameters for the widget.
    pub generics: Vec<HirGeneric>,
    /// Fields of the widget (both state and non-state fields).
    pub fields: Vec<HirField>,
    /// Methods defined on the widget.
    pub methods: Vec<HirFunction>,
    /// Visibility of the widget.
    pub visibility: HirVisibility,
    /// Names of fields that are state fields.
    ///
    /// State fields trigger re-rendering when modified.
    /// This list is populated during semantic analysis.
    pub state_fields: Vec<Identifier>,
    /// Initial values for state fields.
    ///
    /// Maps field names to their default expressions.
    pub initial_state: Vec<(Identifier, HirExpr)>,
    /// Lifecycle hooks configuration.
    pub lifecycle: HirWidgetLifecycle,
}

/// Widget lifecycle configuration.
///
/// Defines the lifecycle methods that should be called
/// at various stages of the widget's existence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirWidgetLifecycle {
    /// Whether the widget has an `on_mount` lifecycle method.
    pub has_on_mount: bool,
    /// Whether the widget has an `on_unmount` lifecycle method.
    pub has_on_unmount: bool,
    /// Whether the widget has an `on_update` lifecycle method.
    pub has_on_update: bool,
    /// Whether the widget has a `before_update` lifecycle method.
    pub has_before_update: bool,
    /// Whether the widget has an `after_update` lifecycle method.
    pub has_after_update: bool,
}

/// A singleton in HIR.
///
/// Singletons are classes that have exactly one global instance.
/// They are useful for managing global state, configuration, or resources.
///
/// # Semantics
///
/// - A singleton has exactly one global instance
/// - The instance is lazily initialized on first access
/// - Singleton members are accessed through the singleton name directly
/// - Singletons cannot be instantiated with constructors
///
/// # Example
///
/// ```v
/// singleton GlobalConfig {
///     host: String = "localhost"
///     port: i32 = 8080
///
///     micro get_url(self) -> String {
///         f"{self.host}:{self.port}"
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirSingleton {
    /// The name of the singleton.
    pub name: Identifier,
    /// Documentation for the singleton.
    pub doc: HirDocumentation,
    /// Generic parameters for the singleton.
    pub generics: Vec<HirGeneric>,
    /// Parent traits this singleton implements.
    pub parents: Vec<HirParent>,
    /// Fields of the singleton.
    pub fields: Vec<HirField>,
    /// Methods defined on the singleton.
    pub methods: Vec<HirFunction>,
    /// Properties (computed fields with getter/setter).
    pub properties: Vec<HirProperty>,
    /// Visibility of the singleton.
    pub visibility: HirVisibility,
    /// Traits to derive via the derive macro system.
    pub derives: Vec<NamePath>,
    /// The unique instance variable name for code generation.
    ///
    /// This is typically `INSTANCE` or similar, used to store
    /// the lazily-initialized singleton instance.
    pub instance_name: Identifier,
}
