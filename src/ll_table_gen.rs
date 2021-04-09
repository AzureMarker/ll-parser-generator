//! LL(1) action table generation

use crate::ast::{AstGrammar, AstSymbol};
use std::collections::{HashMap, HashSet};

type NullableMap<'input> = HashMap<&'input str, bool>;
type FirstMap<'input> = HashMap<&'input str, HashSet<&'input str>>;
type FollowMap<'input> = HashMap<&'input str, HashSet<&'input str>>;

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
            if !nullable[nonterminal]
                && symbols
                    .iter()
                    .all(|symbol| nullable[symbol.term_or_nonterm()])
            {
                changed = true;
                nullable.insert(nonterminal, true);
            }
        }
    }

    nullable
}

fn compute_first<'input>(
    ast: &AstGrammar<'input>,
    nullable: &NullableMap<'input>,
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
            for i in 0..symbols.len() {
                if symbols[..i]
                    .iter()
                    .all(|symbol| nullable[symbol.term_or_nonterm()])
                {
                    let next_symbol = first[symbols[i].term_or_nonterm()].clone();
                    let nonterm_first = first.get_mut(nonterm).unwrap();

                    if !nonterm_first.is_superset(&next_symbol) {
                        changed = true;
                        nonterm_first.extend(next_symbol);
                    }
                }
            }
        }
    }

    first
}

fn compute_follow<'input>(
    ast: &AstGrammar<'input>,
    nullable: &NullableMap<'input>,
    first: &FirstMap<'input>,
) -> FollowMap<'input> {
    let mut follow = HashMap::new();

    let nonterminals: HashSet<_> = ast.nonterminals().collect();

    for nonterm in &nonterminals {
        follow.insert(*nonterm, HashSet::new());
    }
    let productions: Vec<_> = ast.productions().collect();

    let mut changed = true;
    while changed {
        changed = false;
        for (nonterm, symbols) in &productions {
            for i in 0..symbols.len() {
                if !nonterminals.contains(symbols[i].term_or_nonterm()) {
                    continue;
                }

                if symbols[(i + 1)..]
                    .iter()
                    .all(|symbol| nullable[symbol.term_or_nonterm()])
                {
                    let nonterm_follow = follow[nonterm].clone();
                    let symbol_follow = follow.get_mut(symbols[i].term_or_nonterm()).unwrap();

                    if !symbol_follow.is_superset(&nonterm_follow) {
                        changed = true;
                        symbol_follow.extend(nonterm_follow);
                    }
                }
                for j in (i + 1)..symbols.len() {
                    if symbols[(i + 1)..j]
                        .iter()
                        .all(|symbol| nullable[symbol.term_or_nonterm()])
                    {
                        let next_terminals = first[symbols[j].term_or_nonterm()].clone();
                        let symbol_follow = follow.get_mut(symbols[i].term_or_nonterm()).unwrap();

                        if !symbol_follow.is_superset(&next_terminals) {
                            changed = true;
                            symbol_follow.extend(next_terminals);
                        }
                    }
                }
            }
        }
    }

    follow
}

#[cfg(test)]
mod tests {
    use super::*;

    // Taken from https://stackoverflow.com/a/27582993
    macro_rules! collection {
        // map-like
        ($($k:expr => $v:expr),* $(,)?) => {
            std::iter::Iterator::collect(std::array::IntoIter::new([$(($k, $v),)*]))
        };
        // set-like
        ($($v:expr),* $(,)?) => {
            std::iter::Iterator::collect(std::array::IntoIter::new([$($v,)*]))
        };
    }

    #[test]
    fn nullable_basic_grammar() {
        let ast = parse_grammar! {
            token Token {
                "a" = Token::A
            }
            grammar;
            MyNonterminal: () = "a" => ();
            MyEmptyNonterminal: () = => ();
        };

        assert_eq!(
            compute_nullable(&ast),
            collection! {
                "MyNonterminal" => false,
                "MyEmptyNonterminal" => true,
                "\"a\"" => false
            }
        );
    }

    #[test]
    fn infix_parens() {
        let ast = parse_grammar! {
            token Token{
                "var" = Token::Var,
                "(" = Token::LParen,
                ")" = Token::RParen,
                "!" = Token::Not,
                "&&" = Token::And,
                "||" = Token::Or
            }
            grammar;

            pub P: () = O => ();

            O: () = A OP => ();
            OP: () = {
                "||" A OP => (),
                => (),
            };

            A: () = Z AP => ();
            AP: () = {
                "&&" Z AP => (),
                => (),
            };

            Z: () = {
                "var" => (),
                "!" Z => (),
                "(" P ")" => (),
            };
        };

        let nullable = compute_nullable(&ast);
        assert_eq!(
            nullable,
            collection! {
                "P" => false,
                "O" => false,
                "OP" => true,
                "A" => false,
                "AP" => true,
                "Z" => false,
                "\"var\"" => false,
                "\"(\"" => false,
                "\")\"" => false,
                "\"!\"" => false,
                "\"&&\"" => false,
                "\"||\"" => false,
            }
        );

        let first = compute_first(&ast, &nullable);
        assert_eq!(
            first,
            collection! {
                "P" => collection! { "\"var\"", "\"!\"", "\"(\"" },
                "O" => collection!{ "\"var\"", "\"!\"", "\"(\"" },
                "OP" => collection!{ "\"||\"" },
                "A" => collection!{ "\"var\"", "\"!\"", "\"(\"" },
                "AP" => collection!{ "\"&&\"" },
                "Z" => collection!{ "\"var\"", "\"!\"", "\"(\"" },
                "\"var\"" => collection! { "\"var\"" },
                "\"(\"" => collection! { "\"(\"" },
                "\")\"" => collection! { "\")\"" },
                "\"!\"" => collection! { "\"!\"" },
                "\"&&\"" => collection! { "\"&&\"" },
                "\"||\"" => collection! { "\"||\"" },
            }
        );

        assert_eq!(
            compute_follow(&ast, &nullable, &first),
            collection! {
                "P" => collection! { "\")\"" },
                "O" => collection!{ "\")\"" },
                "OP" => collection!{ "\")\"" },
                "A" => collection!{ "\"||\"", "\")\"" },
                "AP" => collection!{ "\"||\"", "\")\"" },
                "Z" => collection!{ "\"||\"", "\"&&\"", "\")\"" },
            }
        )
    }
}
