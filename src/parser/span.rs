// Structs definning spans and objects with an associated span.
#[derive(Debug, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span<'_>) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Spanned<T: PartialEq> {
    pub inner: T,
    pub span: Span,
}
