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
        id u64,
        name u8 : 7,
        yes bool : 1,
        email u32,
        hdr Header,
    }
    ";

    // 1. Parse the source code
    match Parser::new(source).and_then(|p| p.parse_module()) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}
