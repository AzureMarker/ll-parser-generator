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

## Checkpoint Progress Summary
* Created the lexer using the Logos lexer-generator library.
* Created the grammar file parser using the LALRPOP parser-generator library.
* The grammar file parser builds an AST from the input.
* Started implementing LL(1) algorithms (Nullable and First)

## Remaining Work
* Finish implementing LL(1) algorithms (Follow and Action table)
* Generate Rust code
    * First manually write an LL(1) parser, then write the code to generate
      the parser based off of that prototype.
* Add procedural macro which takes the grammar and outputs the parser generator
  Rust code:
  ```rust
  ll_parser! {
      use crate::lexer::Token;
      
      token Token {
          "(" => Token::LParen,
          ")" => Token::RParen,
      }
      
      grammar;
      
      pub Start: () = "(" ")" => ();
  }
  ```
    * Alternatively, could have the user define the grammar in a seperate file
      which is read by a build script (build.rs). This is how LALRPOP works.
* Add tests :)

## Additional Details
### Project Structure
* `lexer.rs`
    * Defines the lexer using Logos
* `parser.lalrpop`
    * Defines the parser using LALRPOP
* `ast.rs`
    * Defines the Abstract Syntax Tree (AST) of the parsed grammar files
* `ll_table_gen.rs`
    * Defines the algorithms used for LL(1) parsing, including Nullable, First,
      Follow, etc

### Crates used
* Logos: https://crates.io/crates/logos
    * A lexer-generator that operates via a derive macro.
* LALRPOP: https://crates.io/crates/lalrpop
    * Rust LR(1) parser-generator framework that emits Rust code.
