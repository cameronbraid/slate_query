use dom_query::Document;
use dom_query::Element;
use dom_query::SerializableNodeRef;
use dom_query::Text;
use html5ever::parse_fragment;
use html5ever::serialize;
use html5ever::QualName;
use markup5ever::{local_name, namespace_url, ns};
use serde_json::json;
use tendril::SliceExt;
use tendril::StrTendril;
use tendril::TendrilSink;

fn parse_and_serialize(input: StrTendril) -> StrTendril {
    let dom = parse_fragment(
        dom_query::DocumentTreeSink::default(),
        Default::default(),
        QualName::new(None, ns!(html), local_name!("body")),
        vec![],
    )
    .one(input);

    let root = dom.root();
    let inner: SerializableNodeRef = root.first_child().unwrap().into();

    let mut result = vec![];
    serialize(&mut result, &inner, Default::default()).unwrap();
    StrTendril::try_from_byte_slice(&result).unwrap()
}
macro_rules! test_fn {
    ($f:ident, $name:ident, $input:expr, $output:expr) => {
        #[test]
        fn $name() {
            assert_eq!($output, &*$f($input.to_tendril()));
        }
    };

    // Shorthand for $output = $input
    ($f:ident, $name:ident, $input:expr) => {
        test_fn!($f, $name, $input, $input);
    };
}

macro_rules! test {
    ($($t:tt)*) => {
        test_fn!(parse_and_serialize, $($t)*);
    };
}

test!(
    smoke_test,
    r#"<doc_meta><text></text></doc_meta><doc_content></doc_content>"#
);

test!(
    smoke_test_2,
    r#"<doc_meta><text></text></doc_meta><doc_content><p><text>hello</text></p></doc_content>"#
);

test!(
    section_roundtrip,
    r#"<doc_meta><text></text></doc_meta><doc_content><section variation="default"><p><text>hello</text></p></section></doc_content>"#
);

test!(num_roundtrip, r#"<foo num="1"></foo>"#);

test!(string_roundtrip, r#"<foo bar="baz"></foo>"#);

test!(bool_roundtrip, r#"<foo b="true"></foo>"#);

test!(array_roundtrip, r#"<foo bar="[&quot;baz&quot;]"></foo>"#);

#[test]
fn test_num_attr() {
    let html = r#"<foo num="1"></foo>"#;

    let document = Document::from_slate_html(html);
    let foo = document.select("foo");
    assert_eq!(foo.attr("num"), Some(json!(1)));
}

#[test]
fn test_bool_attr() {
    let html = r#"<foo b="true"></foo>"#;
    let document = Document::from_slate_html(html);
    let foo = document.select("foo");
    assert_eq!(foo.attr("b"), Some(json!(true)));
}

#[test]
fn test_string_attr() {
    let html = r#"<foo bar="baz"></foo>"#;
    let document = Document::from_slate_html(html);
    let foo = document.select("foo");
    assert_eq!(foo.attr("bar"), Some(json!("baz")));
}

#[test]
fn test_array() {
    let html = r#"<foo bar="[&quot;bar&quot;]"></foo>"#;
    let document = Document::from_slate_html(html);
    let mut foo = document.select("foo");
    assert_eq!(foo.attr("bar"), Some(json!(["bar"])));

    foo.set_attr("baz", json!(["baz"]));
    assert_eq!(
        &document.html()[..],
        r#"<foo bar="[&quot;bar&quot;]" baz="[&quot;baz&quot;]"></foo>"#
    );
}

#[test]
fn test_text_contents() {
    let html = r#"<foo>bar</foo>"#;
    let document = Document::from_slate_html(html);
    let foo = document.select("foo");
    assert_eq!(foo.text(), "bar".into());
}

#[test]
fn test_text_append() {
    let html = r#"<foo><text bold="true">bar</text><text italic="true">bar</text></foo>"#;
    let document = Document::from_slate_html(html);
    let mut txt = document.select("foo text[bold=true]");

    assert_eq!(txt.text(), "bar".into());
    txt.append_text_contents("-baz");
    assert_eq!(txt.text(), "bar-baz".into());

    txt.set_text_contents("cat");
    assert_eq!(txt.text(), "cat".into());

    let txt = document.select("foo text[italic=true]");
    assert_eq!(txt.text(), "bar".into());

}

#[test]
fn test_element_append() {
    let html = r#"<foo></foo>"#;
    let document = Document::from_slate_html(html);
    let mut foo = document.select("foo");
    foo.append_first_child(Text::new("before"));
    foo.append_last_child(Text::new("after"));

    assert_eq!(
        r#"<foo><text>before</text><text>after</text></foo>"#,
        document.html().to_string()
    );
}


#[test]
fn test_element_insert_before_after() {
    let html = r#"<foo></foo>"#;
    let document = Document::from_slate_html(html);
    let mut foo = document.select("foo");
    foo.insert_before(Element::new("before"));
    foo.insert_after(Element::new("after"));
    assert_eq!(
        r#"<before></before><foo></foo><after></after>"#,
        document.html().to_string()
    );
}
