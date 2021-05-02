use super::Range;

struct TerminalPlotter {
    width: usize,
    range: Range,
}

impl TerminalPlotter {
    pub fn new(width: usize, range: Range) -> Self {
        TerminalPlotter { width, range }
    }

    pub fn update(&mut self, x: f64) {
        todo!()
    }
}
