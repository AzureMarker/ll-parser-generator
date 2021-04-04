use crate::ast::{AstGrammar, AstTokenDecl};
use crate::lexer::{StatefulLexer, Token};
use crate::parser::GrammarParser;
use logos::Logos;

macro_rules! grammar_test {
    (grammar { $($grammar:tt)* }, $expected_ast:expr) => {{
        let grammar = stringify!($($grammar)*);

        let lexer = StatefulLexer::new(Token::lexer(&grammar));
        let actual_ast = GrammarParser::new()
            .parse(lexer)
            .expect("Grammar should parse");

        assert_eq!(actual_ast,$expected_ast);
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
