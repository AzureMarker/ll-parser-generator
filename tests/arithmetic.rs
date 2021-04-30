use logos::{Lexer, Logos};

#[derive(Logos, Debug, Clone, Eq, PartialEq)]
pub enum Token {
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
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
    Calculated {
        operation: AstOperation,
        left: Box<AstTerm>,
        right: Box<AstTerm>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum AstOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

mod parser {
    #![allow(clippy::all)]
    ll_parser_generator::ll_parser! {
        use crate::{Token, AstTerm, AstOperation};

        token Token {
            "NUMBER" = Token::Number,
            "(" = Token::LParen,
            ")" = Token::RParen,
            "*" = Token::Mul,
            "/" = Token::Div,
            "-" = Token::Sub,
            "+" = Token::Add,
        }

        grammar;

        pub Term0: AstTerm = {
            <left:Term1> <part2:Term0Part2> => match part2 {
                Some((operation, right)) => AstTerm::Calculated {
                    operation,
                    left: Box::new(left),
                    right: Box::new(right)
                },
                None => left
            },
        };

        Term0Part2: Option<(AstOperation, AstTerm)> = {
            <operation:Operation0> <right:Term0> => Some((operation, right)),
            => None,
        };

        Operation0: AstOperation = {
            "+" => AstOperation::Add,
            "-" => AstOperation::Subtract,
        };

        Term1: AstTerm = {
            <left:Term2> <part2:Term1Part2> => match part2 {
                Some((operation, right)) => AstTerm::Calculated {
                    operation,
                    left: Box::new(left),
                    right: Box::new(right)
                },
                None => left
            },
        };

        Term1Part2: Option<(AstOperation, AstTerm)> = {
            <operation:Operation1> <right:Term1> => Some((operation, right)),
            => None,
        };

        Operation1: AstOperation = {
            "*" => AstOperation::Multiply,
            "/" => AstOperation::Divide,
        };

        Term2: AstTerm = {
            "NUMBER" => AstTerm::Number,
            "(" <inner:Term0> ")" => inner,
        };
    }
}

#[test]
fn number() {
    let lexer = Lexer::new("1");
    let result = parser::parse(lexer);

    assert_eq!(result, Ok(AstTerm::Number));
}

#[test]
fn add() {
    let lexer = Lexer::new("1 + 1");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Ok(AstTerm::Calculated {
            operation: AstOperation::Add,
            left: Box::new(AstTerm::Number),
            right: Box::new(AstTerm::Number)
        })
    )
}

#[test]
fn basic_associativity() {
    let lexer = Lexer::new("1 + 2 * 3");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Ok(AstTerm::Calculated {
            operation: AstOperation::Add,
            left: Box::new(AstTerm::Number),
            right: Box::new(AstTerm::Calculated {
                operation: AstOperation::Multiply,
                left: Box::new(AstTerm::Number),
                right: Box::new(AstTerm::Number)
            })
        })
    );
}

#[test]
fn advanced_associativity() {
    let lexer = Lexer::new("(1 - 2) / (1 * 1)");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Ok(AstTerm::Calculated {
            operation: AstOperation::Divide,
            left: Box::new(AstTerm::Calculated {
                operation: AstOperation::Subtract,
                left: Box::new(AstTerm::Number),
                right: Box::new(AstTerm::Number)
            }),
            right: Box::new(AstTerm::Calculated {
                operation: AstOperation::Multiply,
                left: Box::new(AstTerm::Number),
                right: Box::new(AstTerm::Number)
            })
        })
    );
}

#[test]
fn missing_operand() {
    let lexer = Lexer::new("1 + ");
    let result = parser::parse(lexer);

    assert_eq!(result, Err(parser::ParseError::UnexpectedEOF));
}

#[test]
fn missing_operation() {
    let lexer = Lexer::new("1 2");
    let result = parser::parse(lexer);

    assert_eq!(
        result,
        Err(parser::ParseError::UnrecognizedToken {
            expected: vec!["\"*\"", "\"/\""],
            found: Token::Number
        })
    )
}
