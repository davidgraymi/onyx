use std::{collections::HashMap, error::Error, fmt};

use crate::ast::OnyxModule;

pub mod cpp;

pub trait CodeGenerator {
    /// Translates the AST module into a final, runnable code string.
    fn generate(&mut self, module: &OnyxModule,  packed_sizes: &HashMap<String, usize>) -> Result<Vec<(String, String)>, CompileError>;
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
