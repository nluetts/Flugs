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
enum Token {
    Integer(i64),
    Float(f64),
    Delimiter(char),
    Comment(String),
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
        let mut comments = String::new();

        // These variables are used to keep track of the state of the parser.
        let mut max_column_idx = 0;
        let mut current_column_idx = 0;
        // Use this to check repeated delimiters.
        let mut delimiters = String::with_capacity(10);

        // For each line, we collect numbers in a this hashmap which maps from
        // column index to number. This way, we can fill missing entries in
        // columns which did not get a value from the current line (with NaN).
        let mut current_row: HashMap<usize, f64> = HashMap::with_capacity(10);

        let tokens = self.lexer.walk_input();
        let mut tokens = tokens.into_iter().peekable();
        let mut line_valid = true;
        while let Some(tok) = tokens.next() {
            // If the current column index goes beyond the
            // currently known maximum number of columns, we
            // have to add another column.
            if current_column_idx > max_column_idx {
                max_column_idx = current_column_idx;
            }
            match tok {
                Token::Integer(x) => {
                    current_row.insert(current_column_idx, x as f64);
                }
                Token::Float(x) => {
                    current_row.insert(current_column_idx, x);
                }
                Token::Delimiter(delim) => {
                    // We collect all delimiters that follow the current
                    // delimiter into string `delimiters`.
                    delimiters.clear();
                    delimiters.push(delim);
                    while let Some(Token::Delimiter(delim)) = tokens.peek() {
                        delimiters.push(*delim);
                        tokens
                            .next()
                            .expect("A value that we were able to peek at went missing!");
                    }

                    let is_whitespace = |chr| [' ', '\t'].contains(&chr);
                    // If all repeated delimiters are whitespace, we count them as
                    // a single delimiter.
                    if delimiters.chars().all(is_whitespace) {
                        current_column_idx += 1;
                    // If repeated delimiters contain non-whitespace delimiters,
                    // we ignore the whitespace and count only non-whitespace
                    // delimiters.
                    } else {
                        current_column_idx += delimiters
                            .chars()
                            .filter(|chr| !is_whitespace(*chr))
                            .count();
                    }
                }
                Token::Comment(c) => comments.extend(c.chars().chain(['\n'])),
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

    fn walk_input(&mut self) -> Vec<Token> {
        let raw_input = &self.raw_input;

        let mut tokens = Vec::new();
        let mut lex_buffer = String::with_capacity(4096);

        for (line_no, line) in raw_input.lines().enumerate() {
            self.lex_line(line_no, line, &mut lex_buffer, &mut tokens);
        }
        tokens
    }

    fn lex_line<'a>(
        &'a self,
        line_no: usize,
        line: &'a str,
        lex_buffer: &mut String,
        tokens: &mut Vec<Token>,
    ) {
        if line_no > 0 {
            tokens.push(Token::Newline);
        }

        let mut state = State::StartOfLine;
        let mut chrs = line.chars().enumerate().peekable();
        while let Some((i, chr)) = chrs.next() {
            match chr {
                '0'..='9' => match state {
                    State::StartOfLine | State::OnDelimiter => {
                        lex_buffer.clear();
                        lex_buffer.push(chr);
                        state = State::InInteger
                    }
                    State::MaybeFloat => {
                        lex_buffer.push(chr);
                        state = State::InFloat;
                    }
                    State::MaybeScientific => {
                        lex_buffer.push(chr);
                        state = State::InScientific;
                    }
                    // In all other cases, the state does not need to change.
                    _ => {
                        lex_buffer.push(chr);
                    }
                },
                ' ' | ',' | '\t' | ';' => match state {
                    State::InInteger => {
                        tokens.push(Token::Integer(lex_buffer.parse().unwrap()));
                        lex_buffer.clear();
                        tokens.push(Token::Delimiter(chr));
                        state = State::OnDelimiter;
                    }
                    State::InFloat | State::InScientific => {
                        tokens.push(Token::Float(lex_buffer.parse().unwrap()));
                        lex_buffer.clear();
                        tokens.push(Token::Delimiter(chr));
                        state = State::OnDelimiter;
                    }
                    State::MaybeFloat | State::MaybeScientific => {
                        invalid(&lex_buffer, chr, i, line_no, state);
                        state = State::InComment;
                    }
                    State::InComment => lex_buffer.push(chr),
                    State::StartOfLine => {
                        // Ignore trailing whitespace.
                        if [',', ';'].contains(&chr) {
                            tokens.push(Token::Delimiter(chr))
                        }
                    }
                    State::OnDelimiter => tokens.push(Token::Delimiter(chr)),
                },

                '.' => match state {
                    State::StartOfLine | State::OnDelimiter => {
                        lex_buffer.clear();
                        lex_buffer.push(chr);
                        state = State::MaybeFloat;
                    }
                    State::InComment => {
                        lex_buffer.push(chr);
                    }
                    State::InInteger => {
                        lex_buffer.push(chr);
                        state = State::InFloat;
                    }
                    State::MaybeFloat
                    | State::MaybeScientific
                    | State::InFloat
                    | State::InScientific => invalid(&lex_buffer, chr, i, line_no, state),
                },

                '+' | '-' => match state {
                    State::InInteger | State::InFloat | State::MaybeFloat | State::InScientific => {
                        invalid(&lex_buffer, chr, i, line_no, state);
                    }
                    State::InComment => lex_buffer.push(chr),
                    State::StartOfLine | State::OnDelimiter => {
                        lex_buffer.clear();
                        lex_buffer.push(chr);
                        state = State::InInteger;
                    }
                    State::MaybeScientific => lex_buffer.push(chr),
                },

                'e' | 'E' => match state {
                    State::InFloat | State::InInteger | State::MaybeFloat => {
                        lex_buffer.push(chr);
                        state = State::MaybeScientific;
                    }
                    State::InComment => {
                        lex_buffer.push(chr);
                    }
                    State::StartOfLine | State::OnDelimiter => {
                        lex_buffer.clear();
                        lex_buffer.push(chr);
                        state = State::InComment;
                    }
                    State::InScientific | State::MaybeScientific => {
                        invalid(&lex_buffer, chr, i, line_no, state)
                    }
                },

                _ => match state {
                    State::StartOfLine | State::OnDelimiter => {
                        lex_buffer.clear();
                        lex_buffer.push(chr);
                        state = State::InComment;
                    }
                    State::InComment => {
                        lex_buffer.push(chr);
                    }
                    State::InInteger
                    | State::MaybeFloat
                    | State::MaybeScientific
                    | State::InFloat
                    | State::InScientific => invalid(&lex_buffer, chr, i, line_no, state),
                },
            }

            // If this is the end of the line, we may need to emit the
            // current item.
            if chrs.peek().is_none() {
                match state {
                    State::InInteger => {
                        tokens.push(Token::Integer(lex_buffer.parse().unwrap()));
                    }
                    State::InComment => tokens.push(Token::Comment(lex_buffer.clone())),
                    State::InFloat | State::InScientific => {
                        tokens.push(Token::Float(lex_buffer.parse().unwrap()));
                    }
                    State::MaybeFloat | State::MaybeScientific => {
                        invalid(&lex_buffer, chr, i, line_no, state);
                    }
                    _ => {}
                };
            };
        }
    }
}

fn invalid(raw_text: &str, chr: char, i: usize, line_no: usize, state: State) {
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
        "unable to parse '{}{}' in line {}, position {} as {}, ignoring rest of line",
        &raw_text,
        chr,
        line_no,
        i,
        parse_as,
    );
    let aux_msg = format!("{:~>1$}", "^", 18 + i);
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

        let tokens = lexer.walk_input();
        assert_eq!(tokens[0], Token::Integer(220));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_negative_integer_single_line() {
        init();
        let mut lexer = Lexer::from_string("+220;-210;".into());

        let tokens = lexer.walk_input();
        assert_eq!(tokens[0], Token::Integer(220));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Integer(-210));
    }

    #[test]
    fn test_lex_integer_multi_line() {
        init();
        let mut lexer = Lexer::from_string("220;210;\n152;62;".into());

        let tokens = lexer.walk_input();
        assert_eq!(tokens[2], Token::Integer(210));
        assert_eq!(tokens[3], Token::Delimiter(';'));
        assert_eq!(tokens[4], Token::Newline);
    }

    #[test]
    fn test_lex_float_single_line() {
        init();
        let mut lexer = Lexer::from_string("22.0;".into());

        let tokens = lexer.walk_input();
        assert_eq!(tokens[0], Token::Float(22.0));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_negative_float_single_line() {
        init();
        let mut lexer = Lexer::from_string("-22.0;+2.10;".into());

        let tokens = lexer.walk_input();
        assert_eq!(tokens[0], Token::Float(-22.0));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Float(2.10));

        let mut lexer = Lexer::from_string("+.220;-.10;".into());

        let tokens = lexer.walk_input();
        assert_eq!(tokens[0], Token::Float(0.22));
        assert_eq!(tokens[1], Token::Delimiter(';'));
        assert_eq!(tokens[2], Token::Float(-0.1));
    }

    #[test]
    fn test_lex_float_multi_line() {
        init();
        let mut lexer = Lexer::from_string("22.0;231340298.0;\n0.00023;1.0;".into());

        let tokens = lexer.walk_input();
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

        let tokens = lexer.walk_input();
        assert_eq!(tokens[0], Token::Float(2.20));
        assert_eq!(tokens[1], Token::Delimiter(';'));
    }

    #[test]
    fn test_lex_scientific_multi_line() {
        init();
        let mut lexer = Lexer::from_string("22.e-1;\n0.0e-3;;;".into());

        let tokens = lexer.walk_input();
        let mut tokens = tokens.into_iter();
        assert_eq!(tokens.next(), Some(Token::Float(2.20)));
        assert_eq!(tokens.next(), Some(Token::Delimiter(';')));
        assert_eq!(tokens.next(), Some(Token::Newline));
        assert_eq!(tokens.next(), Some(Token::Float(0.0)));
    }

    #[test]
    fn test_parse_float() {
        init();

        let input = r#"# This is a comment
        10.0,20.0
        20.0,40.0
        "#;

        let parser = Parser::from_string(input.into());
        let expected = vec![vec![10.0, 20.0], vec![20.0, 40.0]];

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

    #[test]
    fn test_watercluster_paper_file() {
        let input = r#"# This is a comment
        10.0, 20.0
        20.0, 40.0
        "#;

        let expected = vec![vec![10.0, 20.0], vec![20.0, 40.0]];
        let parser = Parser::from_string(input.into());
        let (_, result) = parser.parse_as_floats();
        assert_eq!(result, expected);
    }
}
