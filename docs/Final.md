# LL(1) Parser Generator
Team members:
- Mark Drobnak
- Amar Shehzad

## Summary Description
This project is to create an LL(1) parser generator. A parser generator takes in
a file which describes the grammar of a language and how it should be parsed.
From this file, a parser implementation is generated. Given a formal description
of a language, otherwise known as a grammar, a parser will transform a sequence
of tokens from that grammar into a representation more usable to the program
(usually a tree-like data structure). Parsers are most often used in the first
few stages of a compiler or static analysis tool.

## Demo
```rust
mod parser {
  ll_parser_generator::ll_parser! {
        use crate::Token;
        use crate::AstTerm;

        token Token {
            "(" = Token::LParen,
            ")" = Token::RParen,
            "NUMBER" = Token::Number,
        }

        grammar;

        pub Term: AstTerm = {
            <n:Number> => n,
            "(" <t:Term> ")" => AstTerm::Paren(Box::new(t)),
        };

        Number: AstTerm = "NUMBER" => AstTerm::Number;
    }
}

fn main() {
    let input = "(1)";
    let lexer = Lexer::new(input);
    let result = parser::parse(lexer);

    assert_eq!(result, Ok(AstTerm::Paren(Box::new(AstTerm::Number))));
}
```

## Major Work Items Completed
* Parser (and lexer) for the input grammar file.
* Algorithms which generate the LL(1) parse table (Nullable, First, Follow,
  etc).
* Code generation of the output LL(1) parser (via proc-macro).
* Lots of tests.

## Crates used
* `logos`: https://crates.io/crates/logos
    * A lexer-generator that operates via a derive macro.
* `lalrpop`: https://crates.io/crates/lalrpop
    * Rust LR(1) parser-generator framework that emits Rust code.
* `proc_macro`: https://doc.rust-lang.org/proc_macro
    * A compiler crate which supports code generation at compile time
      (procedural macros). This crate is only usable in proc-macro crates.
* `proc_macro2`: https://crates.io/crates/proc-macro2
    * A wrapper around the compiler's `proc_macro` crate which can be used in
      libraries (like `quote`).
* `quote`: https://crates.io/crates/quote/
     * Provides a macro_rules-like syntax for generating code in procedural
       macros.

## Project Structure
* `lexer.rs`
    * Defines the grammar file lexer using Logos.
* `parser.lalrpop`
    * Defines the grammar file parser using LALRPOP.
* `ast.rs`
    * Defines the Abstract Syntax Tree (AST) of the parsed grammar files.
* `ll_table_gen.rs`
    * Defines the algorithms used for LL(1) parsing, including Nullable, First,
      Follow, etc.
* `lib.rs`
    * Defines the procedural macro for the parser. These macros are essentially
      functions that take in a token stream and produce a new token stream.
    * Takes in an input stream, converts it to an AST via the grammar file lexer
      and parser, computes parse table information and generates output code.

## "Rusty" Code Examples
### AST
The AST references the grammar file source via `&'input str` references.
```rust
#[derive(Debug, Eq, PartialEq)]
pub struct AstGrammar<'input> {
    pub imports: Vec<&'input str>,
    pub token_decl: AstTokenDecl<'input>,
    pub nonterminals: Vec<AstNonterminal<'input>>,
}
```

### Grammar Tests
We often utilized macros when writing tests. Grammar tests are done via a
`grammar_test` macro. This macro handles parsing the grammar and checking the
output against the provided `AstGrammar` value.
```rust
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
            /* ... */
        },
        nonterminals: Vec::new(),
    }
}
```

### Using Quote in Procedural Macro
As part of generating the LL(1) parser via the proc-macro, we need to define a
`Symbol` enum which holds all the symbols used in the parser. This enum has a
variant for each terminal and nonterminal. Here's an example:
```rust
enum Symbol {
    Term0,    // (
    Term1,    // )
    Term2,    // NUMBER
    Nonterm0, // Term
    Nonterm1, // Number
}
```

We have a canonical name for each terminal and nonterminal (ex. "NUMBER" is
"Term2"). To generate the `Symbol` enum, we convert each terminal and
nonterminal into their canonical name and output them as enum variant names
using the `quote` macro:
```rust
fn generate_symbol_enum(ast: &AstGrammar, names: &NameMap) -> TokenStream2 {
    let fields = ast
        .terminals()
        .chain(ast.nonterminals())
        .map(|name| &names[name]);

    quote! {
        enum Symbol {
            #(#fields),*
        }
    }
}
```

## Challenges Due to Rust?
There were no issues with expressing our intentions in Rust. In fact, by using
certain rust features we found it easier to implement the algorithms. For
example, in our grammar tests, we created a macro that, as described above,
made testing easier for us by making it so that we did not need to repeat work.
Another example is the `quote` macro which we used to generate the token stream
output. All in all, Rust actually made our lives a lot easier.

## Conclusion
A parser-generator has a lot of moving parts to it. Different algorithms
describe different portions of it, and implementing all of these helped us learn
more about Rust. As described above, macros were heavily used throughout.

This was my (Amar) first exposure to macros, and I have gained a new
appreciation for them and how they can help save time writing code. We also
extensively used iterators, ex. in `lib.rs`, and writing the code for
different iterators has helped me understand them a lot more. 