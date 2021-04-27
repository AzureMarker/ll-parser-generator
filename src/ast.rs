#[derive(Debug, Eq, PartialEq)]
pub struct AstGrammar<'input> {
    pub imports: Vec<&'input str>,
    pub token_decl: AstTokenDecl<'input>,
    pub nonterminals: Vec<AstNonterminal<'input>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AstTokenDecl<'input> {
    pub name: &'input str,
    pub aliases: Vec<AstTokenAlias<'input>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AstTokenAlias<'input> {
    pub term: &'input str,
    pub pattern: AstTokenPattern<'input>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AstTokenPattern<'input> {
    pub ty: &'input str,
    pub variant: &'input str,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AstNonterminal<'input> {
    pub is_pub: bool,
    pub name: &'input str,
    pub ty: AstTypeRef<'input>,
    pub productions: Vec<AstProduction<'input>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AstTypeRef<'input> {
    Ty(AstTypePath<'input>, Vec<AstTypeRef<'input>>),
    Tuple(Vec<AstTypeRef<'input>>),
}

impl<'input> AstTypeRef<'input> {
    #[cfg(test)]
    pub fn simple_ty(segments: Vec<&'input str>) -> Self {
        AstTypeRef::Ty(
            AstTypePath {
                is_absolute: false,
                segments,
            },
            Vec::new(),
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AstTypePath<'input> {
    pub is_absolute: bool,
    pub segments: Vec<&'input str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AstProduction<'input> {
    pub symbols: Vec<AstSymbol<'input>>,
    pub code: &'input str,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum AstSymbol<'input> {
    Terminal(&'input str),
    Nonterminal(&'input str),
    Named(&'input str, &'input str),
}
