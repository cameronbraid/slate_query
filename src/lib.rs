//! HTML manipulation with CSS selectors.
//!
//! # Features
//!
//! * Iteration
//! * Manipulation
//! * Property
//! * Query
//! * Traversal
//!
//! # Get started
//!
//! ```
//! use dom_query::Document;
//!
//! let html = r#"<div>
//!     <a href="/1">One</a>
//!     <a href="/2">Two</a>
//!     <a href="/3">Three</a>
//! </div>"#;
//!
//! let document = Document::from_slate_html(html);
//! let a = document.select("a:nth-child(3)");
//! let text: &str = &a.text();
//! assert!(text == "Three");
//! ```
//!

// #![deny(missing_docs)] // TODO: add this back in.
extern crate html5ever;

mod css;
mod document;
mod dom_tree;
mod element;
mod entities;
mod manipulation;
mod matcher;
mod property;
mod query;
mod selection;
mod traversal;

pub use dom_tree::{Node, NodeRef};
pub use document::{Document, DocumentTreeSink, Attrib, Element, Text, NodeData};
#[doc(hidden)]
pub use document::SerializableNodeRef;
#[doc(hidden)]
pub use entities::NodeId;
pub use matcher::Matcher;
pub use selection::Selection;
pub use traversal::Selections;
