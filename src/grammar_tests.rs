use crate::ast::{AstGrammar, AstTokenDecl};

macro_rules! parse_grammar {
    ($($grammar:tt)*) => {{
        let grammar = stringify!($($grammar)*);

        let lexer = crate::lexer::StatefulLexer::new(
            <crate::lexer::Token as logos::Logos>::lexer(&grammar)
        );
        crate::parser::GrammarParser::new()
            .parse(lexer)
            .expect("Grammar should parse")
    }};
}

macro_rules! grammar_test {
    (grammar { $($grammar:tt)* }, $expected_ast:expr) => {{
        let actual_ast = parse_grammar! { $($grammar)* };
        assert_eq!(actual_ast, $expected_ast);
    }};
}

#[test]
fn empty_grammar() {
    grammar_test! {
        grammar {
            token Token {}
            grammar;
        },
        AstGrammar {
            imports: Vec::new(),
            token_decl: AstTokenDecl {
                name: "Token",
                aliases: Vec::new()
            },
            nonterminals: Vec::new()
        }
    };
}
