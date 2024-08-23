use dom_query::Document;

fn main() {
    let html = r#"
        <h1><text>Welcome to slate-query</text></h1>
        <p><text>slate </text><text bold="true">with css</text></p>
    "#;
    let document = Document::from_slate_html(html);

    document.select("p").iter().for_each(|p| {
        eprintln!("p {:?}", p.outer_html());
        let text = p.select("text:first-child");
        // let bolded = p.select("text[bold=true]");
        // println!("{}", text.outer_html());
        println!("{}", text.text());
        // println!("{}", bolded.text());
    });
}
