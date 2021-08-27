use std::ops::Range;
use yggdrasil_rt::YggdrasilNode;

pub trait YggdrasilNodeExtension<'i>: YggdrasilNode<'i>
where
    Self: Sized,
{
    fn get_range32(&self) -> Range<u32> {
        let Range { start, end } = self.get_range();
        Range { start: start as u32, end: end as u32 }
    }
}

impl<'i, T: YggdrasilNode<'i>> YggdrasilNodeExtension<'i> for T {}
