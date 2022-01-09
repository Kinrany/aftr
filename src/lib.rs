#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::non_ascii_literal)]

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, newline, satisfy},
    combinator::{all_consuming, eof, map, not, opt, peek, recognize, value},
    multi::{many0, many1, many_m_n},
    sequence::{delimited, terminated, tuple},
};

/// Standard return type for [`nom`] string parsers.
type NResult<'a, T> = nom::IResult<&'a str, T>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    BracketRoundClosing,
    BracketRoundOpening,
    Indentation,
    Identifier(String),
    LineComment(String),
    Newline,
    Operator(String),
    Slash,
}

impl Token {
    fn ident(s: &str) -> Self {
        Self::Identifier(s.into())
    }

    fn line_comment(s: &str) -> Self {
        Self::LineComment(s.into())
    }

    fn operator(s: &str) -> Self {
        Self::Operator(s.into())
    }
}

/// A single Unicode alphabetic character.
fn unicode_alphabetic(input: &str) -> NResult<char> {
    satisfy(char::is_alphabetic)(input)
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
    let line_comment = delimited(tag("//"), recognize(many0(not_newline)), opt(peek(newline)));
    let identifier = recognize(tuple((unicode_alphabetic, many0(word_character))));
    let operator = recognize(many_m_n(1, 2, alt((char('+'), char('<')))));
    let slash = terminated(char('/'), peek(not(char('/'))));
    let indentation = alt((tag("\t"), tag("    ")));

    alt((
        map(line_comment, Token::line_comment),
        map(identifier, Token::ident),
        map(operator, Token::operator),
        value(Token::BracketRoundClosing, char(')')),
        value(Token::BracketRoundOpening, char('(')),
        value(Token::Slash, slash),
        value(Token::Newline, newline),
        value(Token::Indentation, indentation),
    ))(input)
}

pub fn lexer(input: &str) -> NResult<Vec<Token>> {
    all_consuming(terminated(many1(token), eof))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn brackets() {
        assert_eq!(
            lexer("(()(").unwrap().1,
            vec![
                Token::BracketRoundOpening,
                Token::BracketRoundOpening,
                Token::BracketRoundClosing,
                Token::BracketRoundOpening,
            ]
        );
    }

    #[test]
    fn line_comment() {
        assert_eq!(
            lexer("//hello\nword").unwrap().1,
            vec![
                Token::line_comment("hello"),
                Token::Newline,
                Token::ident("word"),
            ]
        );
    }

    #[test]
    fn line_comment_on_last_line() {
        assert_eq!(
            lexer("hello\n//world").unwrap().1,
            vec![
                Token::ident("hello"),
                Token::Newline,
                Token::line_comment("world"),
            ]
        );
    }

    #[test]
    fn single_identifier() {
        assert_eq!(
            lexer("identifier").unwrap().1,
            vec![Token::ident("identifier")]
        );
    }

    #[test]
    fn single_identifier_with_underscore() {
        assert_eq!(
            lexer("single_identifier").unwrap().1,
            vec![Token::ident("single_identifier")]
        );
    }

    #[test]
    fn single_identifier_with_unicode() {
        assert_eq!(
            lexer("single_wꙮrd").unwrap().1,
            vec![Token::ident("single_wꙮrd")]
        );
    }

    #[test]
    fn disallow_identifier_starting_with_digit() {
        assert!(matches!(lexer("0word"), Err(_)));
    }

    #[test]
    fn two_identifiers_and_slash() {
        assert_eq!(
            lexer("two/identifiers").unwrap().1,
            vec![
                Token::ident("two"),
                Token::Slash,
                Token::ident("identifiers")
            ]
        );
    }

    #[test]
    fn two_identifiers_and_four_spaces() {
        assert_eq!(
            lexer("two\n    identifiers").unwrap().1,
            vec![
                Token::ident("two"),
                Token::Newline,
                Token::Indentation,
                Token::ident("identifiers")
            ]
        );
    }

    #[test]
    fn three_identifiers_and_newline() {
        assert_eq!(
            lexer("three/identifiers\nw/newline").unwrap().1,
            vec![
                Token::ident("three"),
                Token::Slash,
                Token::ident("identifiers"),
                Token::Newline,
                Token::ident("w"),
                Token::Slash,
                Token::ident("newline"),
            ]
        );
    }

    #[test]
    fn four_identifiers_with_indentation() {
        assert_eq!(
            lexer("four/words\n\twith/indentation").unwrap().1,
            vec![
                Token::ident("four"),
                Token::Slash,
                Token::ident("words"),
                Token::Newline,
                Token::Indentation,
                Token::ident("with"),
                Token::Slash,
                Token::ident("indentation"),
            ]
        );
    }

    #[test]
    fn operator_plus() {
        assert_eq!(
            lexer("one+two").unwrap().1,
            vec![
                Token::ident("one"),
                Token::operator("+"),
                Token::ident("two"),
            ]
        );
    }

    #[test]
    fn operator_left_shift() {
        assert_eq!(
            lexer("one<<two").unwrap().1,
            vec![
                Token::ident("one"),
                Token::operator("<<"),
                Token::ident("two"),
            ]
        );
    }

    #[test]
    fn whitepaper_1() {
        let text = "\
animal
    cat
        tiger";
        assert_eq!(
            lexer(text).unwrap().1,
            vec![
                Token::ident("animal"),
                Token::Newline,
                Token::Indentation,
                Token::ident("cat"),
                Token::Newline,
                Token::Indentation,
                Token::Indentation,
                Token::ident("tiger"),
            ]
        );
    }

    #[ignore = "not implemented"]
    #[test]
    fn whitepaper_2() {
        let text = "\
animal
    var
        legs
    cat
        // set the default value for all objects of type /animal/cat
        legs = 4

        // define a proc for all cats
        proc
            meow()

        tiger
            // override the meow() proc
            meow()
                world << \"ROAR!\"";
        assert_debug_snapshot!(lexer(text).unwrap().1);
    }
}
