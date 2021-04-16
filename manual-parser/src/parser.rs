use crate::ast::AstTerm;
use crate::lexer::Token;

/*
use crate::Token;

token Token {
    "(" = Token::LParen,
    ")" = Token::RParen,
    "NUMBER" = Token::Number,
}

grammar;

pub Term: AstTerm = {
    <n:Number> => n,
    "(" <t:Term> ")" => AstTerm::Parens(Box::new(t)),
};

Number: AstTerm = "NUMBER" => AstTerm::Number;
*/

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnrecognizedToken {
        expected: Vec<&'static str>,
        found: Token,
    },
}

enum Symbol {
    Token1, // (
    Token2, // )
    Token3, // NUMBER
    Term,
    Number,
}

impl Symbol {
    fn is_terminal(&self) -> bool {
        match self {
            Symbol::Token1 => true,
            Symbol::Token2 => true,
            Symbol::Token3 => true,
            _ => false,
        }
    }

    fn is_nonterminal(&self) -> bool {
        !self.is_terminal()
    }

    fn name(&self) -> &'static str {
        match self {
            Symbol::Token1 => "(",
            Symbol::Token2 => ")",
            Symbol::Token3 => "NUMBER",
            Symbol::Term => "Term",
            Symbol::Number => "Number",
        }
    }
}

impl PartialEq<Token> for Symbol {
    fn eq(&self, other: &Token) -> bool {
        match self {
            Symbol::Token1 => {
                matches!(other, Token::LParen)
            }
            Symbol::Token2 => {
                matches!(other, Token::RParen)
            }
            Symbol::Token3 => {
                matches!(other, Token::Number)
            }
            _ => false,
        }
    }
}

impl PartialEq<Symbol> for Token {
    fn eq(&self, other: &Symbol) -> bool {
        other.eq(self)
    }
}

pub fn parse(lexer: impl Iterator<Item = Token>) -> Result<(), ParseError> {
    let mut lexer = lexer.peekable();
    let mut stack = vec![Symbol::Term];

    while let Some(symbol) = stack.pop() {
        if symbol.is_terminal() {
            let token = lexer.next().ok_or(ParseError::UnexpectedEOF)?;

            if symbol == token {
                continue;
            } else {
                return Err(ParseError::UnrecognizedToken {
                    expected: vec![symbol.name()],
                    found: token,
                });
            }
        } else {
            let next_token = lexer.peek().ok_or(ParseError::UnexpectedEOF)?;

            match (symbol, next_token) {
                (Symbol::Term, Token::LParen) => {
                    stack.push(Symbol::Token2);
                    stack.push(Symbol::Term);
                    stack.push(Symbol::Token1);
                }
                (Symbol::Term, Token::Number) => {
                    stack.push(Symbol::Number);
                }
                (Symbol::Number, Token::Number) => {
                    stack.push(Symbol::Token3);
                }
                (symbol, _) => {
                    return Err(ParseError::UnrecognizedToken {
                        expected: match symbol {
                            Symbol::Term => {
                                // First of Term
                                vec!["\"NUMBER\"", "\"(\""]
                            }
                            Symbol::Number => {
                                // First of Number
                                vec!["\"NUMBER\""]
                            }
                            _ => unreachable!(),
                        },
                        found: lexer.next().unwrap(),
                    });
                }
            }
        }
    }

    // unimplemented!()
    Ok(())
}
