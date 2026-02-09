static SPINNER_CHARS: &[char] = &['|', '/', '-', '\\'];

#[derive(Default, Clone, Copy, Debug)]
pub struct Spinner {
    index: usize,
}

impl Spinner {
    pub fn draw(&self) -> char {
        SPINNER_CHARS[self.index]
    }

    pub fn update(&mut self) {
        self.index = (self.index + 1) % SPINNER_CHARS.len();
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}

