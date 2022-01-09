#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::non_ascii_literal)]

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, newline, satisfy},
    combinator::{all_consuming, eof, map, not, opt, peek, recognize, value},
    multi::{many0, many1},
    sequence::{delimited, terminated},
};

/// Standard return type for [`nom`] string parsers.
type NResult<'a, T> = nom::IResult<&'a str, T>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    LineComment(String),
    Newline,
    Slash,
    Tab,
    Word(String),
}

impl Token {
    fn line_comment(s: &str) -> Self {
        Self::LineComment(s.into())
    }

    fn word(s: &str) -> Self {
        Self::Word(s.into())
    }
}

/// A single alphanumeric (Unicode) character or underscore.
fn word_character(input: &str) -> NResult<char> {
    alt((char('_'), satisfy(char::is_alphanumeric)))(input)
}

/// A single character that is not a newline.
fn not_newline(input: &str) -> NResult<char> {
    satisfy(|ch| ch != '\n')(input)
}

fn token(input: &str) -> NResult<Token> {
    let line_comment = map(
        delimited(tag("//"), recognize(many0(not_newline)), opt(peek(newline))),
        Token::line_comment,
    );
    let word = map(recognize(many1(word_character)), Token::word);
    let slash = terminated(value(Token::Slash, char('/')), peek(not(char('/'))));
    let newline_token = value(Token::Newline, newline);
    let tab = value(Token::Tab, char('\t'));

    alt((line_comment, word, slash, newline_token, tab))(input)
}

pub fn lexer(input: &str) -> NResult<Vec<Token>> {
    all_consuming(terminated(many1(token), eof))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_comment() {
        assert_eq!(
            lexer("//hello\nword").unwrap().1,
            vec![
                Token::line_comment("hello"),
                Token::Newline,
                Token::word("word"),
            ]
        );
    }

    #[test]
    fn line_comment_on_last_line() {
        assert_eq!(
            lexer("hello\n//world").unwrap().1,
            vec![
                Token::word("hello"),
                Token::Newline,
                Token::line_comment("world"),
            ]
        );
    }

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
                Token::word("newline"),
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
                Token::word("tab"),
            ]
        );
    }
}
