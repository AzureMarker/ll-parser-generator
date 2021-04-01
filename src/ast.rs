#[derive(Debug)]
pub struct AstGrammar<'input> {
    pub imports: Vec<&'input str>,
    pub token_decl: AstTokenDecl,
    pub nonterminals: Vec<AstNonterminal<'input>>,
}

#[derive(Debug)]
pub struct AstTokenDecl {
    // TODO
}

#[derive(Debug)]
pub struct AstNonterminal<'input> {
    pub name: &'input str,
    pub ty: AstTypeRef,
    pub productions: Vec<AstProduction<'input>>,
}

#[derive(Debug)]
pub struct AstTypeRef {
    // TODO
}

#[derive(Debug)]
pub struct AstProduction<'input> {
    pub symbols: Vec<AstSymbol>,
    pub code: &'input str,
}

#[derive(Debug)]
pub struct AstSymbol {
    // TODO
}
