#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused)]

// TODO: We currently cannot parse negative numbers on line start.

use std::path::Path;

struct Lexer {
    // Holds the raw data from reading the CSV file.
    raw_input: String,
    // Just for now, we collect tokens into a vector for debugging
    cur_state: State,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum State {
    StartOfLine,
    InComment,
    OnDelimiter,
    MaybeInteger,
    InInteger,
    MaybeFloat,
    InFloat,
    InScientific,
    Invalid,
}

#[derive(Debug, PartialEq, Clone)]
enum Token<'a> {
    Integer(i64),
    Float(f64),
    Delimiter(char),
    Comment(&'a str),
    Newline,
}

impl Lexer {
    fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        // Note: Control characters are ignored throughout lexing.
        // TODO: Maybe there is a more performant way to do this
        // with a byte reader...
        let raw_input: String = std::fs::read_to_string(path)?
            .chars()
            .filter(|chr| !chr.is_control())
            .collect();
        Ok(Lexer::from_string(raw_input))
    }

    fn from_string(raw_input: String) -> Self {
        Self {
            cur_state: State::StartOfLine,
            raw_input,
        }
    }

    // fn proceed_to_new_line(&mut self, current_line: &mut usize, current_position: &mut usize) {
    //     self.parsed_tokens.push(Token::Newline);
    //     self.cur_state = State::StartOfLine;
    //     *current_line += 1;
    //     *current_position = 0;
    // }

    fn walk_file<'a>(&'a mut self) -> Vec<Token<'a>> {
        let raw_input = &self.raw_input;
        log::debug!("raw_input = {}", raw_input);
        let mut tokens = Vec::new();
        // i and j are indices that mark the slice of `raw_input`
        // that has to be parsed next (`&raw_input[i..j]`).
        // Note that incrementing j in the code below usually means we
        // accept the char as belonging to the token currently parsed.
        let mut i = 0;
        let mut j = 0;

        for (line_no, line) in raw_input.lines().enumerate() {
            log::debug!("line {}, i = {}, j = {}", line_no, i, j);
            let mut chrs = line.chars().enumerate().peekable();
            while let Some((k, chr)) = chrs.next() {
                match self.cur_state {
                    // Ordered roughly by my guess of how common these patterns
                    // are.
                    State::InInteger => match chr {
                        '0'..='9' => j += 1,

                        ' ' | ',' | '\t' | ';' => {
                            tokens.push(emit_token(&raw_input, &mut i, &mut j, self.cur_state));
                            self.cur_state = State::OnDelimiter;
                            tokens.push(emit_token(&raw_input, &mut i, &mut j, self.cur_state));
                        }

                        '.' => {
                            j += 1;
                            self.cur_state = State::InFloat;
                        }

                        _ => {
                            j += 1;
                            tokens.push(emit_token(&raw_input, &mut i, &mut j, State::Invalid));
                        }
                    },

                    State::InFloat => {}

                    State::OnDelimiter => match chr {
                        '0'..='9' => {
                            j += 1;
                            self.cur_state = State::InInteger;
                        }

                        _ => todo!(),
                    },

                    State::StartOfLine => match chr {
                        '0'..='9' => {
                            j += 1;
                            self.cur_state = State::InInteger;
                        }
                        _ => todo!(),
                    },

                    State::InScientific => {
                        //
                    }

                    State::InComment => {
                        //
                    }

                    State::MaybeInteger => {
                        //
                    }

                    State::MaybeFloat => {
                        //
                    }

                    State::Invalid => {
                        // We never put the lexer into this state, we just use
                        // it in `emit_token`.
                        unreachable!()
                    }
                }
                log::debug!(
                    "char k = {} = '{}', i = {}, j = {}, state = {:?}",
                    k,
                    chr,
                    i,
                    j,
                    self.cur_state
                );
            }
        }
        tokens
    }
}

/// Emit token based on current text slice.
/// Panics if text handed cannot be parsed according to the state.
/// This should only happen if there is an unconsidered edge case
/// or a logic error.
fn emit_token<'a>(raw_text: &'a str, i: &mut usize, j: &mut usize, state: State) -> Token<'a> {
    let fail_msg = "token emitter was handed invalid raw text";
    log::debug!("{}", &raw_text[*i..*j]);
    let token = match state {
        State::InComment => Token::Comment(&raw_text[*i..*j]),
        State::OnDelimiter => Token::Delimiter(raw_text[*i..*j].chars().nth(0).expect(&fail_msg)),
        State::MaybeInteger => todo!(),
        State::InInteger => Token::Integer(raw_text[*i..*j].parse::<i64>().expect(&fail_msg)),
        State::MaybeFloat => todo!(),
        State::InFloat => todo!(),
        State::InScientific => todo!(),
        State::StartOfLine => todo!(),
        State::Invalid => todo!(),
    };
    *i = *j;
    *j = *i + 1;
    token
}

mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_lex_integer() {
        init();
        let mut lexer = Lexer::from_string("220;210;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[0], Token::Integer(220));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }
}
