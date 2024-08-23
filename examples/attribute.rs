use dom_query::Document;
use serde_json::json;

fn main() {
    let html = r#"<div name="foo" value="bar"></div>"#;
    let document = Document::from_slate_html(html);
    println!("{}", document.html());

    let mut input = document.select(r#"div[name="foo"]"#);
    println!("{}", input.outer_html());
    input.set_attr("id", json!("input"));
    input.remove_attr("name");
    println!("{}", input.attr("value").unwrap());

    println!("{}", input.outer_html());

    input.replace_with_html(r#"<a href="https://wisburg.com">wisburg</a><h2>xxx</h2>"#);
    println!("{}", input.outer_html());
    println!("{}", document.html());
}
