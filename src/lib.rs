#[macro_use]
extern crate lalrpop_util;

use crate::ast::{AstGrammar, AstProduction, AstSymbol, AstTypePath, AstTypeRef};
use crate::ll_table_gen::{
    compute_first, compute_follow, compute_nullable, compute_parse_table,
    insert_wrapper_start_nonterm, FirstMap, ParseTable, EOF_TERMINAL,
};
use crate::parsing::parse;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use std::str::FromStr;

#[cfg(test)]
#[macro_use]
mod grammar_tests;

mod ast;
mod lexer;
mod ll_table_gen;
mod parsing;

type NameMap<'input> = HashMap<&'input str, Ident>;
type NontermTyMap<'input> = HashMap<&'input str, &'input AstTypeRef<'input>>;
type TokenPatMap<'input> = HashMap<&'input str, TokenStream2>;
/// Map from nonterminal name and production to production ID
type ProductionIdMap<'input> = HashMap<(&'input str, &'input AstProduction<'input>), usize>;

#[proc_macro]
pub fn ll_parser(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let mut ast = parse(&input).unwrap(); // TODO: return error
    insert_wrapper_start_nonterm(&mut ast);

    // Compute LL(1) parse table
    let nullable = compute_nullable(&ast);
    let first = compute_first(&ast, &nullable);
    let follow = compute_follow(&ast, &nullable, &first);
    let parse_table = compute_parse_table(&ast, &nullable, &first, &follow);

    // Check for parse table conflicts
    if let Some(((symbol, terminal), productions)) = parse_table
        .iter()
        .find(|(_, productions)| productions.len() > 1)
    {
        let message = format!(
            "Found a parse-table conflict at symbol \"{}\" and terminal {}:\n{:#?}",
            symbol, terminal, productions
        );

        let result = quote! {
            compile_error!(#message);
        };
        return result.into();
    }

    // Compute some info about names, nonterminals, etc upfront.
    let name_map = generate_name_map(&ast);
    let nonterm_ty_map = generate_nonterm_ty_map(&ast);
    let token_pats = generate_token_pat_map(&ast);
    let production_ids = generate_production_id_map(&ast);

    // Create data structures and functions
    let action_result_enum = generate_action_result_enum(&ast, &name_map);
    let symbols_enum = generate_symbol_enum(&ast, &name_map);
    let symbol_impl = generate_symbol_impl(&ast, &name_map);
    let symbol_eq_impl = generate_partial_eq_impl(&ast, &name_map, &token_pats);
    let action_result_pop_fns = generate_action_result_pop_fns(&ast, &name_map);
    let action_fns = generate_action_fns(&ast, &name_map, &nonterm_ty_map);
    let reduce_fns = generate_reduce_fns(&ast, &name_map);
    let parse_fn = generate_parse_fn(
        &ast,
        &name_map,
        &token_pats,
        &production_ids,
        &parse_table,
        &first,
    );

    // Generate output code
    let imports: Vec<_> = ast
        .imports
        .into_iter()
        .map(TokenStream2::from_str)
        .collect::<Result<_, _>>()
        .unwrap();
    let output = quote! {
        // TODO: allow for user-specified module name
        #(use #imports;)*

        enum SymbolOrReduction {
            Symbol(Symbol),
            Reduction(fn(&mut std::vec::Vec<ActionResult>)),
        }

        #action_result_enum

        #[derive(Debug, PartialEq)]
        pub enum ParseError<T> {
            UnexpectedEOF,
            ExtraToken(T),
            UnrecognizedToken {
                expected: std::vec::Vec<&'static str>,
                found: T,
            },
        }

        #symbols_enum
        #symbol_impl
        #symbol_eq_impl

        impl PartialEq<Symbol> for Token {
            fn eq(&self, other: &Symbol) -> bool {
                other.eq(self)
            }
        }

        #action_result_pop_fns
        #action_fns
        #reduce_fns

        #parse_fn
    };
    output.into()
}

fn generate_parse_fn<'a>(
    ast: &AstGrammar,
    names: &NameMap,
    token_pats: &TokenPatMap,
    production_ids: &ProductionIdMap<'a>,
    parse_table: &'a ParseTable<'a>,
    first_map: &FirstMap,
) -> TokenStream2 {
    let token_ty = Ident::new(ast.token_decl.name, Span::call_site());
    let start_nonterm = ast
        .nonterminals
        .iter()
        .find(|nonterminal| nonterminal.is_pub)
        .expect("Must have a single public nonterminal");
    let return_ty = &start_nonterm.ty;
    let start_nonterm_canonical = &names[start_nonterm.name];
    let return_pop_fn = format_ident!("pop_{}", start_nonterm_canonical);

    let parse_table_matches = parse_table
        .iter()
        .flat_map(|(key, productions)| productions.iter().map(move |production| (*key, production)))
        .map(|((nonterm, next_token), production)| {
            let nonterm_ident = &names[nonterm];

            let token_pat = if next_token == EOF_TERMINAL {
                quote! { None }
            } else {
                let pat = &token_pats[next_token];
                quote! { Some(#pat) }
            };
            let production_id = production_ids[&(nonterm, production)];
            let reduction_fn = format_ident!("reduce_{}_{}", nonterm_ident, production_id);
            let symbol_push_stmts = production.symbols.iter().rev().map(|symbol| {
                let symbol_variant = match symbol {
                    AstSymbol::Terminal(name)
                    | AstSymbol::Nonterminal(name)
                    | AstSymbol::Named(_, name) => &names[*name],
                };

                quote! {
                    stack.push(SymbolOrReduction::Symbol(Symbol::#symbol_variant));
                }
            });

            quote! {
                (Symbol::#nonterm_ident, #token_pat) => {
                    stack.push(SymbolOrReduction::Reduction(#reduction_fn));
                    #(#symbol_push_stmts)*
                }
            }
        });

    let first_match_rules = first_map.iter().map(|(symbol, first_set)| {
        let canonical_name = &names[*symbol];
        let mut first_vec: Vec<_> = first_set.iter().collect();
        first_vec.sort();
        quote! {
            Symbol::#canonical_name => vec![#(#first_vec),*],
        }
    });

    quote! {
        pub fn parse(lexer: impl Iterator<Item = #token_ty>) -> Result<#return_ty, ParseError<#token_ty>> {
            let mut lexer = lexer.peekable();
            let mut stack = vec![SymbolOrReduction::Symbol(Symbol::#start_nonterm_canonical)];
            let mut results = Vec::new();

            while let Some(item) = stack.pop() {
                let symbol = match item {
                    SymbolOrReduction::Symbol(symbol) => symbol,
                    SymbolOrReduction::Reduction(reduction) => {
                        reduction(&mut results);
                        continue;
                    }
                };

                if symbol.is_terminal() {
                    let token = lexer.next();

                    if symbol.is_end() {
                        if let Some(token) = token {
                            return Err(ParseError::ExtraToken(token));
                        } else {
                            continue;
                        }
                    }

                    let token = token.ok_or(ParseError::UnexpectedEOF)?;

                    if symbol == token {
                        continue;
                    } else {
                        return Err(ParseError::UnrecognizedToken {
                            expected: vec![symbol.name()],
                            found: token,
                        });
                    }
                } else {
                    let next_token = lexer.peek();
                    match (symbol, next_token) {
                        #(#parse_table_matches)*
                        (symbol, None) => return Err(ParseError::UnexpectedEOF),
                        (symbol, Some(_)) => {
                            return Err(ParseError::UnrecognizedToken {
                                expected: match symbol {
                                    #(#first_match_rules)*
                                },
                                found: lexer.next().unwrap(),
                            });
                        }
                    }
                }
            }

            if let Some(token) = lexer.next() {
                return Err(ParseError::ExtraToken(token))
            }

            Ok(#return_pop_fn(&mut results))
        }
    }
}

fn generate_action_result_pop_fns(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    ast.nonterminals
        .iter()
        .map(|nonterminal| {
            let canonical_name = &names[nonterminal.name];
            let fn_name = format_ident!("pop_{}", canonical_name);
            let return_ty = &nonterminal.ty;

            quote! {
                fn #fn_name(results: &mut Vec<ActionResult>) -> #return_ty {
                    match results.pop() {
                        Some(ActionResult::#canonical_name(value)) => value,
                        _ => panic!("Unexpected action result"),
                    }
                }
            }
        })
        .collect()
}

fn generate_action_fns(
    ast: &AstGrammar,
    names: &NameMap,
    nonterm_tys: &NontermTyMap,
) -> TokenStream2 {
    ast.productions_indexed()
        .map(|(nonterminal, production, i)| {
            let canonical_name = &names[nonterminal.name];
            let fn_name = format_ident!("action_{}_{}", canonical_name, i);
            let params = production.symbols.iter().filter_map(|symbol| match symbol {
                AstSymbol::Named(name, nonterm) => {
                    let name_ident = Ident::new(name, Span::call_site());
                    let param_ty = nonterm_tys[*nonterm];
                    Some(quote! { #name_ident: #param_ty })
                }
                _ => None,
            });
            let return_ty = &nonterminal.ty;
            let code = TokenStream2::from_str(production.code).unwrap();

            quote! {
                fn #fn_name(#(#params),*) -> #return_ty {
                    #code
                }
            }
        })
        .collect()
}

fn generate_reduce_fns(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    ast.productions_indexed()
        .map(|(nonterminal, production, i)| {
            let canonical_name = &names[nonterminal.name];
            let reduce_fn_name = format_ident!("reduce_{}_{}", canonical_name, i);
            let params = production
                .symbols
                .iter()
                .filter_map(|symbol| match symbol {
                    AstSymbol::Nonterminal(nonterm) => Some((false, *nonterm)),
                    AstSymbol::Named(_, nonterm) => Some((true, nonterm)),
                    AstSymbol::Terminal(_) => None,
                })
                .enumerate();
            let mut param_stmts: Vec<_> = params
                .clone()
                .map(|(j, (is_named, nonterm))| {
                    let canonical_nonterm_name = &names[nonterm];
                    let pop_fn = format_ident!("pop_{}", canonical_nonterm_name);

                    if is_named {
                        let param_name = format_ident!("param{}", j);
                        quote! { let #param_name = #pop_fn(results); }
                    } else {
                        quote! { #pop_fn(results); }
                    }
                })
                .collect();
            param_stmts.reverse();
            let action_fn = format_ident!("action_{}_{}", canonical_name, i);
            let action_params = params.filter_map(|(j, (is_named, _))| {
                if !is_named {
                    return None;
                }
                Some(format_ident!("param{}", j))
            });

            quote! {
                fn #reduce_fn_name(results: &mut Vec<ActionResult>) {
                    #(#param_stmts)*
                    let result = #action_fn(#(#action_params),*);
                    results.push(ActionResult::#canonical_name(result));
                }
            }
        })
        .collect()
}

fn generate_partial_eq_impl(
    ast: &AstGrammar,
    names: &NameMap,
    token_pats: &TokenPatMap,
) -> TokenStream2 {
    let token_type = Ident::new(ast.token_decl.name, Span::call_site());
    let match_actions = token_pats.iter().map(|(term, token_pat)| {
        let symbol_variant = &names[*term];
        quote! { Symbol::#symbol_variant => matches!(other, #token_pat), }
    });

    quote! {
        impl PartialEq<#token_type> for Symbol {
            fn eq(&self, other: &#token_type) -> bool{
                match self {
                    #(#match_actions)*
                    _ => false
                }
            }
        }
    }
}

fn generate_symbol_impl(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    let terminals: Vec<_> = ast
        .terminals()
        .map(|term| {
            let variant = &names[term];
            quote! { Symbol::#variant }
        })
        .collect();

    let term_names = ast.terminals().map(|term| {
        let variant = &names[term];
        let term = &term[1..(term.len() - 1)];
        quote! { Symbol::#variant => #term }
    });

    let nonterm_names = ast.nonterminals().map(|nonterm| {
        let variant = &names[nonterm];
        quote! { Symbol::#variant => #nonterm }
    });

    let end_variant = &names[EOF_TERMINAL];
    quote! {
        impl Symbol {
            fn is_terminal(&self) -> bool {
                matches!(self, #(#terminals)|*)
            }

            fn is_end(&self) -> bool {
                matches!(self, Symbol::#end_variant)
            }

            fn name(&self) -> &'static str {
                match self {
                    #(#term_names,)*
                    #(#nonterm_names,)*
                }
            }
        }
    }
}

fn generate_symbol_enum(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    let fields: Vec<_> = ast
        .terminals()
        .chain(ast.nonterminals())
        .map(|name| &names[name])
        .collect();

    quote! {
        enum Symbol {
            #(#fields),*
        }
    }
}

fn generate_action_result_enum(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    let fields: Vec<_> = ast
        .nonterminals
        .iter()
        .map(|nonterm| {
            let name = &names[nonterm.name];
            let ty = &nonterm.ty;

            quote! { #name(#ty) }
        })
        .collect();

    quote! {
        enum ActionResult {
            #(#fields),*
        }
    }
}

/// Generate a map from written down terminal/nonterminal names to "canonical"
/// names like Term0 and Nonterm1.
fn generate_name_map<'input>(ast: &AstGrammar<'input>) -> NameMap<'input> {
    let mut counter: usize = 0;
    let mut names = HashMap::new();

    for term in ast.terminals() {
        names.insert(term, format_ident!("Term{}", counter));
        counter += 1;
    }

    for nonterm in ast.nonterminals() {
        names.insert(nonterm, format_ident!("Nonterm{}", counter));
        counter += 1;
    }

    names
}

/// Generate a map from nonterminal name to the returned Rust type.
fn generate_nonterm_ty_map<'input>(ast: &'input AstGrammar<'input>) -> NontermTyMap<'input> {
    ast.nonterminals
        .iter()
        .map(|nonterminal| (nonterminal.name, &nonterminal.ty))
        .collect()
}

/// Generate a map from terminal alias to the Rust pattern that matches it.
fn generate_token_pat_map<'input>(ast: &AstGrammar<'input>) -> TokenPatMap<'input> {
    ast.token_decl
        .aliases
        .iter()
        .map(|token_alias| {
            let token_type = Ident::new(token_alias.pattern.ty, Span::call_site());
            let token_variant = Ident::new(token_alias.pattern.variant, Span::call_site());

            (token_alias.term, quote! { #token_type::#token_variant })
        })
        .collect()
}

/// Generate a map from nonterminal name + production to production ID.
fn generate_production_id_map<'input>(ast: &'input AstGrammar<'input>) -> ProductionIdMap<'input> {
    ast.productions_indexed()
        .map(|(nonterminal, production, id)| ((nonterminal.name, production), id))
        .collect()
}

impl<'input> ToTokens for AstTypeRef<'input> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            AstTypeRef::Ty(path, generics) => {
                path.to_tokens(tokens);

                if !generics.is_empty() {
                    tokens.extend(quote! {
                        < #(#generics),* >
                    });
                }
            }
            AstTypeRef::Tuple(tys) => {
                tokens.extend(quote! {
                    ( #(#tys),* )
                });
            }
        }
    }
}

impl<'input> ToTokens for AstTypePath<'input> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.is_absolute {
            tokens.extend(quote! { :: });
        }

        let segments = self
            .segments
            .iter()
            .map(|segment| Ident::new(*segment, Span::call_site()));
        tokens.extend(quote! {
            #(#segments)::*
        });
    }
}
