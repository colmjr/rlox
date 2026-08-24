use crate::error::LoxError;
pub struct Scanner {}
#[derive(Debug)]
pub struct Token {}

impl Scanner {
    pub fn new(source: &str) -> Self {
        todo!()
    }

    // The reason for the following function signature is that we would like the currently
    // parsed tokens, and this may be useful to see where it errored instead of a Result<>
    pub fn scan_tokens(&mut self) -> (Vec<Token>, Vec<LoxError>) {
        todo!()
    }
}
