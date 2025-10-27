use onyx::{
    generators::{CodeGenerator, cpp::CppGenerator},
    parser::Parser,
};

use onyx::{resolver::resolve_module};

// This part would be in your main execution logic:
pub fn main() {
    // Assume the Lexer and Parser setup from the previous step is here.
    // Re-use the example source code:

    let source = "
    enum Status : u8 {
        Active = 1,
        Inactive,
        Error = 10,
    }

    struct Header {
        version u32 : 4,
        checksum u16,
        tag Status,
    }

    message User {
        id u64,
        name u8 : 7,
        yes bool : 1,
        email u32,
        hdr Header,
    }
";

    // 1. Parse the source code
    let module_ast = Parser::new(source)
        .and_then(|p| p.parse_module())
        .expect("Parsing failed, cannot generate code.");

    let size_table = match resolve_module(&module_ast) {
        Ok(table) => table,
        Err(e) => {
            eprintln!("Parsing Failed: {e}");
            return;
        }
    };

    let mut cpp_generator = CppGenerator::new();
    cpp_generator.file_stem = "my_file".to_string();

    match cpp_generator.generate(&module_ast, &size_table) {
        Ok(files) => {
            for (filename, content) in files {
                println!("\n=============================================");
                println!("--- GENERATED FILE: {filename} ---");
                println!("=============================================");
                println!("{content}");
            }
        }
        Err(e) => {
            eprintln!("Code Generation Failed: {e}");
        }
    }
}
