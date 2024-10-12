use nom::branch::alt;
use nom::bytes::complete::{tag, take_until1, take_while1};
use nom::combinator::{map, map_res, rest};
use nom::multi::{many0, many1, many1_count};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Document {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Text(String),
    Heading(usize, Vec<Segment>),
    Emphasis(Emphasis, Vec<Segment>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Emphasis {
    Bold,
    Italic,
}

impl Emphasis {
    const fn delimiter(self) -> &'static str {
        match self {
            Self::Bold => "**",
            Self::Italic => "*",
        }
    }

    const fn other(self) -> Self {
        match self {
            Self::Bold => Self::Italic,
            Self::Italic => Self::Bold,
        }
    }
}

impl Document {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(many0(Segment::parse), |segments| Self {
            segments: { segments },
        })(input)
    }
}

impl Segment {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((Self::heading, Self::text, Self::bold, Self::italic))(input)
    }
    fn heading(input: &str) -> IResult<&str, Self> {
        (map(
            separated_pair(
                many1_count(tag("#")),
                tag(" "),
                map_res(alt((take_until1("\n"), rest)), |text| {
                    many1(alt((Self::bold, Self::italic, Self::text)))(text)
                        .map(|(_, segments)| segments)
                }),
            ),
            |(depth, segments)| Self::Heading(depth, segments),
        ))(input)
    }

    fn bold(input: &str) -> IResult<&str, Self> {
        Self::emphasis(Emphasis::Bold)(input)
    }

    fn italic(input: &str) -> IResult<&str, Self> {
        Self::emphasis(Emphasis::Italic)(input)
    }

    fn text(input: &str) -> IResult<&str, Self> {
        map(
            take_while1(|char: char| char.is_alphanumeric() || char.is_whitespace() || char == '#'),
            |text: &str| Self::Text(text.to_string()),
        )(input)
    }

    fn emphasis(emphasis: Emphasis) -> impl Fn(&str) -> IResult<&str, Self> {
        let delimiter = emphasis.delimiter();
        move |input: &str| {
            map(
                delimited(
                    tag(delimiter),
                    many1(alt((Self::heading, Self::text, |input| {
                        Self::emphasis(emphasis.other())(input)
                    }))),
                    tag(delimiter),
                ),
                |segments| Self::Emphasis(emphasis, segments),
            )(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

    use super::*;

    #[test]
    fn parsing_document_works() {
        let inputs = [
            "",
            "# Hello",
            "This is a test",
            "## Subheading",
            "This is **bold**",
            "This is *italic*",
            "This is ***bold italic***",
            "## **Bold subheading**",
            "## *Italic subheading*",
            "## ***Bold italic subheading***",
            "This is a # symbol",
        ];

        for input in inputs {
            dbg!(input, all_consuming(Document::parse)(input).unwrap().1);
        }
    }

    #[test]
    fn parsing_headings_works() {
        assert_eq!(
            all_consuming(Segment::heading)("# Hello").unwrap().1,
            Segment::Heading(1, vec![Segment::Text("Hello".to_string())])
        );
        assert_eq!(
            all_consuming(Segment::heading)("## Subheading").unwrap().1,
            Segment::Heading(2, vec![Segment::Text("Subheading".to_string())])
        );
        assert_eq!(
            all_consuming(Segment::heading)("### Subsubheading")
                .unwrap()
                .1,
            Segment::Heading(3, vec![Segment::Text("Subsubheading".to_string())])
        );
        assert_eq!(
            all_consuming(Segment::heading)("# # Hash heading")
                .unwrap()
                .1,
            Segment::Heading(1, vec![Segment::Text("# Hash heading".to_string())])
        );
        assert!(Segment::heading("Not a heading").is_err());
    }

    #[test]
    fn parsing_emphasis_works() {
        assert_eq!(
            all_consuming(Segment::bold)("**bold**").unwrap().1,
            Segment::Emphasis(Emphasis::Bold, vec![Segment::Text("bold".to_string())])
        );
        assert_eq!(
            all_consuming(Segment::italic)("*italic*").unwrap().1,
            Segment::Emphasis(Emphasis::Italic, vec![Segment::Text("italic".to_string())])
        );
        assert_eq!(
            all_consuming(Segment::bold)("**nested *italic* bold**")
                .unwrap()
                .1,
            Segment::Emphasis(
                Emphasis::Bold,
                vec![
                    Segment::Text("nested ".to_string()),
                    Segment::Emphasis(Emphasis::Italic, vec![Segment::Text("italic".to_string())]),
                    Segment::Text(" bold".to_string())
                ]
            )
        );
        assert_eq!(
            all_consuming(Segment::italic)("*nested **bold** italic*")
                .unwrap()
                .1,
            Segment::Emphasis(
                Emphasis::Italic,
                vec![
                    Segment::Text("nested ".to_string()),
                    Segment::Emphasis(Emphasis::Bold, vec![Segment::Text("bold".to_string())]),
                    Segment::Text(" italic".to_string())
                ]
            )
        );
        assert_eq!(
            all_consuming(Segment::bold)("*** bold italic ***")
                .unwrap()
                .1,
            Segment::Emphasis(
                Emphasis::Bold,
                vec![Segment::Emphasis(
                    Emphasis::Italic,
                    vec![Segment::Text(" bold italic ".to_string())]
                )]
            )
        );
    }
}
