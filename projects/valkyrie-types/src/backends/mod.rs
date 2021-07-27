mod wasm;

pub(crate) trait ConvertTo<Item> {
    fn convert(&self) -> Item;
}
