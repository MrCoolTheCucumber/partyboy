pub(super) struct Register {
    pub hi: u8,
    pub lo: u8,
}

impl Register {
    pub fn new(hi: u8, lo: u8) -> Self {
        Self { hi, lo }
    }
}
