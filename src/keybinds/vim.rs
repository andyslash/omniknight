#[derive(Debug, Default)]
pub struct VimState {
    pub count: Option<usize>,
    pub pending_g: bool,
}

impl VimState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed_digit(&mut self, d: u8) {
        let digit = d as usize;
        self.count = Some(self.count.unwrap_or(0) * 10 + digit);
    }

    pub fn take_count(&mut self) -> usize {
        self.count.take().unwrap_or(1)
    }

    pub fn reset(&mut self) {
        self.count = None;
        self.pending_g = false;
    }
}
