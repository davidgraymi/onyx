use onyx::parser::Parser;

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
            assert!(false, "{e}");
            return;
        }
    };

    match parser.parse_module() {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
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
            assert!(false, "{e}");
            return;
        }
    };

    match parser.parse_module() {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}
