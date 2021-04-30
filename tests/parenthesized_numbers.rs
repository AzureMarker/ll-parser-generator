use logos::{Lexer, Logos};

#[derive(Logos, Debug, Clone, Eq, PartialEq)]
pub enum Token {
    #[token(")")]
    RParen,
    #[token("(")]
    LParen,
    #[regex("[0-9]+")]
    Number,
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AstTerm {
    Number,
    Paren(Box<AstTerm>),
}

mod parser {
    #![allow(clippy::all)]
    ll_parser_generator::ll_parser! {
        use crate::Token;
        use crate::AstTerm;

        token Token {
            "(" = Token::LParen,
            ")" = Token::RParen,
            "NUMBER" = Token::Number,
        }

        grammar;

        pub Term: AstTerm = {
            <n:Number> => n,
            "(" <t:Term> ")" => AstTerm::Paren(Box::new(t)),
        };

        Number: AstTerm = "NUMBER" => AstTerm::Number;
    }
}

#[test]
fn simple_input() {
    let lexer = Lexer::new("(1)");
    let result = parser::parse(lexer);

    assert_eq!(result, Ok(AstTerm::Paren(Box::new(AstTerm::Number))));
}

#[test]
fn just_one_number() {
    let lexer = Lexer::new("1");
    let result = parser::parse(lexer);

    assert_eq!(result, Ok(AstTerm::Number));
}

#[test]
fn many_parens() {
    let lexer = Lexer::new("(((((1)))))");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Ok(AstTerm::Paren(Box::new(AstTerm::Paren(Box::new(
            AstTerm::Paren(Box::new(AstTerm::Paren(Box::new(AstTerm::Paren(
                Box::new(AstTerm::Number)
            )))))
        )))))
    );
}

#[test]
fn unmatched_parens() {
    let lexer = Lexer::new("((1)");
    let result = parser::parse(lexer);

    assert_eq!(result, Err(parser::ParseError::UnexpectedEOF));
}

#[test]
fn extra_paren() {
    let lexer = Lexer::new("(1))");
    let result = parser::parse(lexer);

    assert_eq!(result, Err(parser::ParseError::ExtraToken(Token::RParen)));
}

#[test]
fn extra_number() {
    let lexer = Lexer::new("(1 1)");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Err(parser::ParseError::UnrecognizedToken {
            expected: vec![")"],
            found: Token::Number
        })
    );
}

#[test]
fn missing_number() {
    let lexer = Lexer::new("()");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Err(parser::ParseError::UnrecognizedToken {
            expected: vec!["\"(\"", "\"NUMBER\""],
            found: Token::RParen
        })
    )
}
