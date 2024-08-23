use dom_query::Document;

fn main() {
    let html = r#"
    <ul>
    <li>Foo</li>
    <li>Bar</li>
    <li>Baz</li>
</ul>
"#;

    let document = Document::from_slate_html(html);

    let items = document.select("ul").select("li");

    for item in items.iter() {
        println!("{}", item.outer_html());
        println!("{}", item.text());
    }
}
