use core::str;

use pom::{
    char_class::{alphanum, digit, hex_digit},
    parser::*,
    Parser,
};

use crate::util::{assume_string, parse_rgb};

#[derive(Debug)]
pub enum Shape {
    Circle,
    Hexagon,
    Stack,
}

#[derive(Debug)]
pub struct Spec {
    pub shape: Shape,
    pub is_shadow: bool,
    pub is_hat: bool,
    pub block_color: (u8, u8, u8),
    pub text_color: (u8, u8, u8),
    pub alt_color: (u8, u8, u8),
    pub lines: Vec<Vec<Fragment>>,
}

#[derive(Debug, Clone)]
pub enum Fragment {
    Text(String),
    StrumberInput(String, Option<DefaultValue>),
    BooleanInput(String),
    BlockInput(String),
    Dropdown(String),
    Expander,
    Flag,
    Clockwise,
    Anticlockwise,
    FieldText(String),
    CustomColour(String),
    CustomBlock(Vec<()>),
}

#[derive(Debug, Clone)]
pub enum DefaultValue {
    Block(String),
    Str(String),
    Num(f64, bool),
    Color((u8, u8, u8)),
}

/// panics on invalid input so be careful
pub fn spec(s: &'static str) -> Spec {
    let header = &s[..1 + 6 + 1 + 6 + 1 + 6 + 1];
    let shape = match s.as_bytes()[0] {
        b'<' => Shape::Hexagon,
        b'(' => Shape::Circle,
        b'{' => Shape::Stack,
        _ => panic!(),
    };
    let is_shadow = s.as_bytes()[1 + 6 + 1 + 6 + 1 + 6] == b'!';
    let is_hat = s.as_bytes()[1 + 6 + 1 + 6 + 1 + 6] == b'^';
    let block_color = parse_rgb(&header[1..1 + 6]);
    let text_color = parse_rgb(&header[1 + 6 + 1..1 + 6 + 1 + 6]);
    let alt_color = parse_rgb(&header[1 + 6 + 1 + 6 + 1..1 + 6 + 1 + 6 + 1 + 6]);

    let s = &s[1 + 6 + 1 + 6 + 1 + 6 + 1..];
    let lines: Vec<_> = s
        .lines()
        .map(|l| parse_line().parse(l.as_bytes()).unwrap())
        .collect();

    Spec {
        shape,
        is_shadow,
        is_hat,
        block_color,
        text_color,
        alt_color,
        lines,
    }
}

fn id() -> Parser<u8, String> {
    (is_a(alphanum) | sym(b'_')).repeat(1..).map(assume_string)
}

fn string() -> Parser<u8, String> {
    (sym(b'"') * none_of(b"\"").repeat(0..) - sym(b'"')).map(assume_string)
}

fn number() -> Parser<u8, f64> {
    (sym(b'-').opt() + is_a(digit).repeat(1..) - sym(b'.') + is_a(digit).repeat(1..)).map(
        |((negative, mut big), mut small)| {
            // definitely suboptimal
            big.push(b'.');
            big.append(&mut small);
            let s = assume_string(big);
            let pos: f64 = s.parse().unwrap();
            if negative.is_some() {
                -pos
            } else {
                pos
            }
        },
    )
}

fn special() -> Parser<u8, Fragment> {
    seq(b"$FLAG").map(|_| Fragment::Flag)
        | seq(b"$CLOCKWISE").map(|_| Fragment::Clockwise)
        | seq(b"$ANTICLOCKWISE").map(|_| Fragment::Anticlockwise)
        | sym(b'<').map(|_| Fragment::Text("<".into()))
        | sym(b'(').map(|_| Fragment::Text("(".into()))
        | sym(b'[').map(|_| Fragment::Text("[".into()))
        | sym(b'{').map(|_| Fragment::Text("{".into()))
        | id().map(Fragment::Dropdown)
        | (sym(b'*') * id()).map(Fragment::FieldText)
        | (sym(b'#') * id()).map(Fragment::CustomColour)
}

fn default_value() -> Parser<u8, DefaultValue> {
    sym(b'=')
        * (string().map(DefaultValue::Str)
            // todo: distinguish between more number types
            | sym(b'@').map(|_| DefaultValue::Num(0.0, false))
            | number().map(|x| DefaultValue::Num(x, true))
            | id().map(DefaultValue::Block)
            | (sym(b'#') * is_a(hex_digit).repeat(6))
                .map(|digits| DefaultValue::Color(parse_rgb(&assume_string(digits)))))
}

fn parse_line() -> Parser<u8, Vec<Fragment>> {
    ((sym(b'(') * id() + default_value().opt() - sym(b')'))
        .map(|(s, def)| Fragment::StrumberInput(s, def))
        | (sym(b'<') * id() - sym(b'>')).map(Fragment::BooleanInput)
        | (sym(b'{') * id() - sym(b'}')).map(Fragment::BlockInput)
        | (sym(b'[') * special() - sym(b']'))
        | sym(b'\t').map(|_| Fragment::Expander)
        | none_of(b"(<{[")
            .repeat(1..)
            .map(|s| Fragment::Text(assume_string(s))))
    .repeat(1..)
}
