/*
use crate::lexer::Token;
use crate::ast::AstTerm;

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

// User imports
use crate::ast::AstTerm;
use crate::lexer::Token;

enum SymbolOrReduction {
    Symbol(Symbol),
    Reduction(fn(&mut Vec<ActionResult>)),
}

enum ActionResult {
    Nonterm0(AstTerm),
    Nonterm1(AstTerm),
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnrecognizedToken {
        expected: Vec<&'static str>,
        found: Token,
    },
}

enum Symbol {
    Term0, // (
    Term1, // )
    Term2, // NUMBER
    Nonterm0,
    Nonterm1,
}

impl Symbol {
    fn is_terminal(&self) -> bool {
        matches!(self, Symbol::Term0 | Symbol::Term1 | Symbol::Term2)
    }

    fn name(&self) -> &'static str {
        match self {
            Symbol::Term0 => "(",
            Symbol::Term1 => ")",
            Symbol::Term2 => "NUMBER",
            Symbol::Nonterm0 => "Term",
            Symbol::Nonterm1 => "Number",
        }
    }
}

impl PartialEq<Token> for Symbol {
    fn eq(&self, other: &Token) -> bool {
        match self {
            Symbol::Term0 => {
                matches!(other, Token::LParen)
            }
            Symbol::Term1 => {
                matches!(other, Token::RParen)
            }
            Symbol::Term2 => {
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

pub fn parse(lexer: impl Iterator<Item = Token>) -> Result<AstTerm, ParseError> {
    let mut lexer = lexer.peekable();
    let mut stack = vec![SymbolOrReduction::Symbol(Symbol::Nonterm0)];
    let mut results = Vec::new();

    while let Some(item) = stack.pop() {
        let symbol = match item {
            SymbolOrReduction::Symbol(symbol) => symbol,
            SymbolOrReduction::Reduction(reduction) => {
                reduction(&mut results);
                continue;
            }
        };

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
                (Symbol::Nonterm0, Token::LParen) => {
                    stack.push(SymbolOrReduction::Reduction(reduce_nonterm0_1));
                    stack.push(SymbolOrReduction::Symbol(Symbol::Term1));
                    stack.push(SymbolOrReduction::Symbol(Symbol::Nonterm0));
                    stack.push(SymbolOrReduction::Symbol(Symbol::Term0));
                }
                (Symbol::Nonterm0, Token::Number) => {
                    stack.push(SymbolOrReduction::Reduction(reduce_nonterm0_0));
                    stack.push(SymbolOrReduction::Symbol(Symbol::Nonterm1));
                }
                (Symbol::Nonterm1, Token::Number) => {
                    stack.push(SymbolOrReduction::Reduction(reduce_nonterm1_0));
                    stack.push(SymbolOrReduction::Symbol(Symbol::Term2));
                }
                (symbol, _) => {
                    return Err(ParseError::UnrecognizedToken {
                        expected: match symbol {
                            Symbol::Nonterm0 => {
                                // First of Term
                                vec!["\"NUMBER\"", "\"(\""]
                            }
                            Symbol::Nonterm1 => {
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

    Ok(pop_nonterm0(&mut results))
}

fn pop_nonterm0(results: &mut Vec<ActionResult>) -> AstTerm {
    match results.pop() {
        Some(ActionResult::Nonterm0(value)) => value,
        _ => panic!("Unexpected action result"),
    }
}

fn pop_nonterm1(results: &mut Vec<ActionResult>) -> AstTerm {
    match results.pop() {
        Some(ActionResult::Nonterm1(value)) => value,
        _ => {
            panic!("Unexpected action result")
        }
    }
}

fn reduce_nonterm0_0(results: &mut Vec<ActionResult>) {
    let n = pop_nonterm1(results);
    let result = action_nonterm0_0(n);
    results.push(ActionResult::Nonterm0(result));
}

fn action_nonterm0_0(n: AstTerm) -> AstTerm {
    n
}

fn reduce_nonterm0_1(results: &mut Vec<ActionResult>) {
    let t = pop_nonterm0(results);
    let result = action_nonterm0_1(t);
    results.push(ActionResult::Nonterm0(result));
}

fn action_nonterm0_1(t: AstTerm) -> AstTerm {
    AstTerm::Paren(Box::new(t))
}

fn reduce_nonterm1_0(results: &mut Vec<ActionResult>) {
    let result = action_nonterm1_0();
    results.push(ActionResult::Nonterm1(result));
}

fn action_nonterm1_0() -> AstTerm {
    AstTerm::Number
}
