#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<usize> for Span {
    fn from(value: usize) -> Self {
        Self {
            start: value,
            end: value + 1,
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((a, b): (usize, usize)) -> Self {
        Self {
            start: a,
            end: b + 1,
        }
    }
}
