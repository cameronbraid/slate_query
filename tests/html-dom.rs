use dom_query::SerializableNodeRef;
use html5ever::{local_name, parse_fragment, serialize, QualName};
use markup5ever::{namespace_url, ns};
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

test!(empty, r#""#);
test!(fuzz, "<a a=\r\n", "");
test!(
    smoke_test,
    r#"<p><i><text>Hello</text></i><text>, World!</text></p>"#
);

test!(
    misnest,
    r#"<p><i><text>Hello!</text></p><text>, World!</text></i>"#,
    r#"<p><i><text>Hello!</text></i></p><i><text>, World!</text></i>"#
);

test!(attr_literal, r#"<base foo="<'>">"#);
test!(attr_escape_amp, r#"<base foo="&amp;">"#);
test!(
    attr_escape_amp_2,
    r#"<base foo=&amp>"#,
    r#"<base foo="&amp;">"#
);
test!(
    attr_escape_nbsp,
    "<base foo=x\u{a0}y>",
    r#"<base foo="x&nbsp;y">"#
);
test!(
    attr_escape_quot,
    r#"<base foo='"'>"#,
    r#"<base foo="&quot;">"#
);
test!(
    attr_escape_several,
    r#"<span foo=3 title='test "with" &amp;quot;'>"#,
    r#"<span foo="3" title="test &quot;with&quot; &amp;quot;"></span>"#
);

test!(text_literal, r#"<p>"'"</p>"#, r#"<p><text>"'"</text></p>"#);
test!(
    text_escape_amp,
    r#"<p>&amp;</p>"#,
    r#"<p><text>&amp;</text></p>"#
);
test!(
    text_escape_amp_2,
    r#"<p>&amp</p>"#,
    r#"<p><text>&amp;</text></p>"#
);
test!(
    text_escape_nbsp,
    "<p>x\u{a0}y</p>",
    r#"<p><text>x&nbsp;y</text></p>"#
);
test!(
    text_escape_lt,
    r#"<p>&lt;</p>"#,
    r#"<p><text>&lt;</text></p>"#
);
test!(
    text_escape_gt,
    r#"<p>&gt;</p>"#,
    r#"<p><text>&gt;</text></p>"#
);
test!(
    text_escape_gt2,
    r#"<p>></p>"#,
    r#"<p><text>&gt;</text></p>"#
);

test!(
    script_literal,
    r#"<script>(x & 1) < 2; y > "foo" + 'bar'</script>"#,
    r#"<script><text>(x &amp; 1) &lt; 2; y &gt; "foo" + 'bar'</text></script>"#
);
test!(
    style_literal,
    r#"<style>(x & 1) < 2; y > "foo" + 'bar'</style>"#,
    r#"<style><text>(x &amp; 1) &lt; 2; y &gt; "foo" + 'bar'</text></style>"#
);
test!(
    xmp_literal,
    r#"<xmp>(x & 1) < 2; y > "foo" + 'bar'</xmp>"#,
    r#"<xmp><text>(x &amp; 1) &lt; 2; y &gt; "foo" + 'bar'</text></xmp>"#
);
test!(
    iframe_literal,
    r#"<iframe>(x & 1) < 2; y > "foo" + 'bar'</iframe>"#,
    r#"<iframe><text>(x &amp; 1) &lt; 2; y &gt; "foo" + 'bar'</text></iframe>"#
);
test!(
    noembed_literal,
    r#"<noembed>(x & 1) < 2; y > "foo" + 'bar'</noembed>"#,
    r#"<noembed><text>(x &amp; 1) &lt; 2; y &gt; "foo" + 'bar'</text></noembed>"#
);
test!(
    noframes_literal,
    r#"<noframes>(x & 1) < 2; y > "foo" + 'bar'</noframes>"#,
    r#"<noframes><text>(x &amp; 1) &lt; 2; y &gt; "foo" + 'bar'</text></noframes>"#
);

test!(
    pre_lf_0,
    "<pre>foo bar</pre>",
    "<pre><text>foo bar</text></pre>"
);
test!(
    pre_lf_1,
    "<pre>\nfoo bar</pre>",
    "<pre><text>foo bar</text></pre>"
);
test!(
    pre_lf_2,
    "<pre>\n\nfoo bar</pre>",
    "<pre><text>\nfoo bar</text></pre>"
);

test!(
    textarea_lf_0,
    "<textarea>foo bar</textarea>",
    "<textarea><text>foo bar</text></textarea>"
);
test!(
    textarea_lf_1,
    "<textarea>\nfoo bar</textarea>",
    "<textarea><text>foo bar</text></textarea>"
);
test!(
    textarea_lf_2,
    "<textarea>\n\nfoo bar</textarea>",
    "<textarea><text>\nfoo bar</text></textarea>"
);

test!(
    listing_lf_0,
    "<listing>foo bar</listing>",
    "<listing><text>foo bar</text></listing>"
);
test!(
    listing_lf_1,
    "<listing>\nfoo bar</listing>",
    "<listing><text>foo bar</text></listing>"
);
test!(
    listing_lf_2,
    "<listing>\n\nfoo bar</listing>",
    "<listing><text>\nfoo bar</text></listing>"
);

test!(comment_1, r#"<p>hi <!--world--></p>"#, r#"<p><text>hi </text></p>"#);
test!(comment_2, r#"<p>hi <!-- world--></p>"#, r#"<p><text>hi </text></p>"#);
test!(comment_3, r#"<p>hi <!--world --></p>"#, r#"<p><text>hi </text></p>"#);
test!(comment_4, r#"<p>hi <!-- world --></p>"#, r#"<p><text>hi </text></p>"#);
