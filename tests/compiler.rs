use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use onyx::{
    generators::{CodeGenerator, cpp::CppGenerator, rust::RustGenerator},
    parser::Parser,
};

#[test]
fn compile_cpp() {
    let mut file = File::open("tests/example.onyx").unwrap();

    let mut source = String::new();
    let _ = file.read_to_string(&mut source).unwrap();

    // 1. Parse the source code
    let module_ast = Parser::new(&source).and_then(|p| p.parse_module()).unwrap();

    let mut cpp_generator = CppGenerator::default();
    let _ = cpp_generator.add_file_path(PathBuf::from("tests/output_cpp/example"));

    let files = cpp_generator.generate(&module_ast).unwrap();

    let mut binding = Command::new("g++");
    let command: &mut Command = binding.arg("-std=c++11");
    command.arg("tests/cpp_test_main.cpp");

    for (file_path, content) in &files {
        let parent_dir = Path::new(&file_path).parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();
        let mut f = File::create(file_path).unwrap();
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

#[test]
fn compile_rust() {
    let mut file = File::open("tests/example.onyx").unwrap();

    let mut source = String::new();
    let _ = file.read_to_string(&mut source).unwrap();

    let module_ast = Parser::new(&source).and_then(|p| p.parse_module()).unwrap();

    let mut rust_generator = RustGenerator::default();
    let _ = rust_generator.add_file_path(PathBuf::from("tests/output_rust/example"));

    let files = rust_generator.generate(&module_ast).unwrap();

    for (file_path, content) in &files {
        let parent_dir = Path::new(&file_path).parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();
        let mut f = File::create(file_path).unwrap();
        let _ = f.write_all(content.as_bytes());
    }

    // Verify it compiles with rustc
    let output_path = Path::new("tests/output_rust/example.rs");
    let rustc_status = Command::new("rustc")
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2021")
        .arg(output_path)
        .arg("--out-dir")
        .arg("tests/output_rust")
        .status()
        .expect("Failed to run rustc");

    assert!(
        rustc_status.success(),
        "Generated Rust code failed to compile"
    );

    // Run runtime verification
    let runner_path = Path::new("tests/rust_test_main.rs.inc");
    let runner_status = Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("--crate-name")
        .arg("rust_test_main")
        .arg(runner_path)
        .arg("-o")
        .arg("tests/output_rust/runner")
        .status()
        .expect("Failed to compile runner");

    assert!(
        runner_status.success(),
        "Failed to compile verification runner"
    );

    let run_status = Command::new("tests/output_rust/runner")
        .status()
        .expect("Failed to run verification runner");

    assert!(run_status.success(), "Runtime verification failed");
}
