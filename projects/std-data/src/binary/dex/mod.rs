//! Android `DEX` 二进制格式（dex/035）。

mod merge;
mod reader;
mod writer;

pub use merge::{merge_class_sets, merge_dex_images, merge_dex_images_with_tail};
pub use reader::{dex_class_defs_count, dex_class_descriptors, dex_contains_strings, dex_core_bytes, dex_string_ids};
pub use writer::DexImageBuilder;
