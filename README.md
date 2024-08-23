
# SLATE_QUERY
Do Not Use

<div style="padding-top: 1000px">
</div>

> A crate for manipulating [SLATE-JSON](https://github.com/cameronbraid/slate_query) with CSS selectors, and includes a html like representation of slate-js

SLATE_QUERY is based on HTML crate html5ever and the CSS selector crate selectors. You can use the jQuery-like syntax to query and manipulate an slate-js document quickly. **With its help you can query and modify it.**.

It is a fork of [dom_query](https://github.com/niklak/dom_query), with a fair amount of changes to work with the slate js data model

## Example

#### Extract the hacker news.

```rust
use slare_query::Document;

fn main() {
    let html = r#"
      <h1><text>Welcome to slate-query</text></h1>
      <p><text>slate </text><text bold="true">with css</text></p>
    "#;
    let document = Document::from_slate_html(html);

    document.select("p").iter().for_each(|p| {
        let first = p.select("text:first-child");
        let bolded = p.select("text[bold=true]");
        println!("{}", first.text());
        println!("{}", bolded.text());
    });

    // TODO!
    //assert_eq!(document.to_json_string(), r#"[{"type":"h1","children":[{"text":"Welcome to slate-query"}]},{"type":"p","children":[{"text":"slate "}, {"bold":true,"text":"with css"}]}]"#)
}
```


## Related projects

* [dom_query](https://github.com/niklak/dom_query)
* [nipper](https://crates.io/crates/nipper)
* [html5ever](https://crates.io/crates/html5ever)
* [selectors](https://crates.io/crates/selectors)
* [goquery](https://godoc.org/github.com/PuerkitoBio/goquery)
* [scraper](https://crates.io/crates/scraper)
* [select.rs](https://crates.io/crates/select)


## Features

- `hashbrown` -- optional, standard hashmaps and hashsets will be replaced `hashbrown` hashmaps and hashsets;

## Changelog
[Changelog](./CHANGELOG.md)

## License

Licensed under MIT ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)


## Contribution

Any contribution intentionally submitted for inclusion in the work by you, shall be
licensed with MIT license, without any additional terms or conditions.
