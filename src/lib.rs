use nom::{
    branch::alt,
    character::complete::{char, newline, satisfy},
    combinator::{all_consuming, eof, map, recognize, value},
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

/// A single alphanumeric (Unicode) character or underscore.
fn word_character(input: &str) -> NResult<char> {
    alt((char('_'), satisfy(char::is_alphanumeric)))(input)
}

fn token(input: &str) -> NResult<Token> {
    let word = map(recognize(many1(word_character)), Token::word);
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
    fn single_word_with_underscore() {
        assert_eq!(
            lexer("single_word").unwrap().1,
            vec![Token::word("single_word")]
        );
    }

    #[test]
    fn single_word_with_unicode() {
        assert_eq!(
            lexer("single_wꙮrd").unwrap().1,
            vec![Token::word("single_wꙮrd")]
        );
    }

    #[test]
    fn two_words_and_slash() {
        assert_eq!(
            lexer("two/words").unwrap().1,
            vec![Token::word("two"), Token::Slash, Token::word("words")]
        );
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
        );
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
        );
    }
}
