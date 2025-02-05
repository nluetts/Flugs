#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused)]

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
    InInteger,
    MaybeFloat,
    MaybeScientific,
    InFloat,
    InScientific,
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

    fn walk_file<'a>(&'a mut self) -> Vec<Token<'a>> {
        let raw_input = &self.raw_input;
        log::debug!("raw_input = {}", raw_input);
        let mut tokens = Vec::new();
        // i and j are indices that mark the slice of `raw_input`
        // that has to be parsed next (`&raw_input[i..j]`).
        // Note that incrementing j in the code below usually means we
        // accept the char as belonging to the token currently parsed.

        for (line_no, line) in raw_input.lines().enumerate() {
            log::debug!("line {}: '{}'", line_no, line);
            if line_no > 0 {
                tokens.push(Token::Newline);
            }
            self.cur_state = State::StartOfLine;
            let mut chrs = line.chars().enumerate().peekable();
            let mut i = 0;
            while let Some((j, chr)) = chrs.next() {
                match self.cur_state {
                    // Ordered roughly by my guess of how common these patterns
                    // are.
                    State::InInteger => match chr {
                        '0'..='9' => {
                            // Nothing to do, the counter will advance;
                        }

                        ' ' | ',' | '\t' | ';' => {
                            // We have to emit the previous integer token.
                            tokens.push(emit_token(&line, &mut i, j, self.cur_state));
                            self.cur_state = State::OnDelimiter;
                            // tokens.push(emit_token(&line, &mut i, j + 1, self.cur_state));
                        }

                        '.' => {
                            self.cur_state = State::InFloat;
                        }

                        _ => {
                            invalid(&line, i, j, line_no, self.cur_state);
                            break;
                        }
                    },

                    State::InFloat => match chr {
                        '0'..='9' => {
                            // Nothing to do, the counter will advance;
                        }

                        ' ' | ',' | '\t' | ';' => {
                            // We have to emit the previous integer token.
                            tokens.push(emit_token(&line, &mut i, j, self.cur_state));
                            self.cur_state = State::OnDelimiter;
                            // tokens.push(emit_token(&line, &mut i, j + 1, self.cur_state));
                        }

                        'e' | 'E' => match chrs.peek() {
                            Some((_, chr)) => match chr {
                                '0'..='9' => {
                                    self.cur_state = State::InScientific;
                                }
                                '-' | '+' => self.cur_state = State::MaybeScientific,
                                _ => {
                                    invalid(&line, i, j, line_no, self.cur_state);
                                }
                            },
                            None => invalid(&line, i, j, line_no, self.cur_state),
                        },

                        _ => {
                            invalid(&line, i, j, line_no, self.cur_state);
                            break;
                        }
                    },

                    State::OnDelimiter => match chr {
                        ' ' | ',' | '\t' | ';' => {
                            tokens.push(emit_token(&line, &mut i, j, self.cur_state));
                        }

                        '0'..='9' => {
                            self.cur_state = State::InInteger;
                        }

                        '+' | '-' => {
                            match chrs.peek() {
                                Some((_, '0'..'9')) => self.cur_state = State::InInteger,
                                Some((_, '.')) => self.cur_state = State::MaybeFloat,
                                Some(_) => {
                                    invalid(line, i, j, line_no, self.cur_state);
                                    break;
                                }
                                None => {
                                    // If we find a + or - as last char after a delimiter,
                                    // we just ignore it and emit a warning.
                                    log::warn!("ignoring trailing {} in line {}", chr, line_no);
                                }
                            }
                        }

                        _ => {
                            invalid(line, i, j, line_no, self.cur_state);
                            break;
                        }
                    },

                    State::StartOfLine => match chr {
                        '0'..='9' => {
                            self.cur_state = State::InInteger;
                        }
                        // With this we trim trailing whitespace.
                        ' ' | '\t' => (),
                        '+' | '-' => {
                            match chrs.peek() {
                                Some((_, '0'..'9')) => self.cur_state = State::InInteger,
                                Some((_, '.')) => self.cur_state = State::MaybeFloat,
                                Some(_) => self.cur_state = State::InComment,
                                None => {
                                    // If the line contains just a single + or
                                    // -, we treat it like a comment.
                                    emit_token(line, &mut i, j, State::InComment);
                                }
                            }
                        }
                        '.' => {
                            self.cur_state = State::MaybeFloat;
                        }
                        ',' | ';' => {
                            self.cur_state = State::OnDelimiter;
                            tokens.push(emit_token(&line, &mut i, j + 1, self.cur_state));
                        }
                        _ => {
                            self.cur_state = State::InComment;
                        }
                    },

                    State::InScientific => match chr {
                        '0'..='9' => {
                            self.cur_state = State::InScientific;
                        }

                        ' ' | ',' | '\t' | ';' => {
                            // We have to emit the previous integer token.
                            tokens.push(emit_token(&line, &mut i, j, self.cur_state));
                            self.cur_state = State::OnDelimiter;
                            // tokens.push(emit_token(&line, &mut i, j + 1, self.cur_state));
                        }
                        _ => {
                            invalid(line, i, j, line_no, State::InFloat);
                            break;
                        }
                    },

                    State::InComment => match chr {
                        // If we are in a comment, we accept all further
                        // characters (the line end will be recognized
                        // automatically by the outer loop).
                        _ => (),
                    },

                    State::MaybeFloat => match chr {
                        '0'..'9' => self.cur_state = State::InFloat,
                        // We stay in state MaybeFloat and check next character
                        // in next iteration.
                        '.' => (),
                        'e' | 'E' => self.cur_state = State::MaybeScientific,
                        _ => {
                            invalid(line, i, j, line_no, State::InFloat);
                            break;
                        }
                    },

                    State::MaybeScientific => match chr {
                        '0'..='9' => self.cur_state = State::InScientific,
                        '-' | '+' => (),
                        _ => {
                            invalid(line, i, j, line_no, State::InFloat);
                            break;
                        }
                    },
                }
                log::debug!(
                    "char '{}', i = {}, j = {}, state = {:?}",
                    chr,
                    i,
                    j,
                    self.cur_state
                );

                // If this is the end of the line, we maybe need to emit the
                // current item.
                if chrs.peek().is_none() {
                    match self.cur_state {
                        State::InInteger
                        | State::InFloat
                        | State::InScientific
                        | State::OnDelimiter
                        | State::InComment => {
                            tokens.push(emit_token(&line, &mut i, j + 1, self.cur_state));
                        }
                        State::MaybeFloat | State::MaybeScientific => {
                            invalid(line, i, j, line_no, State::InFloat);
                        }
                        State::StartOfLine => (),
                    };
                };
            }
        }
        tokens
    }
}

/// Emit token based on current text slice.
/// Panics if text handed cannot be parsed according to the state.
/// This should only happen if there is an unconsidered edge case
/// or a logic error.
fn emit_token<'a>(raw_text: &'a str, i: &mut usize, j: usize, state: State) -> Token<'a> {
    let fail_msg = "token emitter was handed invalid raw text";
    log::debug!(
        "emitting token for {}, i = {}, j = {}",
        &raw_text[*i..j],
        i,
        j
    );
    let token = match state {
        State::InComment => Token::Comment(&raw_text[*i..j]),
        State::OnDelimiter => Token::Delimiter(raw_text[*i..j].chars().nth(0).expect(&fail_msg)),
        State::InInteger => Token::Integer(raw_text[*i..j].parse::<i64>().expect(&fail_msg)),
        State::InFloat => Token::Float(raw_text[*i..j].parse::<f64>().expect(&fail_msg)),
        State::InScientific => Token::Float(raw_text[*i..j].parse::<f64>().expect(&fail_msg)),
        // MaybeStates should never emit a token!
        // StartOfLine is handeled in `walk_file` separately.
        State::MaybeFloat | State::MaybeScientific | State::StartOfLine => unreachable!(),
    };
    *i = j;
    token
}

fn invalid(raw_text: &str, i: usize, j: usize, line_no: usize, state: State) {
    let parse_as = match state {
        State::OnDelimiter => "delimiter",
        State::InInteger => "integer",
        State::InFloat | State::InScientific => "float",
        // This function should never be used in these states.
        State::StartOfLine | State::InComment | State::MaybeFloat | State::MaybeScientific => {
            unreachable!()
        }
    };
    log::warn!(
        "unable to parse '{}' in line {}, position {} as {}, skipping line",
        &raw_text[i..j + 1],
        line_no + 1,
        j + 1,
        parse_as,
    );
    let aux_msg = format!("{:~>1$}", "^", 18 + j - i);
    log::warn!("{}", aux_msg);
}

mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_lex_integer_single_line() {
        init();
        let mut lexer = Lexer::from_string("220;210;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[0], Token::Integer(220));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_negative_integer_single_line() {
        init();
        let mut lexer = Lexer::from_string("+220;-210;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[0], Token::Integer(220));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Integer(-210));
    }

    #[test]
    fn test_lex_integer_single_line_invalid() {
        init();
        let mut lexer = Lexer::from_string("220;231390823a;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        // assert_eq!(tokens[0], Token::Invalid(""));
    }

    #[test]
    fn test_lex_integer_multi_line() {
        init();
        let mut lexer = Lexer::from_string("220;210;\n152;62;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[2], Token::Integer(210));
        assert_eq!(tokens[3], Token::Delimiter(';'));
        assert_eq!(tokens[4], Token::Newline);
    }

    #[test]
    fn test_lex_float_single_line() {
        init();
        let mut lexer = Lexer::from_string("22.0;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[0], Token::Float(22.0));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_negative_float_single_line() {
        init();
        let mut lexer = Lexer::from_string("-22.0;+2.10;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[0], Token::Float(-22.0));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Float(2.10));

        let mut lexer = Lexer::from_string("+.220;-.10;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[0], Token::Float(0.22));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Float(-0.1));
    }

    #[test]
    fn test_lex_float_multi_line() {
        init();
        let mut lexer = Lexer::from_string("22.0;231340298.0;\n0.00023;1.0;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[4], Token::Newline);
        assert_eq!(tokens[5], Token::Float(0.00023));
    }

    #[test]
    fn test_lex_scientific_single_line() {
        init();
        let mut lexer = Lexer::from_string("22.0e-1;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        assert_eq!(tokens[0], Token::Float(2.20));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_scientific_multi_line() {
        init();
        let mut lexer = Lexer::from_string("22.e-1;\n0.0e-3;;;".into());

        let tokens = lexer.walk_file();
        log::debug!("tokens: {:?}", &tokens);
        let mut tokens = tokens.into_iter();
        assert_eq!(tokens.next(), Some(Token::Float(2.20)));
        assert_eq!(tokens.next(), Some(Token::Delimiter(';')));
        assert_eq!(tokens.next(), Some(Token::Newline));
        assert_eq!(tokens.next(), Some(Token::Float(0.0)));
    }
}
