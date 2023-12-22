
# DOM_QUERY

> A crate for manipulating HTML with Rust.

<div>
  <!-- Crates version -->
  <a href="https://crates.io/crates/dom_query">
    <img src="https://img.shields.io/crates/v/dom_query.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/dom_query">
    <img src="https://img.shields.io/crates/d/dom_query.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/dom_query">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>


DOM_QUERY is based on HTML crate html5ever and the CSS selector crate selectors. You can use the jQuery-like syntax to query and manipulate an HTML document quickly. **Not only can query, but also can modify**.

It is a fork of [nipper](https://crates.io/crates/nipper), with some updates. Also this fork supports ":has" pseudo-class, and some others.

## Example

#### Extract the hacker news.

```rust
use dom_query::Document;

fn main() {
    let html = include_str!("../test-pages/hacker_news.html");
    let document = Document::from(html);

    document.select("tr.athing:has(a[href][id])").iter().for_each(|athing| {
        let title = athing.select(".title a");
        let href = athing.select(".storylink");
        println!("{}", title.text());
        println!("{}", href.attr("href").unwrap());
        println!();
    });
}
```

#### Readability. 
[examples/readability.rs](./examples/readability.rs)

## Related projects

* [nipper](https://crates.io/crates/nipper)
* [html5ever](https://crates.io/crates/html5ever)
* [selectors](https://crates.io/crates/selectors)
* [goquery](https://godoc.org/github.com/PuerkitoBio/goquery)
* [scraper](https://crates.io/crates/scraper)
* [select.rs](https://crates.io/crates/select)


## License

Licensed under MIT ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)


#### Contribution

Any contribution intentionally submitted for inclusion in the work by you, shall be
licensed with MIT license, without any additional terms or conditions.
