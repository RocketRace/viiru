use pom::{
    char_class::{alphanum, digit, hex_digit},
    parser::*,
    Parser,
};

#[derive(Debug)]
pub enum Shape {
    Circle,
    Hexagon,
    Stack,
}

#[derive(Debug)]
pub struct Spec {
    pub shape: Shape,
    pub shadow: bool,
    pub block_color: (u8, u8, u8),
    pub text_color: (u8, u8, u8),
    pub head: Vec<Fragment>,
    pub mouths: Vec<(String, Vec<Fragment>)>,
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
    VariableName(String),
    ListName(String),
    CustomColour(String),
    CustomBlock(Vec<()>),
}

#[derive(Debug, Clone)]
pub enum DefaultValue {
    Block(String),
    Str(String),
    Num(f64),
    Color(String /* #RRGGBB */),
}

/// panics on invalid input so be careful
pub fn spec(s: &'static str) -> Spec {
    let header = &s[..15];
    let shape = match s.as_bytes()[0] {
        b'<' => Shape::Hexagon,
        b'(' => Shape::Circle,
        b'{' => Shape::Stack,
        _ => panic!(),
    };
    let shadow = s.as_bytes()[14] == b'!';
    let block_bits = u32::from_str_radix(&header[1..7], 16).unwrap();
    let block_color = (
        (block_bits >> 16) as u8,
        (block_bits >> 8) as u8,
        block_bits as u8,
    );
    let text_bits = u32::from_str_radix(&header[8..14], 16).unwrap();
    let text_color = (
        (text_bits >> 16) as u8,
        (text_bits >> 8) as u8,
        text_bits as u8,
    );

    let s = &s[15..];
    let lines: Vec<_> = s
        .lines()
        .map(|l| parse_line().parse(l.as_bytes()).unwrap())
        .collect();
    let head = lines[0].clone();

    let mut mouths = vec![];
    if lines.len() > 1 {
        for mouth in lines[1..].chunks(2) {
            let Fragment::BlockInput(id) = &mouth[0][0] else {
                panic!()
            };
            mouths.push((id.clone(), mouth[1].clone()))
        }
    }
    Spec {
        shape,
        shadow,
        block_color,
        text_color,
        head,
        mouths,
    }
}

fn id() -> Parser<u8, String> {
    (is_a(alphanum) | sym(b'_'))
        .repeat(1..)
        .map(|cs| String::from_utf8(cs).unwrap())
}

fn string() -> Parser<u8, String> {
    (sym(b'"') * none_of(b"\"").repeat(0..) - sym(b'"')).map(|s| String::from_utf8(s).unwrap())
}

fn number() -> Parser<u8, f64> {
    (sym(b'-').opt() + is_a(digit).repeat(1..) - sym(b'.') + is_a(digit).repeat(1..)).map(
        |((negative, mut big), mut small)| {
            // definitely suboptimal
            big.push(b'.');
            big.append(&mut small);
            let s = String::from_utf8(big).unwrap();
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
        | (sym(b'=') * id()).map(Fragment::VariableName)
        | (sym(b'*') * id()).map(Fragment::ListName)
        | (sym(b'@') * id()).map(Fragment::FieldText)
        | (sym(b'#') * id()).map(Fragment::CustomColour)
}

fn default_value() -> Parser<u8, DefaultValue> {
    sym(b'=')
        * (string().map(DefaultValue::Str)
            | number().map(DefaultValue::Num)
            | id().map(DefaultValue::Block)
            | (sym(b'#') * is_a(hex_digit).repeat(6)).map(|mut digits| {
                // unnecessary but whatever
                digits.insert(0, b'#');
                let s = String::from_utf8(digits).unwrap();
                DefaultValue::Color(s)
            }))
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
            .map(|s| Fragment::Text(String::from_utf8(s).unwrap())))
    .repeat(1..)
}
