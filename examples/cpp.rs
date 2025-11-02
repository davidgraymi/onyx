use std::path::PathBuf;
use std::process::Command;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use onyx::{
    generators::{CodeGenerator, cpp::CppGenerator},
    parser::Parser,
};

// This part would be in your main execution logic:
pub fn main() {
    // Assume the Lexer and Parser setup from the previous step is here.
    // Re-use the example source code:

    let source = "
    endian = little

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
    let module_ast = match Parser::new(source).and_then(|p| p.parse_module()) {
        Ok(table) => table,
        Err(e) => {
            eprintln!("Parsing Failed: {e}");
            return;
        }
    };

    let mut cpp_generator = CppGenerator::default();
    let _ = cpp_generator.add_file_path(PathBuf::from("examples/data/my_file"));

    let files = match cpp_generator.generate(&module_ast) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Code Generation Failed: {e}");
            return;
        }
    };

    let mut binding = Command::new("g++");
    let command = binding.arg("-c").arg("-std=c++11");

    for (file_path, content) in &files {
        let parent_dir = Path::new(&file_path).parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();
        let mut f = match File::create(file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        let _ = f.write_all(content.as_bytes());
        command.arg(file_path);
    }

    let command_status = command
        .spawn()
        .expect("Failed to execute 'g++' command")
        .wait()
        .expect("Failed to wait for 'g++' command");

    if command_status.success() {
        println!("Compilation successful.");
    } else {
        eprintln!(
            "Compilation failed with status: {:?}",
            command_status.code()
        );
    }
}
