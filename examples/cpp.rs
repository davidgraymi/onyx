use onyx::{
    generators::{CodeGenerator, cpp::CppGenerator},
    parser::Parser,
};

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
        name u8 : 8,
        email u32,
    }
";

    // 1. Parse the source code
    let module_ast = Parser::new(source)
        .and_then(|p| p.parse_module())
        .expect("Parsing failed, cannot generate code.");

    let mut cpp_generator = CppGenerator::new();

    match cpp_generator.generate(&module_ast) {
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
