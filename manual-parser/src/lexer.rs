use logos::Logos;

#[derive(Logos, Debug, Clone)]
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
