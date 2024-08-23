use tendril::StrTendril;

use crate::{Attrib, Document, Selection};

impl Document {
    /// Gets the HTML contents of the document. It includes
    /// the text and comment nodes.
    pub fn html(&self) -> StrTendril {
        self.select("html>body").inner_html()
    }

    /// Gets the text content of the document.
    pub fn text(&self) -> StrTendril {
        self.tree.root().text()
    }
}

impl<'a> Selection<'a> {
    /// Gets the specified attribute's value for the first element in the
    /// selection. To get the value for each element individually, use a looping
    /// construct such as map method.
    pub fn attr(&self, name: &str) -> Option<serde_json::Value> {
      self.nodes().first().and_then(|node| node.attr(name))
    }

    /// Works like `attr` but returns default value if attribute is not present.
    pub fn attr_or(&self, name: &str, default: serde_json::Value) -> serde_json::Value {
        self.attr(name).unwrap_or_else(|| default)
    }

    pub fn attrs(&self) -> Option<Vec<Attrib>> {
      self.nodes().first().map(|node| node.attrs())
    }

    /// Sets the given attribute to each element in the set of matched elements.
  //   pub fn set_attr_parse_json_or_use_as_string(&mut self, name: &str, val: &str) {
  //     for node in self.nodes() {
  //         node.set_attr_parse_json_or_use_as_string(name, val);
  //     }
  // }
  pub fn set_attr(&mut self, name: &str, val: impl Into<serde_json::Value>) {
    let val = val.into();
    for node in self.nodes() {
        node.set_attr(name, val.clone());
    }
}

    /// Removes the named attribute from each element in the set of matched elements.
    pub fn remove_attr(&mut self, name: &str) {
        for node in self.nodes() {
            node.remove_attr(name);
        }
    }

    /// Adds the given class to each element in the set of matched elements.
    /// Multiple class names can be specified, separated by a space via multiple arguments.
    pub fn add_class(&mut self, class: &str) {
        for node in self.nodes() {
            node.add_class(class);
        }
    }

    /// Determines whether any of the matched elements are assigned the
    /// given class.
    pub fn has_class(&self, class: &str) -> bool {
        self.nodes().iter().any(|node| node.has_class(class))
    }

    /// Removes the given class from each element in the set of matched elements.
    /// Multiple class names can be specified, separated by a space via multiple arguments.
    pub fn remove_class(&mut self, class: &str) {
        for node in self.nodes() {
            node.remove_class(class);
        }
    }

    /// Returns the number of elements in the selection object.
    pub fn length(&self) -> usize {
        self.nodes().len()
    }

    /// Is an alias for `length`.
    pub fn size(&self) -> usize {
        self.length()
    }

    /// Is there any matched elements.
    pub fn exists(&self) -> bool {
        self.length() > 0
    }

    /// Gets the HTML contents of the first element in the set of matched
    /// elements. It includes the text and comment nodes.
    pub fn outer_html(&self) -> StrTendril {
      match self.nodes().first() {
          Some(node) => node.outer_html(),
          None => StrTendril::new(),
      }
  }
  pub fn inner_html(&self) -> StrTendril {
    match self.nodes().first() {
        Some(node) => node.inner_html(),
        None => StrTendril::new(),
    }
}

    /// Gets the combined text content of each element in the set of matched
    /// elements, including their descendants.
    pub fn text(&self) -> StrTendril {
        let mut s = StrTendril::new();

        for node in self.nodes() {
            s.push_tendril(&node.text());
        }
        s
    }
}
