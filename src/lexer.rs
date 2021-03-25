use logos::{Lexer, Logos};
use std::ops::Range;

#[derive(Logos, Debug, Clone)]
pub enum Token<'input> {
    #[token("grammar")]
    Grammar,
    #[token("token")]
    Token,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("=>")]
    EqArrow,
    #[token("=")]
    Equal,
    #[token(",")]
    Comma,
    #[token(")")]
    RParen,
    #[token("(")]
    LParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("|")]
    VBAR,

    #[regex("[a-zA-Z][a-zA-Z0-9]*")]
    Identifier(&'input str),

    #[error]
    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub struct LalrpopLexerIter<'input> {
    pub lexer: Lexer<'input, Token<'input>>,
}

impl<'input> LalrpopLexerIter<'input> {
    pub fn new(lexer: Lexer<'input, Token<'input>>) -> Self {
        Self { lexer }
    }
}

impl<'input> Iterator for LalrpopLexerIter<'input> {
    type Item = Result<(usize, Token<'input>, usize), Range<usize>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(token) => {
                let span = self.lexer.span();
                Some(match token {
                    Token::Error => Err(span),
                    token => Ok((span.start, token, span.end)),
                })
            }
            None => None,
        }
    }
}
