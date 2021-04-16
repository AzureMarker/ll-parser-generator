#[macro_use]
extern crate lalrpop_util;

use crate::ast::{AstGrammar, AstTypePath, AstTypeRef};
use crate::ll_table_gen::{compute_first, compute_follow, compute_nullable, compute_parse_table};
use crate::parsing::parse;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;

#[cfg(test)]
#[macro_use]
mod grammar_tests;

mod ast;
mod lexer;
mod ll_table_gen;
mod parsing;

type NameMap<'input> = HashMap<&'input str, Ident>;

#[proc_macro]
pub fn ll_parser(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let ast = parse(&input).unwrap(); // TODO: return error

    // Compute LL(1) parse table
    let nullable = compute_nullable(&ast);
    let first = compute_first(&ast, &nullable);
    let follow = compute_follow(&ast, &nullable, &first);
    let parse_table = compute_parse_table(&ast, &nullable, &first, &follow);

    // Compute canonical names for terminals and nonterminals
    let name_map = generate_name_map(&ast);

    // Create data structures and functions
    let action_result_enum = generate_action_result_enum(&ast, &name_map);
    let symbols_enum = generate_symbol_enum(&ast, &name_map);
    let symbol_impl = generate_symbol_impl(&ast, &name_map);

    // Generate output code
    let imports = ast.imports;
    let token_ty = ast.token_decl.name;
    let output = quote! {
        // TODO: allow for user-specified module name
        #(use #imports;)*

        enum SymbolOrReduction {
            Symbol(Symbol),
            Reduction(fn(&mut std::vec::Vec<ActionResult>)),
        }

        #action_result_enum

        #[derive(Debug)]
        pub enum ParseError {
            UnexpectedEOF,
            UnrecognizedToken {
                expected: std::vec::Vec<&'static str>,
                found: #token_ty,
            },
        }

        #symbols_enum
        #symbol_impl
    };
    output.into()
}

fn generate_symbol_impl(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    let terminals: Vec<_> = ast
        .terminals()
        .map(|term| format_ident!("Symbol::{}", names[term]))
        .collect();

    let term_names = ast.terminals().map(|term| {
        let pattern = format_ident!("Symbol::{}", names[term]);
        let term = &term[1..(term.len() - 1)];
        quote! {
            #pattern => #term
        }
    });

    let nonterm_names = ast.nonterminals().map(|nonterm| {
        let pattern = format_ident!("Symbol::{}", names[nonterm]);
        quote! {
            #pattern => #nonterm
        }
    });

    quote! {
        impl Symbol {
            fn is_terminal(&self) -> bool {
                matches!(self, #(#terminals)|*)
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

        let segments = &self.segments;
        tokens.extend(quote! {
            #(#segments)::*
        });
    }
}
