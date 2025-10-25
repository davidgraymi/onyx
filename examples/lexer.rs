use onyx::parser::Parser;

// This part would be in your main execution logic:
pub fn main() {
    // Sample Onyx IDL input
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

    match Parser::new(source).and_then(|p| p.parse_module()) {
        Ok(module) => {
            println!("Successfully Parsed Onyx Module!\n");
            // Print the AST to verify structure
            for def in module.definitions {
                println!("{:#?}", def);
                println!("---");
            }
        }
        Err(e) => {
            eprintln!("Parsing Failed: {}", e);
        }
    }
}
