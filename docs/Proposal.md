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

LL(1) is parsing algorithm which stands for "Left-to-right Left-most derivation
with 1 token of lookahead". LL(1) parsers have a strong formal foundation and
are similar in power to recursive descent parsers.

## Additional Details
### Key Components
* Parser for the input grammar file
* Core algorithms to create the LL(1) parse table
* Rust parser code generation algorithm

### Testing
* Tests of the grammar file parser
  * Input: string of the grammar file
* Tests of the parse table generation
  * Input: Internal representation of the grammar
* Integration tests
  * Run the rust parser code to make sure it parses the specified grammar
  * Test crate specifies a bunch of parsers via grammar files
  * Test crate build script generates parser code
  * Run test crate to run the generated parsers on test inputs

### MVP
* Parses grammar files
* Generates Rust parser code according to the grammar file
* Just do parsing, no default lexer like in LALRPOP
* Only support basic BNF, not extended BNF
* Grammar files contain:
  * Rust imports
  * Declaration of token source
  * Nonterminals
    * Nonterminals have associated action code and return type

### Stretch Goals
* Integrate default lexer
* Support extended BNF

### Expected Checkpoint Progress
* Functional grammar file parser
* Start of parse table generation (basic analysis of the grammar)