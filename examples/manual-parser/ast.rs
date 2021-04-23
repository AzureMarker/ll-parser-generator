#[derive(Debug)]
pub enum AstTerm {
    Number,
    Paren(Box<AstTerm>)
}