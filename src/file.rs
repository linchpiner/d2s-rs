#[repr(usize)]
pub enum FileOffset {
    Header = 0,
    Version = 4,
    Size = 8,
    Checksum = 12,
    Class = 40,
    Level = 43,
    CharacterStats = 767,
}

