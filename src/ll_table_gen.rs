//! LL(1) action table generation

use crate::ast::{AstGrammar, AstNonterminal, AstProduction, AstSymbol};
use std::collections::{HashMap, HashSet};

pub(crate) type NullableMap<'input> = HashMap<&'input str, bool>;
pub(crate) type FirstMap<'input> = HashMap<&'input str, HashSet<&'input str>>;
pub(crate) type FollowMap<'input> = HashMap<&'input str, HashSet<&'input str>>;
pub(crate) type ParseTable<'input> =
    HashMap<(&'input str, &'input str), HashSet<AstProduction<'input>>>;

const WRAPPER_NONTERM: &str = "__ll_parser_wrapper_start";
pub const EOF_TERMINAL: &str = "\"__ll_parser_eof\"";

impl<'input> AstGrammar<'input> {
    /// Get the terminals used in the grammar
    pub fn terminals<'a>(&'a self) -> impl Iterator<Item = &'input str> + 'a {
        self.token_decl
            .aliases
            .iter()
            .map(|alias| alias.term)
            .chain(Some(EOF_TERMINAL))
    }

    /// Get the nonterminals used in the grammar
    pub fn nonterminals<'a>(&'a self) -> impl Iterator<Item = &'input str> + 'a {
        self.nonterminals.iter().map(|nonterminal| nonterminal.name)
    }

    /// Get the productions used in the grammar.
    /// The first element is the nonterminal name, the second is the symbols of
    /// the production.
    pub fn productions<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'input str, AstProduction<'input>)> + 'a {
        self.nonterminals.iter().flat_map(|nonterminal| {
            nonterminal
                .productions
                .iter()
                .map(move |production| (nonterminal.name, production.clone()))
        })
    }

    /// Get the productions used in the grammar, along with their index in the nonterminal's
    /// production list.
    pub fn productions_indexed<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a AstNonterminal<'input>, &'a AstProduction<'input>, usize)> + 'a
    {
        self.nonterminals.iter().flat_map(|nonterminal| {
            nonterminal
                .productions
                .iter()
                .enumerate()
                .map(move |(i, production)| (nonterminal, production, i))
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

pub fn insert_wrapper_start_nonterm(ast: &mut AstGrammar) {
    let start_nonterm = ast
        .nonterminals
        .iter_mut()
        .find(|nonterminal| nonterminal.is_pub)
        .expect("Must have a single public nonterminal");
    start_nonterm.is_pub = false;

    let wrapper_nonterm = AstNonterminal {
        is_pub: true,
        name: WRAPPER_NONTERM,
        ty: start_nonterm.ty.clone(),
        productions: vec![AstProduction {
            symbols: vec![
                AstSymbol::Named("result", start_nonterm.name),
                AstSymbol::Terminal(EOF_TERMINAL),
            ],
            code: "result",
        }],
    };

    ast.nonterminals.push(wrapper_nonterm);
}

pub fn compute_nullable<'input>(ast: &AstGrammar<'input>) -> NullableMap<'input> {
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
        for (nonterminal, production) in &productions {
            if !nullable[nonterminal]
                && production
                    .symbols
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

pub fn compute_first<'input>(
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
        for (nonterm, production) in &productions {
            let symbols = &production.symbols;
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

pub fn compute_follow<'input>(
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
        for (nonterm, production) in &productions {
            let symbols = &production.symbols;
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

pub fn compute_parse_table<'input>(
    ast: &AstGrammar<'input>,
    nullable: &NullableMap<'input>,
    first: &FirstMap<'input>,
    follow: &FollowMap<'input>,
) -> ParseTable<'input> {
    let terminals: Vec<_> = ast.terminals().collect();
    let mut parse_table = HashMap::new();

    for nonterm in ast.nonterminals() {
        for term in &terminals {
            parse_table.insert((nonterm, *term), HashSet::new());
        }
    }

    for (nonterminal, production) in ast.productions() {
        if production
            .symbols
            .iter()
            .all(|symbol| nullable[symbol.term_or_nonterm()])
        {
            for term in &follow[nonterminal] {
                parse_table
                    .get_mut(&(nonterminal, *term))
                    .unwrap()
                    .insert(production.clone());
            }
        }

        for term in first_range(&production.symbols, first, nullable) {
            parse_table
                .get_mut(&(nonterminal, term))
                .unwrap()
                .insert(production.clone());
        }
    }

    parse_table
}

/// Compute the possible first terminals in a range of symbols
fn first_range<'input>(
    symbols: &[AstSymbol<'input>],
    first: &FollowMap<'input>,
    nullable: &NullableMap<'input>,
) -> HashSet<&'input str> {
    let mut new_first = HashSet::new();

    for symbol in symbols {
        let symbol = symbol.term_or_nonterm();
        new_first.extend(first[symbol].clone());

        if !nullable[symbol] {
            break;
        }
    }

    new_first
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

    macro_rules! symbol {
        ($nonterm:ident) => {
            AstSymbol::Nonterminal(stringify!($nonterm))
        };
        ($term:expr) => {
            AstSymbol::Terminal(stringify!($term))
        };
    }

    macro_rules! symbols {
        ($($sym:tt)*) => {
            vec![$(symbol!($sym)),*]
        };
    }

    #[test]
    fn nullable_basic_grammar() {
        let mut ast = parse_grammar! {
            token Token {
                "a" = Token::A
            }
            grammar;
            pub MyNonterminal: () = "a" => ();
            MyEmptyNonterminal: () = => ();
        };
        insert_wrapper_start_nonterm(&mut ast);

        assert_eq!(
            compute_nullable(&ast),
            collection! {
                "MyNonterminal" => false,
                "MyEmptyNonterminal" => true,
                "\"a\"" => false,
                WRAPPER_NONTERM => false,
                EOF_TERMINAL => false,
            }
        );
    }

    #[test]
    fn infix_parens() {
        let mut ast = parse_grammar! {
            token Token {
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
        insert_wrapper_start_nonterm(&mut ast);

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
                WRAPPER_NONTERM => false,
                EOF_TERMINAL => false,
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
                WRAPPER_NONTERM => collection! { "\"var\"", "\"!\"", "\"(\"" },
                EOF_TERMINAL => collection! { EOF_TERMINAL }
            }
        );

        let follow = compute_follow(&ast, &nullable, &first);
        assert_eq!(
            follow,
            collection! {
                "P" => collection! { "\")\"", EOF_TERMINAL },
                "O" => collection!{ "\")\"",EOF_TERMINAL },
                "OP" => collection!{ "\")\"",EOF_TERMINAL },
                "A" => collection!{ "\"||\"", "\")\"",EOF_TERMINAL },
                "AP" => collection!{ "\"||\"", "\")\"",EOF_TERMINAL },
                "Z" => collection!{ "\"||\"", "\"&&\"", "\")\"",EOF_TERMINAL },
                WRAPPER_NONTERM => collection!()
            }
        );

        let code = "()";
        assert_eq!(
            compute_parse_table(&ast, &nullable, &first, &follow),
            collection! {
                (WRAPPER_NONTERM, "\"var\"") => collection!(AstProduction {
                    symbols: vec![
                        AstSymbol::Named("result", "P"),
                        AstSymbol::Terminal(EOF_TERMINAL)
                    ],
                    code: "result"
                }),
                (WRAPPER_NONTERM, "\"!\"") => collection!(AstProduction {
                    symbols: vec![
                        AstSymbol::Named("result", "P"),
                        AstSymbol::Terminal(EOF_TERMINAL)
                    ],
                    code: "result"
                }),
                (WRAPPER_NONTERM, "\"&&\"") => collection!(),
                (WRAPPER_NONTERM, "\"||\"") => collection!(),
                (WRAPPER_NONTERM, "\"(\"") => collection!(AstProduction {
                    symbols: vec![
                        AstSymbol::Named("result", "P"),
                        AstSymbol::Terminal(EOF_TERMINAL)
                    ],
                    code: "result"
                }),
                (WRAPPER_NONTERM, "\")\"") => collection!(),
                (WRAPPER_NONTERM, EOF_TERMINAL) => collection!(),

                ("P", "\"var\"") => collection!(AstProduction {
                    symbols: symbols!(O), code
                }),
                ("P", "\"!\"") => collection!(AstProduction {
                    symbols: symbols!(O), code
                }),
                ("P", "\"&&\"") => collection!(),
                ("P", "\"||\"") => collection!(),
                ("P", "\"(\"") => collection!(AstProduction {
                    symbols: symbols!(O), code
                }),
                ("P", "\")\"") => collection!(),
                ("P", EOF_TERMINAL) => collection!(),

                ("O", "\"var\"") => collection!(AstProduction {
                    symbols: symbols!(A OP), code
                }),
                ("O", "\"!\"") => collection!(AstProduction {
                    symbols: symbols!(A OP), code
                }),
                ("O", "\"&&\"") => collection!(),
                ("O", "\"||\"") => collection!(),
                ("O", "\"(\"") => collection!(AstProduction {
                    symbols: symbols!(A OP), code
                }),
                ("O", "\")\"") => collection!(),
                ("O", EOF_TERMINAL) => collection!(),

                ("OP", "\"var\"") => collection!(),
                ("OP", "\"!\"") => collection!(),
                ("OP", "\"&&\"") => collection!(),
                ("OP", "\"||\"") => collection!(AstProduction {
                    symbols: symbols!("||" A OP), code
                }),
                ("OP", "\"(\"") => collection!(),
                ("OP", "\")\"") => collection!(AstProduction {
                    symbols: symbols!(), code
                }),
                ("OP", EOF_TERMINAL) => collection!(AstProduction {
                    symbols: symbols!(), code
                }),

                ("A", "\"var\"") => collection!(AstProduction {
                    symbols: symbols!(Z AP), code
                }),
                ("A", "\"!\"") => collection!(AstProduction {
                    symbols: symbols!(Z AP), code
                }),
                ("A", "\"&&\"") => collection!(),
                ("A", "\"||\"") => collection!(),
                ("A", "\"(\"") => collection!(AstProduction {
                    symbols: symbols!(Z AP), code
                }),
                ("A", "\")\"") => collection!(),
                ("A", EOF_TERMINAL) => collection!(),

                ("AP", "\"var\"") => collection!(),
                ("AP", "\"!\"") => collection!(),
                ("AP", "\"&&\"") => collection!(AstProduction {
                    symbols: symbols!("&&" Z AP), code
                }),
                ("AP", "\"||\"") => collection!(AstProduction {
                    symbols: symbols!(), code
                }),
                ("AP", "\"(\"") => collection!(),
                ("AP", "\")\"") => collection!(AstProduction {
                    symbols: symbols!(), code
                }),
                ("AP", EOF_TERMINAL) => collection!(AstProduction {
                    symbols: symbols!(), code
                }),

                ("Z", "\"var\"") => collection!(AstProduction {
                    symbols: symbols!("var"), code
                }),
                ("Z", "\"!\"") => collection!(AstProduction {
                    symbols: symbols!("!" Z), code
                }),
                ("Z", "\"&&\"") => collection!(),
                ("Z", "\"||\"") => collection!(),
                ("Z", "\"(\"") => collection!(AstProduction {
                    symbols: symbols!("(" P ")"), code
                }),
                ("Z", "\")\"") => collection!(),
                ("Z", EOF_TERMINAL) => collection!(),
            }
        );
    }
}
