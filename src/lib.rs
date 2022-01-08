use nom::{
    branch::alt,
    character::complete::{alphanumeric1, char},
    combinator::{all_consuming, eof, map, value},
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

fn token_word(input: &str) -> NResult<Token> {
    map(alphanumeric1, Token::word)(input)
}

fn token_slash(input: &str) -> NResult<Token> {
    value(Token::Slash, char('/'))(input)
}

fn token(input: &str) -> NResult<Token> {
    alt((token_word, token_slash))(input)
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
            Ok(("", vec![Token::word("singleword")]))
        );
    }

    #[test]
    fn two_words_and_slash() {
        assert_eq!(
            lexer("two/words"),
            Ok((
                "",
                vec![Token::word("two"), Token::Slash, Token::word("words")]
            ))
        )
    }
}
