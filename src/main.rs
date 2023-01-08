use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use myexpl::*;
use std::{env, error::Error, ffi::OsStr, fs::File, io::Read, path::PathBuf};

// Using `lrlex_mod!` brings the lexer for `calc.l` into scope. By default the
// module name will be `lexer_l` (i.e. the file name, minus any extensions,
// with a suffix of `_l`).
lrlex_mod!("lexer.l");
// Using `lrpar_mod!` brings the parser for `calc.y` into scope. By default the
// module name will be `parser_y` (i.e. the file name, minus any extensions,
// with a suffix of `_y`).
lrpar_mod!("parser.y");

fn get_input(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let mut fd = File::open(path)?;
    let mut s = String::new();
    fd.read_to_string(&mut s)?;
    Ok(s)
}

fn main() {
    let mut file_name = match env::args().nth(1) {
        Some(arg) => PathBuf::from(arg),
        None => PathBuf::from("./test_progs/prg.expl".to_owned()),
    };
    match file_name.extension().and_then(OsStr::to_str) {
        Some("expl") => {}
        _ => {
            eprintln!("expl file wasn\'t provided!");
            std::process::exit(1);
        }
    }
    let input = match get_input(&file_name) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let lexerdef = lexer_l::lexerdef();
    let lexer = lexerdef.lexer(&input);
    // Pass the lexer to the parser and lex and parse the input.
    let (res, errs) = parser_y::parse(&lexer);
    if errs.is_empty() == false {
        for e in errs {
            println!("{}", e.pp(&lexer, &parser_y::token_epp));
        }
        eprintln!("Unable to evaluate expression!");
        std::process::exit(1);
    }
    file_name.set_extension("xsm");
    match res {
        Some(Ok(root)) => match generate_code(root, &lexer, &file_name) {
            Ok(_) => println!("Comipled successfully"),
            Err(e) => eprintln!("{e}"),
        },
        _ => eprintln!("Error in code generation phase"),
    }
}
