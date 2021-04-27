use crate::ast::{
    AstGrammar, AstNonterminal, AstProduction, AstSymbol, AstTokenAlias, AstTokenDecl,
    AstTokenPattern, AstTypePath, AstTypeRef,
};

macro_rules! parse_grammar {
    ($($grammar:tt)*) => {{
        let grammar = stringify!($($grammar)*);

        let lexer = crate::lexer::StatefulLexer::new(
            <crate::lexer::Token as logos::Logos>::lexer(&grammar)
        );
        crate::parsing::parser::GrammarParser::new()
            .parse(lexer)
            .expect("Grammar should parse")
    }};
}

macro_rules! grammar_test {
    (grammar { $($grammar:tt)* }, $expected_ast:expr) => {{
        let actual_ast = parse_grammar! { $($grammar)* };
        assert_eq!(actual_ast, $expected_ast);
    }};
}

#[test]
fn empty_grammar() {
    grammar_test! {
        grammar {
            token Token {}
            grammar;
        },
        AstGrammar {
            imports: Vec::new(),
            token_decl: AstTokenDecl {
                name: "Token",
                aliases: Vec::new()
            },
            nonterminals: Vec::new()
        }
    };
}

#[test]
fn token_aliases() {
    grammar_test! {
        grammar {
            token Token {
                "(" = Token::LParen,
                ")" = Token::RParen,
                "NUMBER" = Token::Number,
            }
            grammar;
        },
        AstGrammar {
            imports: Vec::new(),
            token_decl: AstTokenDecl {
                name: "Token",
                aliases: vec![
                    AstTokenAlias{
                        term: "\"(\"",
                        pattern: AstTokenPattern{
                            ty: "Token",
                            variant: "LParen"
                        }
                    },
                    AstTokenAlias {
                        term: "\")\"",
                        pattern: AstTokenPattern {
                            ty: "Token",
                            variant: "RParen"
                        }
                    },
                    AstTokenAlias {
                        term: "\"NUMBER\"",
                        pattern: AstTokenPattern {
                            ty: "Token",
                            variant: "Number"
                        }
                    }
                ]
            },
            nonterminals: Vec::new()
        }
    }
}

#[test]
fn imports() {
    grammar_test! {
        grammar {
            use crate::Token;
            use std::collections::{HashMap, HashSet};
            use std::{
                self,
                iter::{self, once}
            };

            token Token {}
            grammar;
        },
        AstGrammar {
            imports: vec![
                "crate :: Token",
                "std :: collections :: { HashMap, HashSet }",
                "std\n:: { self, iter :: { self, once } }"
            ],
            token_decl: AstTokenDecl {
                name: "Token",
                aliases: Vec::new()
            },
            nonterminals: Vec::new()
        }
    }
}

#[test]
fn nonterminals() {
    grammar_test! {
        grammar {
            token Token {}
            grammar;
            pub Start: () = "token1" Nonterm1 => ();
            Nonterm1: () = {
                "token2" => (),
                "token3" "token4" => (),
                <var:Nonterm2> => var,
            };
            Nonterm2: () = "token5" => ();
        },
        AstGrammar {
            imports: Vec::new(),
            token_decl: AstTokenDecl {
                name: "Token",
                aliases: Vec::new()
            },
            nonterminals: vec![
                AstNonterminal {
                    is_pub: true,
                    name: "Start",
                    ty: AstTypeRef::Tuple(Vec::new()),
                    productions: vec![AstProduction {
                        symbols: vec![AstSymbol::Terminal("\"token1\""), AstSymbol::Nonterminal("Nonterm1")],
                        code: "()"
                    }]
                },
                AstNonterminal {
                    is_pub: false,
                    name: "Nonterm1",
                    ty: AstTypeRef::Tuple(Vec::new()),
                    productions: vec![
                        AstProduction {
                            symbols: vec![AstSymbol::Terminal("\"token2\"")],
                            code: "()"
                        },
                        AstProduction {
                            symbols: vec![AstSymbol::Terminal("\"token3\""), AstSymbol::Terminal("\"token4\"")],
                            code: "()"
                        },
                        AstProduction {
                            symbols: vec![AstSymbol::Named("var", "Nonterm2")],
                            code: "var"
                        }
                    ]
                },
                AstNonterminal {
                    is_pub: false,
                    name: "Nonterm2",
                    ty: AstTypeRef::Tuple(Vec::new()),
                    productions: vec![AstProduction {
                        symbols: vec![AstSymbol::Terminal("\"token5\"")],
                        code: "()"
                    }]
                }
            ]
        }
    }
}

#[test]
fn nonterminal_types() {
    grammar_test! {
        grammar {
            token Token {}
            grammar;
            pub Start: (usize, String) = "token1" => (1, "test".to_string());
            Nonterm1: crate::lexer::Token = "token2" => Token::LParen;
            Nonterm2: ::std::collections::HashMap<usize, String> = "token3" => HashMap::new();
        },
        AstGrammar {
            imports: Vec::new(),
            token_decl: AstTokenDecl {
                name: "Token",
                aliases: Vec::new()
            },
            nonterminals: vec![
                AstNonterminal {
                    is_pub: true,
                    name: "Start",
                    ty: AstTypeRef::Tuple(vec![
                        AstTypeRef::simple_ty(vec!["usize"]),
                        AstTypeRef::simple_ty(vec!["String"]),
                    ]),
                    productions: vec![AstProduction{
                        symbols: vec![AstSymbol::Terminal("\"token1\"")],
                        code: "(1, \"test\" . to_string())"
                    }]
                },
                AstNonterminal {
                    is_pub: false,
                    name: "Nonterm1",
                    ty: AstTypeRef::simple_ty(vec!["crate", "lexer", "Token"]),
                    productions: vec![AstProduction {
                        symbols: vec![AstSymbol::Terminal("\"token2\"")],
                        code: "Token :: LParen"
                    }]
                },
                AstNonterminal {
                    is_pub: false,
                    name: "Nonterm2",
                    ty: AstTypeRef::Ty(
                        AstTypePath {
                            is_absolute: true,
                            segments: vec!["std", "collections", "HashMap"]
                        },
                        vec![
                            AstTypeRef::simple_ty(vec!["usize"]),
                            AstTypeRef::simple_ty(vec!["String"]),
                        ]
                    ),
                    productions: vec![AstProduction {
                        symbols: vec![AstSymbol::Terminal("\"token3\"")],
                        code: "HashMap :: new()"
                    }]
                }
            ]
        }
    }
}
