use pom::{
    char_class::{alphanum, digit, hex_digit},
    parser::*,
    Parser,
};


#[derive(Debug)]
pub struct Spec {
    pub head: Vec<Fragment>,
    pub mouths: Vec<(String, Vec<Fragment>)>,
}

#[derive(Debug, Clone)]
pub enum Fragment {
    Text(String),
    StrumberInput(String, Option<DefaultValue>),
    BooleanInput(String),
    BlockInput(String),
    Expander,
    Special(Special),
}
#[derive(Debug, Clone)]
pub enum Special {
    Flag,
    Clockwise,
    Anticlockwise,
    ColorPicker(String),
    CustomBlock(Vec<()>),
}

#[derive(Debug, Clone)]
pub enum DefaultValue {
    Id(String),
    Str(String),
    Num(f64),
    Color(u8, u8, u8),
}

/// panics on invalid input so be careful
pub fn spec(s: &'static str) -> Spec {
    let lines: Vec<_> = s.lines().map(|l| parse_line().parse(l.as_bytes()).unwrap()).collect();
    let head = lines[0].clone();

    let mut mouths = vec![];
    if lines.len() > 1 {
        for mouth in lines[1..].chunks(2) {
            let Fragment::BlockInput(id) = &mouth[0][0] else { panic!() };
            mouths.push((id.clone(), mouth[1].clone()))
        }
    }
    Spec { head, mouths }
}

fn id() -> Parser<u8, String> {
    (is_a(alphanum) | sym(b'_'))
        .repeat(1..)
        .map(|cs| String::from_utf8(cs).unwrap())
}

fn string() -> Parser<u8, String> {
    (sym(b'"') * none_of(b"\"").repeat(0..) - sym(b'"'))
        .map(|s| String::from_utf8(s).unwrap())
}

fn number() -> Parser<u8, f64> {
    (sym(b'-').opt() + is_a(digit).repeat(1..) - sym(b'.') + is_a(digit).repeat(1..))
        .map(|((negative, mut big), mut small)| {
            // definitely suboptimal
            big.push(b'.');
            big.append(&mut small);
            let s = String::from_utf8(big).unwrap();
            let pos: f64 = s.parse().unwrap();
            if negative.is_some() { -pos } else { pos }
        })
}

fn special() -> Parser<u8, Fragment> {
      seq(b"$FLAG").map(|_| Fragment::Special(Special::Flag))
    | seq(b"$CLOCKWISE").map(|_| Fragment::Special(Special::Clockwise))
    | seq(b"$ANTICLOCKWISE").map(|_| Fragment::Special(Special::Anticlockwise))
    | sym(b'<').map(|_| Fragment::Text("<".into()))
    | sym(b'(').map(|_| Fragment::Text("(".into()))
    | sym(b'[').map(|_| Fragment::Text("[".into()))
    | sym(b'{').map(|_| Fragment::Text("{".into()))
    | id().map(|s| Fragment::Special(Special::ColorPicker(s)))
}

fn default_value() -> Parser<u8, DefaultValue> {
    sym(b'=') * (
          string().map(DefaultValue::Str)
        | number().map(DefaultValue::Num)
        | id().map(DefaultValue::Id)
        | (sym(b'#') * is_a(hex_digit).repeat(6)).map(|digits| {
            // unnecessary but whatever
            let s = u32::from_str_radix(&String::from_utf8(digits).unwrap(), 16).unwrap();
            DefaultValue::Color((s >> 16) as u8, (s >> 8) as u8, s as u8)
        })
    )
}

fn parse_line() -> Parser<u8, Vec<Fragment>> {
    (
        (sym(b'(') * id() + default_value().opt() - sym(b')'))
            .map(|(s, def)| Fragment::StrumberInput(s, def))
        | (sym(b'<') * id() - sym(b'>')).map(Fragment::BooleanInput)
        | (sym(b'{') * id() - sym(b'}')).map(Fragment::BlockInput)
        | (sym(b'[') * special() - sym(b']'))
        | sym(b'\t').map(|_| Fragment::Expander)
        | none_of(b"(<{[").repeat(1..).map(|s| Fragment::Text(String::from_utf8(s).unwrap()))
    ).repeat(1..)
}
