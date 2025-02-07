#![warn(clippy::all, rust_2018_idioms)]

use std::{collections::HashMap, path::Path};

pub struct Parser {
    lexer: Lexer,
}

struct Lexer {
    // Holds the raw data from reading the CSV file.
    raw_input: String,
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

impl Parser {
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        Ok(Self {
            lexer: Lexer::from_path(path)?,
        })
    }

    #[allow(unused)]
    fn from_string(raw_input: String) -> Self {
        Self {
            lexer: Lexer::from_string(raw_input),
        }
    }

    pub fn parse_as_floats(mut self) -> (String, Vec<Vec<f64>>) {
        // We collect columns into this vector.
        let mut data: Vec<Vec<f64>> = Vec::new();
        let mut comments: Vec<&str> = Vec::new();

        // These variables are used to keep track of the state of the parser.
        let mut delimiter: Option<Token<'_>> = None;
        let mut max_column_idx = 0;
        let mut current_column_idx = 0;

        // For each line, we collect numbers in a this hashmap which maps from
        // column index to number. This way, we can fill missing entries in
        // columns which did not get a value from the current line (with NaN).
        let mut current_row: HashMap<usize, f64> = HashMap::with_capacity(10);

        let mut tokens = self.lexer.walk_file().into_iter();
        let mut line_valid = true;
        while let Some(tok) = tokens.next() {
            // dbg!("##################");
            // dbg!(
            //     &delimiter,
            //     &max_column_idx,
            //     &current_column_idx,
            //     &current_row,
            //     &result
            // );
            match tok {
                Token::Integer(x) => {
                    current_row.insert(current_column_idx, x as f64);
                }
                Token::Float(x) => {
                    current_row.insert(current_column_idx, x);
                }
                Token::Delimiter(c) => match delimiter {
                    Some(Token::Delimiter(ld)) => {
                        // If we already saw this delimiter, we assume a new
                        // column starts here.
                        if ld == c {
                            current_column_idx += 1;
                            // If the current column index goes beyond the
                            // currently known maximum number of columns, we
                            // have to add another column.
                            if current_column_idx > max_column_idx {
                                max_column_idx = current_column_idx;
                            }
                        } else {
                            // If we already saw a delimiter and the current one
                            // is different, we reject the current line.
                            log::warn!("mixed delimiters found, skipping line");
                            loop {
                                match tokens.next() {
                                    Some(Token::Newline) | None => {
                                        line_valid = false;
                                        break;
                                    }
                                    Some(_) => {}
                                }
                            }
                            current_column_idx = 0;
                            current_row.clear();
                        }
                    }
                    None => {
                        delimiter = Some(Token::Delimiter(c));
                        current_column_idx += 1;
                    }
                    Some(_) => unreachable!(),
                },
                Token::Comment(c) => comments.push(c),
                Token::Newline => {
                    if !current_row.is_empty() && line_valid {
                        #[allow(clippy::needless_range_loop)]
                        for col_idx in 0..=max_column_idx {
                            if data.get(col_idx).is_none() {
                                add_column(&mut data);
                            }
                        }
                        #[allow(clippy::needless_range_loop)]
                        for col_idx in 0..=max_column_idx {
                            let value = current_row.get(&col_idx).unwrap_or(&f64::NAN);
                            data[col_idx].push(*value);
                        }
                    }
                    current_column_idx = 0;
                    current_row.clear();
                    line_valid = true;
                }
            }
        }
        let comments = comments.join("\n");
        (comments, data)
    }
}

fn add_column(result: &mut Vec<Vec<f64>>) {
    if let Some(col) = result.first() {
        let n_rows = col.len();
        // If we already have columns, we add a
        // new column and fill it with NaN so all
        // columns have the same length.
        result.push(vec![f64::NAN; n_rows])
    } else {
        // If there is no column yet, we add one.
        result.push(Vec::new());
    }
}

impl Lexer {
    fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        // Note: Control characters are ignored throughout lexing.
        // TODO: Maybe there is a more performant way to do this
        // with a byte reader...
        let raw_input: String = std::fs::read_to_string(path)?.chars().collect();
        Ok(Lexer::from_string(raw_input))
    }

    fn from_string(raw_input: String) -> Self {
        Self { raw_input }
    }

    fn walk_file(&mut self) -> Vec<Token<'_>> {
        let raw_input = &self.raw_input;

        // DEBUG: This parallel version does not seem to keep line order correctly.
        // Sometimes the tests pass, sometimes not. Race condition?
        // use rayon::prelude::*;
        // let tokens = self
        //     .raw_input
        //     .lines()
        //     .enumerate()
        //     .par_bridge()
        //     .into_par_iter()
        //     .fold(
        //         || Vec::new(),
        //         |mut tokens, (line_no, line)| {
        //             self.lex_line(line_no, line, &mut tokens);
        //             tokens
        //         },
        //     )
        //     .reduce(
        //         || Vec::new(),
        //         |mut a, mut b| {
        //             a.append(&mut b);
        //             a
        //         },
        //     );
        // tokens

        let mut tokens = Vec::new();

        for (line_no, line) in raw_input.lines().enumerate() {
            self.lex_line(line_no, line, &mut tokens);
        }
        dbg!(&tokens);
        tokens
    }

    fn lex_line<'a>(&'a self, line_no: usize, line: &'a str, tokens: &mut Vec<Token<'a>>) {
        if line_no > 0 {
            tokens.push(Token::Newline);
        }

        let mut cur_state = State::StartOfLine;
        let mut chrs = line.chars().enumerate().peekable();
        // `i` and `j` are indices that mark the slice of `line` that has to be
        // parsed next (`&line[i..j]`).
        let mut i = 0; // TODO: This counter is currently handled poorly: sometimes it is
                       // advanved by the `emit_token` function, sometimes directly in the
                       // `while let` loop. This should be unified.
        while let Some((j, chr)) = chrs.next() {
            match cur_state {
                // Ordered roughly by my guess of how common these patterns
                // are.
                State::InInteger => match chr {
                    '0'..='9' => {
                        // Nothing to do, the counter will advance;
                    }

                    ' ' | ',' | '\t' | ';' => {
                        // We have to emit the previous integer token.
                        tokens.push(emit_token(line, &mut i, j, cur_state));
                        cur_state = State::OnDelimiter;
                        tokens.push(emit_token(line, &mut i, j + 1, cur_state));
                    }

                    '.' => {
                        cur_state = State::InFloat;
                    }

                    'e' | 'E' => match chrs.peek() {
                        Some((_, chr)) => match chr {
                            '0'..='9' => {
                                cur_state = State::InScientific;
                            }
                            '-' | '+' => cur_state = State::MaybeScientific,
                            _ => {
                                invalid(line, i, j, line_no, cur_state);
                                break;
                            }
                        },
                        None => {
                            invalid(line, i, j, line_no, cur_state);
                            break;
                        }
                    },

                    _ => {
                        invalid(line, i, j, line_no, cur_state);
                        break;
                    }
                },

                State::InFloat => match chr {
                    '0'..='9' => {
                        // Nothing to do, the counter will advance;
                    }

                    ' ' | ',' | '\t' | ';' => {
                        // We have to emit the previous integer token.
                        tokens.push(emit_token(line, &mut i, j, cur_state));
                        cur_state = State::OnDelimiter;
                        tokens.push(emit_token(line, &mut i, j + 1, cur_state));
                    }

                    'e' | 'E' => match chrs.peek() {
                        Some((_, chr)) => match chr {
                            '0'..='9' => {
                                cur_state = State::InScientific;
                            }
                            '-' | '+' => cur_state = State::MaybeScientific,
                            _ => {
                                invalid(line, i, j, line_no, cur_state);
                                break;
                            }
                        },
                        None => {
                            invalid(line, i, j, line_no, cur_state);
                            break;
                        }
                    },

                    _ => {
                        invalid(line, i, j, line_no, cur_state);
                        break;
                    }
                },

                State::OnDelimiter => match chr {
                    ' ' | ',' | '\t' | ';' => {
                        tokens.push(emit_token(line, &mut i, j + 1, cur_state));
                    }

                    '0'..='9' => {
                        cur_state = State::InInteger;
                        i += 1;
                    }

                    '+' | '-' => {
                        match chrs.peek() {
                            Some((_, '0'..='9')) => cur_state = State::InInteger,
                            Some((_, '.')) => cur_state = State::MaybeFloat,
                            Some(_) => {
                                invalid(line, i, j, line_no, cur_state);
                                break;
                            }
                            None => {
                                // If we find a + or - as last char after a delimiter,
                                // we just ignore it and emit a warning.
                                log::warn!("ignoring trailing {} in line {}", chr, line_no);
                            }
                        }
                        i += 1;
                    }

                    _ => {
                        invalid(line, i, j, line_no, cur_state);
                        break;
                    }
                },

                State::StartOfLine => match chr {
                    '0'..='9' => {
                        cur_state = State::InInteger;
                    }
                    // With this we trim trailing whitespace.
                    ' ' | '\t' => {
                        // Otherwise, `i` is advanced by `emit_token`.
                        i += 1;
                    }
                    '+' | '-' => {
                        match chrs.peek() {
                            Some((_, '0'..='9')) => cur_state = State::InInteger,
                            Some((_, '.')) => cur_state = State::MaybeFloat,
                            Some(_) => cur_state = State::InComment,
                            None => {
                                // If the line contains just a single + or
                                // -, we treat it like a comment.
                                emit_token(line, &mut i, j, State::InComment);
                            }
                        }
                    }
                    '.' => {
                        cur_state = State::MaybeFloat;
                    }
                    ',' | ';' => {
                        cur_state = State::OnDelimiter;
                        tokens.push(emit_token(line, &mut i, j + 1, cur_state));
                    }
                    _ => {
                        cur_state = State::InComment;
                    }
                },

                State::InScientific => match chr {
                    '0'..='9' => {
                        cur_state = State::InScientific;
                    }

                    ' ' | ',' | '\t' | ';' => {
                        // We have to emit the previous integer token.
                        tokens.push(emit_token(line, &mut i, j, cur_state));
                        cur_state = State::OnDelimiter;
                        tokens.push(emit_token(line, &mut i, j + 1, cur_state));
                    }
                    _ => {
                        invalid(line, i, j, line_no, State::InFloat);
                        break;
                    }
                },

                // If we are in a comment, we accept all further
                // characters (the line end will be recognized
                // automatically by the outer loop).
                State::InComment => {}

                State::MaybeFloat => match chr {
                    '0'..='9' => cur_state = State::InFloat,
                    // We stay in state MaybeFloat and check next character
                    // in next iteration.
                    '.' => (),
                    'e' | 'E' => cur_state = State::MaybeScientific,
                    _ => {
                        invalid(line, i, j, line_no, State::InFloat);
                        break;
                    }
                },

                State::MaybeScientific => match chr {
                    '0'..='9' => cur_state = State::InScientific,
                    '-' | '+' => (),
                    _ => {
                        invalid(line, i, j, line_no, State::InFloat);
                        break;
                    }
                },
            }

            // If this is the end of the line, we may need to emit the
            // current item.
            if chrs.peek().is_none() {
                match cur_state {
                    State::InInteger | State::InFloat | State::InScientific | State::InComment => {
                        tokens.push(emit_token(line, &mut i, j + 1, cur_state));
                    }
                    State::MaybeFloat | State::MaybeScientific => {
                        invalid(line, i, j, line_no, State::InFloat);
                    }
                    // The delimiter was immediately emitted, so it must not be
                    // emitted here agian.
                    State::StartOfLine | State::OnDelimiter => (),
                };
            };
        }
    }
}

/// Emit token based on current text slice.
/// Panics if text handed cannot be parsed according to the state.
/// This should only happen if there is an unconsidered edge case
/// or a logic error.
///
/// This function also advances the counter `i`!
fn emit_token<'a>(raw_text: &'a str, i: &mut usize, j: usize, state: State) -> Token<'a> {
    let fail_msg = "token emitter was handed invalid raw text";
    let token = match state {
        State::InComment => Token::Comment(&raw_text[*i..j]),
        State::OnDelimiter => Token::Delimiter(raw_text[*i..j].chars().next().expect(fail_msg)),
        State::InInteger => Token::Integer(raw_text[*i..j].parse::<i64>().expect(fail_msg)),
        State::InFloat => Token::Float(raw_text[*i..j].parse::<f64>().expect(fail_msg)),
        State::InScientific => Token::Float(raw_text[*i..j].parse::<f64>().expect(fail_msg)),
        // MaybeStates should never emit a token!
        // StartOfLine is handeled in `walk_file` separately.
        State::MaybeFloat | State::MaybeScientific | State::StartOfLine => unreachable!(),
    };
    if state == State::OnDelimiter {
        *i = j - 1;
    } else {
        *i = j;
    }
    token
}

fn invalid(raw_text: &str, i: usize, j: usize, line_no: usize, state: State) {
    let parse_as = match state {
        State::OnDelimiter => "delimiter",
        State::InInteger => "integer",
        State::InFloat | State::InScientific | State::MaybeFloat | State::MaybeScientific => {
            "float"
        }
        // This function should never be used in these states.
        State::StartOfLine | State::InComment => {
            unreachable!()
        }
    };
    log::warn!(
        "unable to parse '{}' in line {}, position {} as {}, ignoring rest of line",
        &raw_text[i..j + 1],
        line_no,
        j + 1,
        parse_as,
    );
    let aux_msg = format!("{:~>1$}", "^", 18 + j - i);
    log::warn!("{}", aux_msg);
}

#[cfg(test)]
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
        assert_eq!(tokens[0], Token::Integer(220));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Integer(-210));
    }

    #[test]
    fn test_lex_integer_multi_line() {
        init();
        let mut lexer = Lexer::from_string("220;210;\n152;62;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[2], Token::Integer(210));
        assert_eq!(tokens[3], Token::Delimiter(';'));
        assert_eq!(tokens[4], Token::Newline);
    }

    #[test]
    fn test_lex_float_single_line() {
        init();
        let mut lexer = Lexer::from_string("22.0;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[0], Token::Float(22.0));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_negative_float_single_line() {
        init();
        let mut lexer = Lexer::from_string("-22.0;+2.10;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[0], Token::Float(-22.0));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Float(2.10));

        let mut lexer = Lexer::from_string("+.220;-.10;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[0], Token::Float(0.22));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Float(-0.1));
    }

    #[test]
    fn test_lex_float_multi_line() {
        init();
        let mut lexer = Lexer::from_string("22.0;231340298.0;\n0.00023;1.0;".into());

        let tokens = lexer.walk_file();
        let mut tokens = tokens.into_iter();
        assert_eq!(tokens.next(), Some(Token::Float(22.0)));
        assert_eq!(tokens.next(), Some(Token::Delimiter(';')));
        assert_eq!(tokens.next(), Some(Token::Float(231340298.0)));
        assert_eq!(tokens.next(), Some(Token::Delimiter(';')));
        assert_eq!(tokens.next(), Some(Token::Newline));
        assert_eq!(tokens.next(), Some(Token::Float(0.00023)));
        assert_eq!(tokens.next(), Some(Token::Delimiter(';')));
        assert_eq!(tokens.next(), Some(Token::Float(1.0)));
    }

    #[test]
    fn test_lex_scientific_single_line() {
        init();
        let mut lexer = Lexer::from_string("22.0e-1;".into());

        let tokens = lexer.walk_file();
        assert_eq!(tokens[0], Token::Float(2.20));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_scientific_multi_line() {
        init();
        let mut lexer = Lexer::from_string("22.e-1;\n0.0e-3;;;".into());

        let tokens = lexer.walk_file();
        let mut tokens = tokens.into_iter();
        assert_eq!(tokens.next(), Some(Token::Float(2.20)));
        assert_eq!(tokens.next(), Some(Token::Delimiter(';')));
        assert_eq!(tokens.next(), Some(Token::Newline));
        assert_eq!(tokens.next(), Some(Token::Float(0.0)));
    }

    #[test]
    fn test_parse_float() {
        // init();

        let input = r#"This is a header without comment char.
        # This is a comment
        2 34 1.2
        +1e-3 0.000213 1232e-3
        34 -2 3
        0.1 +2e-2 3.0001
        23.ef this is invalid"#;

        let parser = Parser::from_string(input.into());
        let expected = vec![
            vec![2.0, 0.001, 34.0, 0.1],
            vec![34.0, 0.000213, -2.0, 0.02],
            vec![1.2, 1.232, 3.0, 3.0001],
        ];

        let (_, result) = parser.parse_as_floats();
        assert_eq!(result, expected);

        let input = r#"This is a header without comment char.
        # This is a comment
        2;34;1.2
        +1e-3;0.000213;1232e-3
        34;-2;3
        0.1;+2e-2;3.0001
        23.ef this is invalid"#;

        let parser = Parser::from_string(input.into());
        let expected = vec![
            vec![2.0, 0.001, 34.0, 0.1],
            vec![34.0, 0.000213, -2.0, 0.02],
            vec![1.2, 1.232, 3.0, 3.0001],
        ];

        let (comment, result) = parser.parse_as_floats();
        println!("{}", comment);
        assert_eq!(result, expected);
    }
}
