use nom::{
    branch::alt,
    character::complete::{alphanumeric1, char, newline},
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

fn token(input: &str) -> NResult<Token> {
    let word = map(alphanumeric1, Token::word);
    let slash = value(Token::Slash, char('/'));
    let newline = value(Token::Newline, newline);
    let tab = value(Token::Tab, char('\t'));

    alt((word, slash, newline, tab))(input)
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
            lexer("singleword").unwrap().1,
            vec![Token::word("singleword")]
        );
    }

    #[test]
    fn two_words_and_slash() {
        assert_eq!(
            lexer("two/words").unwrap().1,
            vec![Token::word("two"), Token::Slash, Token::word("words")]
        )
    }

    #[test]
    fn three_words_and_newline() {
        assert_eq!(
            lexer("three/words\nw/newline").unwrap().1,
            vec![
                Token::word("three"),
                Token::Slash,
                Token::word("words"),
                Token::Newline,
                Token::word("w"),
                Token::Slash,
                Token::word("newline")
            ]
        )
    }

    #[test]
    fn four_words_with_tab() {
        assert_eq!(
            lexer("four/words\n\twith/tab").unwrap().1,
            vec![
                Token::word("four"),
                Token::Slash,
                Token::word("words"),
                Token::Newline,
                Token::Tab,
                Token::word("with"),
                Token::Slash,
                Token::word("tab")
            ]
        )
    }
}
