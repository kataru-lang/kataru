pub trait ReversedClone {
    fn reversed(&self) -> Self;
}

impl<T: Clone> ReversedClone for Vec<T> {
    /// Creates a clone of `vector` reversed.
    fn reversed(&self) -> Vec<T> {
        let mut result: Vec<T> = Vec::new();
        result.reserve(self.len());
        for element in self.iter().rev() {
            result.push((*element).clone());
        }
        result
    }
}
