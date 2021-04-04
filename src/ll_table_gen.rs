//! LL(1) action table generation

use crate::ast::{AstGrammar, AstSymbol};
use std::collections::{HashMap, HashSet};

type NullableMap<'input> = HashMap<&'input str, bool>;
type FirstMap<'input> = HashMap<&'input str, HashSet<&'input str>>;

impl<'input> AstGrammar<'input> {
    /// Get the terminals used in the grammar
    fn terminals<'a>(&'a self) -> impl Iterator<Item = &'input str> + 'a {
        self.token_decl.aliases.iter().map(|alias| alias.term)
    }

    /// Get the nonterminals used in the grammar
    fn nonterminals<'a>(&'a self) -> impl Iterator<Item = &'input str> + 'a {
        self.nonterminals.iter().map(|nonterminal| nonterminal.name)
    }

    /// Get the productions used in the grammar.
    /// The first element is the nonterminal name, the second is the symbols of
    /// the production.
    fn productions<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'input str, Vec<AstSymbol<'input>>)> + 'a {
        self.nonterminals.iter().flat_map(|nonterminal| {
            nonterminal
                .productions
                .iter()
                .map(move |production| (nonterminal.name, production.symbols.clone()))
        })
    }
}

impl<'input> AstSymbol<'input> {
    fn term_or_nonterm(&self) -> &'input str {
        match self {
            AstSymbol::Terminal(term) => *term,
            AstSymbol::Nonterminal(nonterm) => *nonterm,
            AstSymbol::Named(_, nonterm) => *nonterm,
        }
    }
}

fn compute_nullable<'input>(ast: &AstGrammar<'input>) -> NullableMap<'input> {
    let mut nullable = HashMap::new();

    for terminal in ast.terminals() {
        nullable.insert(terminal, false);
    }

    for nonterminal in ast.nonterminals() {
        nullable.insert(nonterminal, false);
    }

    let mut changed = true;
    let productions: Vec<_> = ast.productions().collect();
    while changed {
        changed = false;
        for (nonterminal, symbols) in &productions {
            if symbols
                .iter()
                .all(|symbol| nullable[symbol.term_or_nonterm()])
            {
                nullable.insert(nonterminal, true);
                changed = true;
            }
        }
    }

    nullable
}

fn compute_first<'input>(
    ast: &AstGrammar<'input>,
    nullable: NullableMap<'input>,
) -> FirstMap<'input> {
    let mut first = HashMap::new();

    for term in ast.terminals() {
        let mut set = HashSet::new();
        set.insert(term);
        first.insert(term, set);
    }

    for nonterm in ast.nonterminals() {
        first.insert(nonterm, HashSet::new());
    }

    let productions: Vec<_> = ast.productions().collect();
    let mut changed = true;
    while changed {
        changed = false;
        for (nonterm, symbols) in &productions {
            for i in 1..symbols.len() {
                if symbols[0..i]
                    .iter()
                    .all(|symbol| nullable[symbol.term_or_nonterm()])
                {
                    let next_symbol = first[symbols[i].term_or_nonterm()].clone();
                    let nonterm_first = first.get_mut(nonterm).unwrap();
                    nonterm_first.extend(next_symbol);
                    changed = true;
                }
            }
        }
    }

    first
}
