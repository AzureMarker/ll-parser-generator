use crate::ast::AstGrammar;
use crate::lexer::{StatefulLexer, Token};
use lalrpop_util::ParseError;
use logos::Logos;
use std::ops::Range;

lalrpop_mod!(
    #[allow(clippy::all)]
    pub(crate) parser
);

/// Parse a grammar. If there are errors during parsing, the error will be returned.
pub fn parse(grammar_str: &str) -> Result<AstGrammar, ParseError<usize, Token, Range<usize>>> {
    let lexer = StatefulLexer::new(Token::lexer(grammar_str));

    match parser::GrammarParser::new().parse(lexer) {
        Ok(parsed_grammar) => Ok(parsed_grammar),
        Err(ParseError::InvalidToken { location }) => {
            let (line, col) = index_to_line_col(grammar_str, location);
            eprintln!("Invalid token at line {}, column {}", line, col);
            Err(ParseError::InvalidToken { location })
        }
        Err(ParseError::UnrecognizedToken {
            token: (lspan, token, _rspan),
            expected,
        }) => {
            let (line, col) = index_to_line_col(grammar_str, lspan);
            eprintln!(
                "Unrecognized token '{:?}' at line {}, column {}, expected [{}]",
                token,
                line,
                col,
                expected.join(", ")
            );
            Err(ParseError::UnrecognizedToken {
                token: (lspan, token, _rspan),
                expected,
            })
        }
        Err(ParseError::UnrecognizedEOF { location, expected }) => {
            let (line, col) = index_to_line_col(grammar_str, location);
            eprintln!(
                "Unexpected EOF at line {}, column {}, expected [{}]",
                line,
                col,
                expected.join(", ")
            );
            Err(ParseError::UnrecognizedEOF { location, expected })
        }
        Err(ParseError::ExtraToken {
            token: (lspan, token, _rspan),
        }) => {
            let (line, col) = index_to_line_col(grammar_str, lspan);
            eprintln!(
                "Unexpected extra token '{:?}' at line {}, column {}",
                token, line, col
            );
            Err(ParseError::ExtraToken {
                token: (lspan, token, _rspan),
            })
        }
        Err(ParseError::User { error }) => {
            let token = &grammar_str[error.clone()];
            let (line, col) = index_to_line_col(grammar_str, error.start);
            eprintln!("Invalid token '{}' at line {}, column {}", token, line, col);
            Err(ParseError::User { error })
        }
    }
}

/// Convert an index of the file into a line and column index
fn index_to_line_col(file_str: &str, index: usize) -> (usize, usize) {
    let line = file_str
        .chars()
        .enumerate()
        .take_while(|(i, _)| *i != index)
        .filter(|(_, c)| *c == '\n')
        .count()
        + 1;
    let column = file_str[0..index]
        .chars()
        .rev()
        .take_while(|c| *c != '\n')
        .count()
        + 1;

    (line, column)
}
