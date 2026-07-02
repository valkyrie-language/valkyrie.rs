/// `PE` 二进制写入器。
pub(crate) struct PeBinaryWriter {
    data: Vec<u8>,
}

impl PeBinaryWriter {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub(crate) fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub(crate) fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

pub(crate) fn align_up(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    let remainder = value % alignment;
    if remainder == 0 { value } else { value + (alignment - remainder) }
}

pub(crate) const FILE_ALIGNMENT: u32 = 0x200;
pub(crate) const SECTION_ALIGNMENT: u32 = 0x1000;
pub(crate) const IMAGE_BASE_X64: u64 = 0x0000_1400_0000;
