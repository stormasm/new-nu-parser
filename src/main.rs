use std::process::exit;

use new_nu_parser::compiler::Compiler;
use new_nu_parser::ir_generator::IrGenerator;
use new_nu_parser::lexer::lex;
use new_nu_parser::parser::Parser;
use new_nu_parser::resolver::Resolver;
use new_nu_parser::typechecker::Typechecker;

fn main() {
    let mut compiler = Compiler::new();
    let mut do_print = true;

    for arg in std::env::args().skip(1) {
        if arg == "--no-print" {
            do_print = false;
        }
    }

    for fname in std::env::args().skip(1) {
        if fname == "--no-print" {
            continue;
        }

        let contents = std::fs::read(&fname);

        let Ok(contents) = contents else {
            eprintln!("can't find {}", fname);
            exit(1);
        };

        let span_offset = compiler.span_offset();
        compiler.add_file(&fname, &contents);

        let (tokens, err) = lex(&contents, span_offset);
        if let Err(e) = err {
            tokens.print(&compiler.source);
            eprintln!(
                "Lexing error. Error: {:?}, '{}'",
                e,
                String::from_utf8_lossy(
                    compiler.get_span_contents_manual(e.span.start, e.span.end)
                )
            );
            exit(1);
        }

        if do_print {
            tokens.print(&compiler.source);
        }

        let parser = Parser::new(compiler, tokens);

        compiler = parser.parse();

        if do_print {
            compiler.print();
        }

        if !compiler.errors.is_empty() {
            exit(1);
        }

        let mut resolver = Resolver::new(&compiler);
        resolver.resolve();

        if do_print {
            resolver.print();
        }

        compiler.merge_name_bindings(resolver.to_name_bindings());

        if !compiler.errors.is_empty() {
            exit(1);
        }

        let mut typechecker = Typechecker::new(&compiler);
        typechecker.typecheck();

        if do_print {
            typechecker.print();
        }

        let mut ir_generator = IrGenerator::new(&compiler);
        ir_generator.generate();
        if do_print {
            ir_generator.print();
        }

        compiler.merge_types(typechecker.to_types());
    }
}
