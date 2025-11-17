use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::{
    ast::OnyxModule,
    generators::{CodeGenerator, CompileError},
};

/// Configuration settings specific to Python code generation
#[derive(Debug, Clone)]
pub struct PythonConfig {
    /// Number of spaces to use for each indentation level.
    pub indent_spaces: u8,
    /// Whether to use single quotes ('') or double quotes ("").
    pub use_double_quotes: bool,
    /// Maximum line length for wrapping.
    pub max_line_length: usize,
}

impl Default for PythonConfig {
    fn default() -> Self {
        PythonConfig {
            indent_spaces: 4,
            use_double_quotes: true,
            max_line_length: 88,
        }
    }
}

pub struct PythonGenerator<W: Write> {
    /// The configuration settings for the generated Python code.
    config: PythonConfig,
    /// The underlying writer object where the code is streamed.
    writer: W,
    /// The current indentation level (number of tabs/spaces to prefix the line with).
    current_indent_level: u8,
}

impl<W: Write> PythonGenerator<W> {
    /// Creates a new `PythonGenerator` instance.
    ///
    /// # Arguments
    ///
    /// * `writer` - The output sink where the generated code will be written.
    /// * `config` - Optional configuration; uses default if None is provided.
    pub fn new(writer: W, config: Option<PythonConfig>) -> Self {
        PythonGenerator {
            config: config.unwrap_or_default(),
            writer,
            current_indent_level: 0,
        }
    }

    /// Helper function to write a line with the appropriate indentation.
    fn write_line(&mut self, content: &str) -> io::Result<()> {
        let indent = " ".repeat((self.current_indent_level * self.config.indent_spaces) as usize);
        writeln!(&mut self.writer, "{indent}{content}")
    }

    /// Increases the current indentation level.
    pub fn increase_indent(&mut self) {
        self.current_indent_level += 1;
    }

    /// Decreases the current indentation level, preventing it from going below zero.
    pub fn decrease_indent(&mut self) {
        if self.current_indent_level > 0 {
            self.current_indent_level -= 1;
        }
    }

    /// Generates a simple Python class definition.
    pub fn generate_class(&mut self, name: &str, bases: &[&str]) -> io::Result<()> {
        let base_list = if bases.is_empty() {
            "".to_string()
        } else {
            format!("({})", bases.join(", "))
        };

        self.write_line(&format!("class {name}{base_list}:"))?;
        self.increase_indent();

        // Add a docstring
        let quote = if self.config.use_double_quotes {
            "\"\"\""
        } else {
            "'''"
        };
        self.write_line(&format!(
            "{}A generated Python class.{}{}",
            quote, quote, "\n"
        ))?;

        // Add an initializer
        self.write_line("def __init__(self, value):")?;
        self.increase_indent();
        self.write_line("self.value = value")?;
        self.decrease_indent();

        // Add a placeholder method
        self.write_line("\ndef get_value(self):")?;
        self.increase_indent();
        self.write_line("return self.value")?;
        self.decrease_indent();

        self.decrease_indent(); // Return to top level indentation
        Ok(())
    }
}

impl<W: Write> CodeGenerator for PythonGenerator<W> {
    fn generate(&mut self, _module: &OnyxModule) -> Result<Vec<(PathBuf, String)>, CompileError> {
        // Here, the generator would typically call its own methods (like generate_class)
        // to write code to self.writer, and then return the collected files.
        // For now, we return a successful empty list of files.
        Ok(vec![])
    }
}
