use std::iter::once;

use leptos::html::{div, h1, h2, h3, h4, h5, h6, AnyElement};
use leptos::{
    create_effect, create_node_ref, create_rw_signal, provide_context, use_context, view,
    CollectView, IntoView, NodeRef, RwSignal, SignalUpdate, View,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1, take_until1};
use nom::character::complete::newline;
use nom::combinator::{map, map_res, rest};
use nom::multi::{many0, many1, many1_count};
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    Heading(usize, Vec<Segment>),
    Emphasis(Emphasis, Vec<Segment>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
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
    pub fn parse(input: &str) -> IResult<&str, Self> {
        map(many0(Segment::parse), |segments| Self {
            segments: { segments },
        })(input)
    }
}

impl IntoView for Document {
    fn into_view(self) -> View {
        let headings = create_rw_signal(Vec::<(i32, usize)>::new());
        provide_context(headings);
        view! {
            {self.segments}
            {move || {
                let headings = headings();
                let min = headings.iter().map(|(offset, _)| *offset).min().unwrap_or_default();
                headings
                    .into_iter()
                    .map(|(offset, depth)| {
                        view! {
                            <div
                                style:top=format!("{}px", offset - min)
                                class="absolute -left-16 w-12 [*:has(&)]:overflow-visible flex justify-end pointer-events-none"
                            >
                                {"#".repeat(depth) + " "}
                            </div>
                        }
                    })
                    .collect_view()
            }}
        }.into_view()
    }
}

impl IntoView for Segment {
    fn into_view(self) -> View {
        match self {
            Self::Text(text) => view! { <span>{text}</span> }.into_view(),
            Self::Heading(depth, segments) => {
                let hashes = "#".repeat(depth) + " ";
                let heading: NodeRef<AnyElement> = create_node_ref();
                create_effect(move |_| {
                    use_context::<RwSignal<Vec<_>>>()
                        .unwrap()
                        .update(|headings| {
                            headings.push((heading().unwrap().offset_top(), depth));
                        });
                });
                view! {
                    {match depth {
                        1 => h1().into_any(),
                        2 => h2().into_any(),
                        3 => h3().into_any(),
                        4 => h4().into_any(),
                        5 => h5().into_any(),
                        6 => h6().into_any(),
                        _ => div().into_any(),
                    }
                        .node_ref(heading)
                        .classes("inline font-bold")
                        .child((view! { <div class="inline text-fade">{&hashes}</div> }, segments))}
                }
                .into_view()
            }
            Self::Emphasis(emphasis, segments) => view! {
                <div
                    class="inline"
                    class=("font-bold", emphasis == Emphasis::Bold)
                    class=("italic", emphasis == Emphasis::Italic)
                >
                    <div class="inline text-fade">{emphasis.delimiter()}</div>
                    {segments}
                    <div class="inline text-fade">{emphasis.delimiter()}</div>
                </div>
            }
            .into_view(),
        }
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
                map_res(
                    alt((
                        map(tuple((take_until1("\n"), newline)), |(text, _)| text),
                        rest,
                    )),
                    |text| {
                        many1(alt((Self::bold, Self::italic, Self::text)))(text)
                            .map(|(_, segments)| segments)
                    },
                ),
            ),
            |(depth, segments)| {
                Self::Heading(
                    depth,
                    segments
                        .into_iter()
                        .chain(once(Self::Text("\n".to_string())))
                        .collect(),
                )
            },
        ))(input)
    }

    fn bold(input: &str) -> IResult<&str, Self> {
        Self::emphasis(Emphasis::Bold)(input)
    }

    fn italic(input: &str) -> IResult<&str, Self> {
        Self::emphasis(Emphasis::Italic)(input)
    }

    fn text(input: &str) -> IResult<&str, Self> {
        map(take_till1(|char| "#*".contains(char)), |text: &str| {
            Self::Text(text.to_string())
        })(input)
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
