#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::non_ascii_literal)]

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace1, newline, one_of, satisfy},
    combinator::{all_consuming, eof, map, opt, peek, recognize, value},
    multi::{many0, many1, many_m_n},
    number::complete::double,
    sequence::{delimited, terminated, tuple},
};

/// Standard return type for [`nom`] string parsers.
type NResult<'a, T> = nom::IResult<&'a str, T>;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    BracketRoundClosing,
    BracketRoundOpening,
    Identifier(&'a str),
    LineComment(&'a str),
    Number(f64),
    Operator(&'a str),
    Slash,
    String(&'a str),
    Whitespace(&'a str),
}

/// A single Unicode alphabetic character.
fn unicode_alphabetic(input: &str) -> NResult<char> {
    satisfy(char::is_alphabetic)(input)
}

/// A single alphanumeric (Unicode) character or underscore.
fn word_character(input: &str) -> NResult<char> {
    alt((char('_'), satisfy(char::is_alphanumeric)))(input)
}

/// Any single character but the specified one.
fn not_char(banned_char: char) -> impl Fn(&str) -> NResult<char> {
    move |input| satisfy(|ch| ch != banned_char)(input)
}

fn token(input: &str) -> NResult<Token> {
    let line_comment = delimited(
        tag("//"),
        recognize(many0(not_char('\n'))),
        opt(peek(newline)),
    );
    let identifier = recognize(tuple((unicode_alphabetic, many0(word_character))));
    let string = delimited(char('"'), recognize(many0(not_char('"'))), char('"'));
    let operator = recognize(many_m_n(1, 2, one_of("+<=.")));

    alt((
        map(line_comment, Token::LineComment),
        map(identifier, Token::Identifier),
        map(double, Token::Number),
        map(string, Token::String),
        map(operator, Token::Operator),
        map(multispace1, Token::Whitespace),
        value(Token::BracketRoundClosing, char(')')),
        value(Token::BracketRoundOpening, char('(')),
        value(Token::Slash, char('/')),
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
                Token::LineComment("hello"),
                Token::Whitespace("\n"),
                Token::Identifier("word"),
            ]
        );
    }

    #[test]
    fn line_comment_on_last_line() {
        assert_eq!(
            lexer("hello\n//world").unwrap().1,
            vec![
                Token::Identifier("hello"),
                Token::Whitespace("\n"),
                Token::LineComment("world"),
            ]
        );
    }

    #[test]
    fn single_identifier() {
        assert_eq!(
            lexer("identifier").unwrap().1,
            vec![Token::Identifier("identifier")]
        );
    }

    #[test]
    fn single_identifier_with_underscore() {
        assert_eq!(
            lexer("single_identifier").unwrap().1,
            vec![Token::Identifier("single_identifier")]
        );
    }

    #[test]
    fn single_identifier_with_unicode() {
        assert_eq!(
            lexer("single_wꙮrd").unwrap().1,
            vec![Token::Identifier("single_wꙮrd")]
        );
    }

    #[test]
    fn identifier_after_number() {
        assert_eq!(
            lexer("0word").unwrap().1,
            vec![Token::Number(0.0), Token::Identifier("word")]
        );
    }

    #[test]
    fn two_identifiers_and_slash() {
        assert_eq!(
            lexer("two/identifiers").unwrap().1,
            vec![
                Token::Identifier("two"),
                Token::Slash,
                Token::Identifier("identifiers")
            ]
        );
    }

    #[test]
    fn two_identifiers_and_four_spaces() {
        assert_eq!(
            lexer("two\n    identifiers").unwrap().1,
            vec![
                Token::Identifier("two"),
                Token::Whitespace("\n    "),
                Token::Identifier("identifiers")
            ]
        );
    }

    #[test]
    fn three_identifiers_and_newline() {
        assert_eq!(
            lexer("three/identifiers\nw/newline").unwrap().1,
            vec![
                Token::Identifier("three"),
                Token::Slash,
                Token::Identifier("identifiers"),
                Token::Whitespace("\n"),
                Token::Identifier("w"),
                Token::Slash,
                Token::Identifier("newline"),
            ]
        );
    }

    #[test]
    fn four_identifiers_with_indentation() {
        assert_eq!(
            lexer("four/words\n\twith/indentation").unwrap().1,
            vec![
                Token::Identifier("four"),
                Token::Slash,
                Token::Identifier("words"),
                Token::Whitespace("\n\t"),
                Token::Identifier("with"),
                Token::Slash,
                Token::Identifier("indentation"),
            ]
        );
    }

    #[test]
    fn operator_plus() {
        assert_eq!(
            lexer("one+two").unwrap().1,
            vec![
                Token::Identifier("one"),
                Token::Operator("+"),
                Token::Identifier("two"),
            ]
        );
    }

    #[test]
    fn operator_left_shift() {
        assert_eq!(
            lexer("one<<two").unwrap().1,
            vec![
                Token::Identifier("one"),
                Token::Operator("<<"),
                Token::Identifier("two"),
            ]
        );
    }

    #[test]
    fn string() {
        assert_eq!(lexer("\"text\"").unwrap().1, vec![Token::String("text")]);
    }

    #[test]
    fn whitespace() {
        assert_eq!(
            lexer("one two").unwrap().1,
            vec![
                Token::Identifier("one"),
                Token::Whitespace(" "),
                Token::Identifier("two"),
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
                Token::Identifier("animal"),
                Token::Whitespace("\n    "),
                Token::Identifier("cat"),
                Token::Whitespace("\n        "),
                Token::Identifier("tiger"),
            ]
        );
    }

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

    #[test]
    fn whitepaper_3() {
        let text = r#"
// define some stuff
var/x = 3
var/player = usr
var/mob/m = usr

// output some stuff
world << player.x  // "player" has no type so we get a compile-time error
world << m.x       // "m" is of type /mob, and mobs have an x variable""#;

        assert_debug_snapshot!(lexer(text).unwrap().1);
    }
}
