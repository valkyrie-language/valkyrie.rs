//! Hermes schema / query AST (minimal Atlas vertical slice).

use std::fmt::{Display, Formatter};

/// Top-level Hermes document (`.hermes` / `.her`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HermesDocument {
    /// Optional `namespace name;`
    pub namespace: Option<String>,
    /// Declarations in source order.
    pub items: Vec<HermesItem>,
}

/// Top-level item inside a Hermes document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HermesItem {
    /// `storage Name { model … }`
    Storage(StorageDecl),
    /// Standalone `model Name { … }` (implicit default storage).
    Model(ModelDecl),
    /// Atlas-style `select … from …`
    Query(SelectQuery),
}

/// `storage Name { models… }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDecl {
    /// Storage name.
    pub name: String,
    /// Contained models.
    pub models: Vec<ModelDecl>,
}

/// `model Name { fields… }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDecl {
    /// Model / table name.
    pub name: String,
    /// Fields in declaration order.
    pub fields: Vec<FieldDecl>,
}

/// Field key kind (`@@` primary, `@` unique, plain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKeyKind {
    /// Ordinary column.
    Plain,
    /// `@name` — unique key.
    Unique,
    /// `@@name` — primary key.
    Primary,
}

/// `@@id: i64` / `@email: utf8` / `name: utf8`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
    /// Field name (without `@` markers).
    pub name: String,
    /// Type name text (`i64`, `utf8`, `uuid`, …).
    pub ty: String,
    /// Key kind.
    pub key: FieldKeyKind,
}

/// `select cols from Model [where col = lit] [limit n]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectQuery {
    /// Projection; empty means `*`.
    pub columns: Vec<String>,
    /// Source model / table name.
    pub from: String,
    /// Optional equality filter.
    pub where_eq: Option<(String, Literal)>,
    /// Optional limit.
    pub limit: Option<u64>,
}

/// Literal used in query filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// Integer literal.
    Integer(i64),
    /// String literal.
    String(String),
    /// Boolean literal.
    Bool(bool),
    /// Named parameter (`$id` / `@id`).
    Param(String),
}

impl Display for HermesDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ns) = &self.namespace {
            writeln!(f, "namespace {ns};")?;
            writeln!(f)?;
        }
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{item}")?;
        }
        Ok(())
    }
}

impl Display for HermesItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(s) => write!(f, "{s}"),
            Self::Model(m) => write!(f, "{m}"),
            Self::Query(q) => write!(f, "{q}"),
        }
    }
}

impl Display for StorageDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "storage {} {{", self.name)?;
        for model in &self.models {
            for line in format!("{model}").lines() {
                writeln!(f, "    {line}")?;
            }
        }
        write!(f, "}}")
    }
}

impl Display for ModelDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "model {} {{", self.name)?;
        for field in &self.fields {
            writeln!(f, "    {field},")?;
        }
        write!(f, "}}")
    }
}

impl Display for FieldDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.key {
            FieldKeyKind::Plain => "",
            FieldKeyKind::Unique => "@",
            FieldKeyKind::Primary => "@@",
        };
        write!(f, "{prefix}{}: {}", self.name, self.ty)
    }
}

impl Display for SelectQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "select ")?;
        if self.columns.is_empty() {
            write!(f, "*")?;
        }
        else {
            write!(f, "{}", self.columns.join(", "))?;
        }
        write!(f, " from {}", self.from)?;
        if let Some((col, lit)) = &self.where_eq {
            write!(f, " where {col} = {lit}")?;
        }
        if let Some(n) = self.limit {
            write!(f, " limit {n}")?;
        }
        Ok(())
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Param(p) => write!(f, "{p}"),
        }
    }
}

impl HermesDocument {
    /// Collect all models (standalone + nested in storage).
    pub fn models(&self) -> Vec<&ModelDecl> {
        let mut out = Vec::new();
        for item in &self.items {
            match item {
                HermesItem::Model(m) => out.push(m),
                HermesItem::Storage(s) => out.extend(s.models.iter()),
                HermesItem::Query(_) => {}
            }
        }
        out
    }

    /// Collect select queries.
    pub fn queries(&self) -> Vec<&SelectQuery> {
        self.items
            .iter()
            .filter_map(|item| match item {
                HermesItem::Query(q) => Some(q),
                _ => None,
            })
            .collect()
    }
}
