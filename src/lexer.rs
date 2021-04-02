use logos::{Lexer, Logos};
use std::ops::Range;

#[derive(Logos, Debug, Clone)]
pub enum Token<'input> {
    #[token("grammar")]
    Grammar,
    #[token("token")]
    Token,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
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
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,

    /// Identifiers are alphanumeric names
    #[regex("[a-zA-Z][a-zA-Z0-9]*")]
    Identifier(&'input str),

    /// Terminals are anything enclosed in double quotes
    #[regex("\"[^\"]+\"")]
    Terminal(&'input str),

    // These two tokens are generated by the sub-lexers ImportToken and ActionToken
    ImportCode(&'input str),
    ActionCode(&'input str),

    #[error]
    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Logos)]
enum ImportToken<'input> {
    /// This token represents a Rust import expression (everything after "use"
    /// and before ";").
    #[regex(r"[^ \t\n\f][^;]*")]
    ImportCode(&'input str),

    #[error]
    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Logos)]
enum ActionToken<'input> {
    /// This token represents Rust action code (code that appears after "=>"),
    /// which will be lexed by matching parens/braces/brackets.
    #[regex(r".", parse_action_code)]
    ActionCode(&'input str),

    #[error]
    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip, priority = 2)]
    Error,
}

fn parse_action_code<'input>(lexer: &mut Lexer<'input, ActionToken<'input>>) -> &'input str {
    // FIXME: This lexing fails in multiple ways because it doesn't handle
    //        strings, comments, and other elements of Rust code that could
    //        contain extra parens or comma/semicolons. Try replicating
    //        LALRPOP's lexing behavior:
    //        https://github.com/lalrpop/lalrpop/blob/fc9986c725d908a60b11d8480711afa33f7f3564/lalrpop/src/tok/mod.rs#L433
    let mut balance = 0;

    // Get the initial char so we can consider it in parens matching
    let first_char = lexer.slice().chars().next().unwrap();
    let chars = std::iter::once(first_char).chain(lexer.remainder().chars());

    for (i, c) in chars.enumerate() {
        match c {
            '(' | '[' | '{' => balance += 1,
            ')' | ']' | '}' => {
                // Check if we're expecting a closing brace. If not, then we're
                // done with the Rust code
                if balance == 0 {
                    break;
                }

                balance -= 1
            }
            ';' | ',' => {
                if balance == 0 {
                    break;
                }
            }
            _ => {}
        }

        if i != 0 {
            // Don't bump the first char because it's already in the slice
            lexer.bump(c.len_utf8());
        }
    }

    lexer.slice()
}

/// Wrap the lexer with some state so it can switch between lexer
/// implementations depending on the context. This is used so we can avoid
/// parsing Rust code, skipping over it in imports and action code blocks.
pub struct StatefulLexer<'input> {
    state: LexerState<'input>,
}

enum LexerState<'input> {
    Normal(Lexer<'input, Token<'input>>),
    Import(Lexer<'input, ImportToken<'input>>),
    ActionCode(Lexer<'input, ActionToken<'input>>),
    Done,
}

impl<'input> StatefulLexer<'input> {
    pub fn new(lexer: Lexer<'input, Token<'input>>) -> Self {
        Self {
            state: LexerState::Normal(lexer),
        }
    }
}

impl<'input> Iterator for StatefulLexer<'input> {
    type Item = Result<(usize, Token<'input>, usize), Range<usize>>;

    fn next(&mut self) -> Option<Self::Item> {
        match std::mem::replace(&mut self.state, LexerState::Done) {
            LexerState::Normal(mut lexer) => {
                let token = lexer.next()?;
                let span = lexer.span();

                self.state = match token {
                    Token::Use => LexerState::Import(lexer.morph()),
                    Token::EqArrow => LexerState::ActionCode(lexer.morph()),
                    _ => LexerState::Normal(lexer),
                };

                Some(match token {
                    Token::Error => Err(span),
                    _ => Ok((span.start, token, span.end)),
                })
            }
            LexerState::Import(mut lexer) => {
                let token = lexer.next()?;
                let span = lexer.span();
                self.state = LexerState::Normal(lexer.morph());
                Some(match token {
                    ImportToken::ImportCode(code) => {
                        Ok((span.start, Token::ImportCode(code), span.end))
                    }
                    ImportToken::Error => Err(span),
                })
            }
            LexerState::ActionCode(mut lexer) => {
                let token = lexer.next()?;
                let span = lexer.span();
                self.state = LexerState::Normal(lexer.morph());
                Some(match token {
                    ActionToken::ActionCode(code) => {
                        Ok((span.start, Token::ActionCode(code), span.end))
                    }
                    ActionToken::Error => Err(span),
                })
            }
            LexerState::Done => None,
        }
    }
}
