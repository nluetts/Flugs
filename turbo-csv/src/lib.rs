#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused)]

// TODO: We currently cannot parse negative numbers on line start.

use std::{ops::Index, path::Path};

const DELIMITERS: [char; 4] = [',', ' ', '\t', ';'];

struct Lexer<'a> {
    // Holds the raw data from reading the CSV file.
    raw_input: String,
    // Just for now, we collect tokens into a vector for debugging
    parsed_tokens: Vec<Token<'a>>,
    cur_state: State,
}

#[derive(PartialEq)]
enum State {
    StartOfLine,
    InComment,
    OnDelimiter,
    InInteger,
    InFloat,
    InScientific,
}

enum Token<'a> {
    Integer(f64),
    Float(f64),
    Delimiter(char),
    Comment(&'a str),
    Newline,
}

impl<'a> Lexer<'a> {
    fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        // Note: Control characters are ignored throughout lexing.
        // TODO: Maybe there is a more performant way to do this
        // with a byte reader...
        let raw_input: String = std::fs::read_to_string(path)?
            .chars()
            .filter(|chr| !chr.is_control())
            .collect();
        let cur_segment = &raw_input[0..0];
        let lexer = Self {
            cur_state: State::StartOfLine,
            parsed_tokens: Vec::new(),
            raw_input,
        };

        Ok(lexer)
    }

    fn proceed_to_new_line(&mut self, current_line: &mut usize, current_position: &mut usize) {
        self.parsed_tokens.push(Token::Newline);
        self.cur_state = State::StartOfLine;
        *current_line += 1;
        *current_position = 0;
    }

    fn walk_file(&'a mut self) {
        let raw_input = &self.raw_input;
        let mut chrs = self.raw_input.chars().enumerate().peekable();
        let mut cur_line = 1;
        let mut cur_position = 0;
        // i and j are indices that mark the slice of `raw_input`
        // that has to be parsed next (`&raw_input[i..j]`).
        let mut i = 0;
        let mut j = 0;

        while let Some((k, cur)) = chrs.next() {
            if chrs.peek().is_none() {
                // We reached the end of the file.
                match self.cur_state {
                    State::StartOfLine => {}
                    State::InComment => {
                        //
                    }
                    State::OnDelimiter => {
                        //
                    }
                    State::InInteger => {
                        //
                    }
                    State::InFloat => {
                        //
                    }
                    State::InScientific => {
                        //
                    }
                }
                return;
            }
            // This is save here, because we checked for `is_none` above.
            let (l, nxt) = unsafe { chrs.peek().unwrap_unchecked() };

            match (cur, nxt, &self.cur_state) {
                // Inside a number, current char is digit.
                ('0'..='9', _, State::InInteger) => {
                    // If we are inside a number and the current character is
                    // also a number, we can just increment the stop index.
                    j += 1;
                }
                // Inside a number, current char is delimiter.
                ((',' | ' ' | '\t' | ';'), _, State::InInteger) => {
                    // If we are inside a number and the current character is
                    // also a number, we can just increment the stop index.
                    todo!()
                }
                (_, _, State::InInteger) => {
                    //
                }
                (_, _, State::InFloat) => {
                    //
                }
                (_, _, State::InScientific) => {
                    //
                }
                (_, _, State::StartOfLine) => {
                    return;
                }
                (_, _, State::InComment) => {
                    //
                }
                (_, _, State::OnDelimiter) => {
                    //
                }
                (_, _, _) => unreachable!(),
            }
        }
    }

    // fn walk_file(&'a mut self) {
    //     let raw_input = &self.raw_input;
    //     let mut chrs = self.raw_input.chars().enumerate();
    //     let mut cur_segment = &raw_input[0..0];
    //     self.cur_line = 1;
    //     self.cur_position = 0;

    //     // Initialize current character.
    //     let mut current = chrs.next();
    //     if current.is_none() {
    //         // An empty file will return immediately.
    //         return;
    //     }

    //     loop {
    //         let next = chrs.next();
    //         self.cur_position += 1;

    //         use State as S;
    //         use Token as T;
    //         match (current, next, &self.cur_state) {
    //             // TODO: Match on specific characters everywhere, I think this is easier.

    //             // If we end up here, this is a file containing only a single
    //             // character—we do not emit a token for that.
    //             (None, _, _) => break,
    //             // We are currently lexing a comment.
    //             (Some((k, chr)), Some((_, nxt)), S::InComment) => {
    //                 match (chr, nxt) {
    //                     // End of comment, windows line end.
    //                     ('\r', '\n') => {
    //                         self.parsed_tokens.push(Token::Comment(cur_segment));
    //                         self.proceed_to_new_line();
    //                     }
    //                     // End of comment, *nix line end.
    //                     (chr, '\n') => {
    //                         let i = get_start_index(raw_input, cur_segment);
    //                         let new_segment = &raw_input[i..k + 1];
    //                         self.parsed_tokens.push(Token::Comment(new_segment));
    //                         self.proceed_to_new_line();
    //                     }
    //                     // Two chars that we accept as belonging to the comment.
    //                     (_, _) => {
    //                         let i = get_start_index(raw_input, cur_segment);
    //                         let new_segment = &raw_input[i..k + 2];
    //                         self.parsed_tokens.push(Token::Comment(new_segment));
    //                         // Skipping one position, because we handled two chars,
    //                         // see below.
    //                         self.cur_position += 1;
    //                     }
    //                 }
    //                 // We skip the current `next`, because we already handled
    //                 // two characters here.
    //                 // Note that this character will be set to the current
    //                 // character at the end of this match statement.
    //                 next = chrs.next();
    //             }
    //             (Some((k, chr)), Some((_, nxt)), S::InNumber) => {
    //                 match (chr, nxt) {
    //                     // End of number, windows line end.
    //                     ('\r', '\n') => {
    //                         let token = Token::Number(
    //                             cur_segment
    //                                 .parse::<f64>()
    //                                 .expect("CSV lexer failed to parse number."),
    //                         );
    //                         self.parsed_tokens.push(token);
    //                         self.proceed_to_new_line();
    //                     }
    //                     // End of number, *nix line end.
    //                     // The current char is a digit and needs to be included.
    //                     ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9', '\n') => {
    //                         let i = get_start_index(raw_input, cur_segment);
    //                         let new_segment = &raw_input[i..k + 1];
    //                         let token = Token::Number(
    //                             cur_segment
    //                                 .parse::<f64>()
    //                                 .expect("CSV lexer failed to parse number."),
    //                         );
    //                         self.parsed_tokens.push(token);
    //                         self.proceed_to_new_line();
    //                     }
    //                     // End of number, *nix line end.
    //                     // The current char is a dot and might be valid.
    //                     ('.', '\n') => {
    //                         let i = get_start_index(raw_input, cur_segment);
    //                         let new_segment = &raw_input[i..k + 1];
    //                         let token = Token::Number(
    //                             cur_segment
    //                                 .parse::<f64>()
    //                                 .expect("CSV lexer failed to parse number."),
    //                         );
    //                         self.parsed_tokens.push(token);
    //                         self.proceed_to_new_line();
    //                     }
    //                     // Two chars that we accept as belonging to the comment.
    //                     (_, _) => {
    //                         todo!();
    //                     }
    //                 }
    //                 // We skip the current `next`, because we already handled
    //                 // two characters here.
    //                 // Note that this character will be set to the current
    //                 // character at the end of this match statement.
    //                 next = chrs.next();
    //             }
    //             (Some(_), None, S::OnDelimiter) => todo!(),
    //             (Some(_), Some(_), S::OnDelimiter) => todo!(),
    //             (Some((_, chr)), Some((_, nxt)), S::StartOfLine) => {
    //                 if chr.is_ascii_digit() || chr == '.' {
    //                     self.cur_state = S::InNumber;
    //                     cur_segment = &raw_input[0..1];
    //                 } else if chr.is_whitespace() {
    //                     // Nothing to do here, just ignoring, keeping the
    //                     // "StartOfLine" state.
    //                 } else {
    //                     // All lines that do not start with whitespace or an
    //                     // ASCII digit are considered comments.
    //                     self.cur_state = S::InComment;
    //                     cur_segment = &raw_input[0..1];
    //                 }
    //             }
    //             (Some((k, chr)), None, S::InNumber(segment)) => {
    //                 if chr.is_ascii_digit() {
    //                     let i = get_start_index(raw_input, segment);
    //                     let token = Token::Number(
    //                         raw_input[i..k + 1]
    //                             .parse::<f64>()
    //                             .expect("CSV lexer failed to parse number."),
    //                     );
    //                     self.parsed_tokens.push(token);
    //                 } else if chr.is_whitespace() {
    //                     // We can ignore whitespace here.
    //                 } else {
    //                     log::warn!("unable to parse last character \"{chr}\"")
    //                 }
    //             }
    //             // Last line, first (and last) character.
    //             (Some((i, chr)), None, State::StartOfLine) => {
    //                 if chr.is_ascii_digit() {
    //                     let num = raw_input[i..i]
    //                         .parse::<f64>()
    //                         .expect("failed to parse ASCII digit as float");
    //                     self.parsed_tokens.push(Token::Number(num));
    //                 } else {
    //                     self.parsed_tokens.push(Token::Comment(&raw_input[i..i]));
    //                 }
    //             }
    //             (Some((_, chr)), None, S::InComment(segment)) => {
    //                 if !chr.is_whitespace() {
    //                     let (k, l) = get_start_index(raw_input, segment);
    //                     let token = Token::Comment(&raw_input[k..l + 1]);
    //                 }
    //             }
    //         }

    //         current = next;
    //         if current.is_none() {
    //             return;
    //         }
    //     }
    // }
    //
}

/// Get the start index of a slice relative to its parent a str.
fn get_start_index(parent: &str, child: &str) -> usize {
    let start = child.as_ptr() as usize - parent.as_ptr() as usize;
    start
}

// mod test {
//     use super::*;

//     #[test]
//     fn test_create_lexer() {
//         let lex = Lexer::from_path(Path::new("./test/test_1.csv"));
//         assert!(lex.is_ok());
//     }

//     // #[test]
//     /// Test that indices from subslice within slice are correctly
//     /// recovered by `get_start_end`.
//     // fn test_get_start_end() {
//     //     let input = "0, 220".to_string();
//     //     let input_ref = &input;
//     //     let segment = &input_ref[3..5];
//     //     let state = State::InNumber(NumState::InInteger);
//     //     let cur = Some((5, '0'));

//     //     match (cur, state) {
//     //         (Some((i, chr)), State::InNumber(NumState::InInteger)) => {
//     //             if chr.is_ascii_digit() {
//     //                 let k = get_start_index(input_ref, segment);
//     //                 assert_eq!(221.0, input_ref[k..=i].parse::<f64>().unwrap());
//     //             } else {
//     //                 unreachable!()
//     //             }
//     //         }
//     //         _ => unreachable!(),
//     //     }
//     // }
// }
