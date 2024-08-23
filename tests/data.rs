#![allow(dead_code)]
use dom_query::Document;

pub fn doc() -> Document {
  Document::from_slate_html(&include_str!("../test-pages/page.html"))
}

pub fn docwiki() -> Document {
  Document::from_slate_html(&include_str!("../test-pages/rustwiki.html"))
}

pub fn doc2() -> Document {
  Document::from_slate_html(&include_str!("../test-pages/page2.html"))
}
