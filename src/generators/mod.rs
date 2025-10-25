use std::{error::Error, fmt};

use crate::ast::OnyxModule;

pub mod cpp;

pub trait CodeGenerator {
    /// Translates the AST module into a final, runnable code string.
    fn generate(&mut self, module: &OnyxModule) -> Result<Vec<(String, String)>, CompileError>;
}

// Minimal error type
#[derive(Debug)]
pub struct CompileError(pub String);
impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Compile Error: {}", self.0)
    }
}
impl Error for CompileError {}
