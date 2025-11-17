use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use onyx::{
    generators::{CodeGenerator, cpp::CppGenerator},
    parser::Parser,
};

#[test]
fn circular_dependency() {
    let source = "
    struct Header {
        tag User,
    }

    message User {
        hdr Header,
    }
    ";

    let parser = match Parser::new(source) {
        Ok(p) => p,
        Err(e) => {
            panic!("{e}");
        }
    };

    let result = parser.parse_module();
    assert!(result.is_err());
}

#[test]
fn undefined_type() {
    let source = "
    message User {
        hdr Header,
    }
    ";

    let parser = match Parser::new(source) {
        Ok(p) => p,
        Err(e) => {
            panic!("{e}");
        }
    };

    let result = parser.parse_module();
    assert!(result.is_err());
}

#[test]
fn compile_cpp() {
    let mut file = match File::open("tests/example.onyx") {
        Ok(file) => file,
        Err(e) => {
            panic!("{e}");
        }
    };

    let mut source = String::new();
    let _ = file.read_to_string(&mut source).inspect_err(|e| {
        panic!("{e}");
    });

    // 1. Parse the source code
    let module_ast = match Parser::new(&source).and_then(|p| p.parse_module()) {
        Ok(table) => table,
        Err(e) => {
            panic!("{e}");
        }
    };

    let mut cpp_generator = CppGenerator::default();
    let _ = cpp_generator.add_file_path(PathBuf::from("tests/data/example"));

    let files = match cpp_generator.generate(&module_ast) {
        Ok(files) => files,
        Err(e) => {
            panic!("{e}");
        }
    };

    let mut binding = Command::new("g++");
    let command: &mut Command = binding.arg("-std=c++11");
    command.arg("tests/use.cpp");

    for (file_path, content) in &files {
        let parent_dir = Path::new(&file_path).parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();
        let mut f = match File::create(file_path) {
            Ok(f) => f,
            Err(e) => {
                panic!("{e}");
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
        panic!(
            "Compilation failed with status: {:?}",
            command_status.code()
        );
    }

    let command_status = Command::new("./a.out")
        .spawn()
        .expect("Failed to execute 'g++' command")
        .wait()
        .expect("Failed to wait for 'g++' command");

    if command_status.success() {
        println!("Program run successful.");
    } else {
        panic!("Program failed with status: {:?}", command_status.code());
    }
}
