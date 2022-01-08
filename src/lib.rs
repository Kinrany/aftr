use nom::{
    character::complete::alphanumeric1,
    combinator::{all_consuming, eof, map},
    multi::many1,
    sequence::terminated,
};

/// Standard return type for [`nom`] string parsers.
type NResult<'a, T> = nom::IResult<&'a str, T>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Slash,
    Newline,
    Tab,
}

impl Token {
    fn word(s: &str) -> Self {
        Self::Word(s.to_string())
    }
}

fn token(input: &str) -> NResult<Token> {
    map(alphanumeric1, Token::word)(input)
}

pub fn lexer(input: &str) -> NResult<Vec<Token>> {
    all_consuming(terminated(many1(token), eof))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_word() {
        assert_eq!(
            lexer("singleword"),
            Ok(("", vec![Token::Word("singleword".to_string())]))
        );
    }
}
