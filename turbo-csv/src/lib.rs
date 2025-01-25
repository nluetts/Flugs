#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused)]

use std::{ops::Index, path::Path};

const DELIMITERS: [char; 4] = [',', ' ', '\t', ';'];

struct Lexer<'a> {
    // Holds the raw data from reading the CSV file.
    raw_input: String,
    // Just for now, we collect tokens into a vector for debugging
    parsed_tokens: Vec<Token<'a>>,
    current_state: State<'a>,
}

#[derive(PartialEq)]
enum State<'a> {
    StartOfLine,
    InComment(&'a str),
    OnDelimiter(&'a char),
    InNumber(&'a str),
}

enum Token<'a> {
    Comment(&'a str),
    Number(f64),
    Delimiter(char),
    Newline,
}

impl<'a> Lexer<'a> {
    fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let raw_input = std::fs::read_to_string(path)?;
        Ok(Self {
            current_state: State::StartOfLine,
            parsed_tokens: Vec::new(),
            raw_input,
        })
    }

    fn walk_file(&'a mut self) {
        let mut chrs = self.raw_input.chars().enumerate();
        let raw_input = &self.raw_input;
        let mut line = 1;
        let mut char_line = 1;

        // Initialize current character
        let mut cur = chrs.next();
        if cur.is_none() {
            return;
        }

        loop {
            let nxt = chrs.next();

            use State as S;
            match (cur, nxt, &self.current_state) {
                // If this is a file with only a single character, we
                // do not emit a token for that.
                (None, _, _) => break,
                (Some(_), Some(_), S::InComment(_)) => todo!(),
                (Some(_), Some(_), S::InNumber(_)) => todo!(),
                (Some(_), None, S::OnDelimiter(_)) => todo!(),
                (Some(_), Some(_), S::OnDelimiter(_)) => todo!(),
                (Some((_, chr)), Some((_, nxt)), S::StartOfLine) => {
                    if chr.is_ascii_digit() || chr == '.' {
                        self.current_state = S::InNumber(&raw_input[0..1]);
                    } else if chr.is_whitespace() {
                        // Nothing to do here, just ignoring, keeping the
                        // "StartOfLine" state.
                    } else {
                        // All lines that do not start with whitespace or an
                        // ASCII digit are considered comments.
                        self.current_state = S::InComment(&raw_input[0..1]);
                    }
                }
                (Some((i, chr)), None, S::InNumber(segment)) => {
                    if chr.is_ascii_digit() {
                        let (k, l) = get_start_end(raw_input, segment);
                        let token = Token::Number(
                            raw_input[k..l + 1]
                                .parse::<f64>()
                                .expect("CSV lexer failed to parse number."),
                        );
                        self.parsed_tokens.push(token);
                    } else if chr.is_whitespace() {
                        // We can ignore whitespace here.
                    } else {
                        log::warn!("unable to parse last character \"{chr}\"")
                    }
                }
                // Last line, first (and last) character.
                (Some((i, chr)), None, State::StartOfLine) => {
                    if chr.is_ascii_digit() {
                        let num = raw_input[i..i]
                            .parse::<f64>()
                            .expect("failed to parse ASCII digit as float");
                        self.parsed_tokens.push(Token::Number(num));
                    } else {
                        self.parsed_tokens.push(Token::Comment(&raw_input[i..i]));
                    }
                }
                (Some((_, chr)), None, S::InComment(segment)) => {
                    if !chr.is_whitespace() {
                        let (k, l) = get_start_end(raw_input, segment);
                        let token = Token::Comment(&raw_input[k..l + 1]);
                    }
                }
            }

            cur = nxt;
            if cur.is_none() {
                return;
            }
        }
    }
}

/// Get the start and end index of a slice from a string.
/// `child` must be a slice from `parent`, otherwise the function
/// yields nonsense.
fn get_start_end(parent: &str, child: &str) -> (usize, usize) {
    let start = child.as_ptr() as usize - parent.as_ptr() as usize;
    let end = start + child.len();
    (start, end)
}

mod test {
    use super::*;

    #[test]
    fn test_create_lexer() {
        let lex = Lexer::from_path(Path::new("./test/test_1.csv"));
        assert!(lex.is_ok());
    }

    #[test]
    /// Test that indices from subslice within slice are correctly
    /// recovered by `get_start_end`.
    fn test_get_start_end() {
        let input = "0, 220".to_string();
        let input_ref = &input;
        let state = State::InNumber(&input_ref[3..5]);
        let cur = Some((5, '0'));

        match (cur, state) {
            (Some((i, chr)), State::InNumber(segment)) => {
                if chr.is_ascii_digit() {
                    let (k, l) = get_start_end(input_ref, segment);
                    assert_eq!(221.0, input_ref[k..=l].parse::<f64>().unwrap());
                } else {
                    unreachable!()
                }
            }
            _ => unreachable!(),
        }
    }
}
