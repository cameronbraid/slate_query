use html5ever::QualName;
use html5ever::{
    tree_builder::{NoQuirks, TreeBuilderOpts},
    ParseOpts,
};
use markup5ever::{local_name, namespace_url, ns};
use tendril::StrTendril;
use tendril::TendrilSink;

use crate::dom_tree::Tree;
use crate::{document::DocumentTreeSink, Selection};
use crate::{Attrib, Node, NodeData, NodeId};

macro_rules! parse_html {
    ($html: expr) => {
        html5ever::parse_fragment(
            DocumentTreeSink::default(),
            ParseOpts {
                tokenizer: Default::default(),
                tree_builder: TreeBuilderOpts {
                    exact_errors: false,
                    scripting_enabled: true,
                    iframe_srcdoc: false,
                    drop_doctype: true,
                    ignore_missing_rules: false,
                    quirks_mode: NoQuirks,
                },
            },
            QualName::new(None, ns!(html), local_name!("")),
            Vec::new(),
        )
        .one($html)
    };
}

impl<'a> Selection<'a> {
    /// Removes the set of matched elements from the document.
    pub fn remove(&mut self) {
        for node in &self.nodes {
            node.remove_from_parent()
        }
    }

    // / Set the html contents of each element in the selection to specified parsed HTML.
    pub fn set_slate_html<T>(&mut self, html: T)
    where
        T: Into<StrTendril>,
    {
        for node in self.nodes() {
            node.remove_children();
        }

        self.append_slate_html(html)
    }

    /// Replaces each element in the set of matched elements with
    /// the parsed HTML.
    /// It returns the removed elements.
    ///
    /// This follows the same rules as `append`.
    pub fn replace_with_html<T>(&mut self, html: T)
    where
        T: Into<StrTendril>,
    {
        let dom = parse_html!(html);

        for (i, node) in self.nodes().iter().enumerate() {
            if i + 1 == self.size() {
                node.append_prev_siblings_from_another_tree(dom.tree);
                break;
            } else {
                node.append_prev_siblings_from_another_tree(dom.tree.clone());
            }
        }

        self.remove()
    }

    /// Replaces each element in the set of matched element with
    /// the nodes from the given selection.
    ///
    /// This follows the same rules as `append`.
    pub fn replace_with_selection(&mut self, sel: &Selection) {
        for node in self.nodes() {
            for prev_sibling in sel.nodes() {
                node.append_prev_sibling(&prev_sibling.id);
            }
        }

        self.remove()
    }

    // / Parses the html and appends it to the set of matched elements.
    pub fn append_slate_html<T>(&mut self, html: T)
    where
        T: Into<StrTendril>,
    {
        let dom = parse_html!(html);

        for (i, node) in self.nodes().iter().enumerate() {
            if i + 1 == self.size() {
                node.append_children_from_another_tree(dom.tree);
                break;
            } else {
                node.append_children_from_another_tree(dom.tree.clone());
            }
        }
    }
    pub fn append_text_contents<T: Into<StrTendril>>(&mut self, text: T) {
        let text = text.into();
        for (i, node) in self.nodes().iter().enumerate() {
            if i + 1 == self.size() {
                node.append_text_contents(text);
                break;
            } else {
                node.append_text_contents(text.clone());
            }
        }
    }
    pub fn set_text_contents<T: Into<StrTendril>>(&mut self, text: T) {
        let text = text.into();
        for (i, node) in self.nodes().iter().enumerate() {
            if i + 1 == self.size() {
                node.set_text_contents(text);
                break;
            } else {
                node.set_text_contents(text.clone());
            }
        }
    }
    pub fn set_text_attrs(&mut self, attrs: Vec<Attrib>) {
        for (i, node) in self.nodes().iter().enumerate() {
            if i + 1 == self.size() {
                node.set_text_attrs(attrs);
                break;
            } else {
                node.set_text_attrs(attrs.clone());
            }
        }
    }

    /// Appends the elements in the selection to the end of each element
    /// in the set of matched elements.
    pub fn append_selection(&mut self, sel: &Selection) {
        for node in self.nodes() {
            for child in sel.nodes() {
                node.append_child(&child.id);
            }
        }
    }

    pub fn append_first_child(&mut self, content: impl Into<NodeData>) {
        let content = content.into();
        for (i, node) in self.nodes().into_iter().enumerate() {
            if i + 1 == self.size() {
                node.append_first_child(content);
                break;
            } else {
                node.append_first_child(content.clone());
            }
        }
    }

    pub fn append_last_child(&mut self, content: impl Into<NodeData>) {
        let content = content.into();
        for (i, node) in self.nodes().into_iter().enumerate() {
            if i + 1 == self.size() {
                node.append_last_child(content);
                break;
            } else {
                node.append_last_child(content.clone());
            }
        }
    }

    pub fn insert_before(&mut self, content: impl Into<NodeData>) {
        let content = content.into();
        for (i, node) in self.nodes().into_iter().enumerate() {
            if i + 1 == self.size() {
                node.insert_before(content);
                break;
            } else {
                node.insert_before(content.clone());
            }
        }
    }
    pub fn insert_after(&mut self, content: impl Into<NodeData>) {
        let content = content.into();
        for (i, node) in self.nodes().into_iter().enumerate() {
            if i + 1 == self.size() {
                node.insert_after(content);
                break;
            } else {
                node.insert_after(content.clone());
            }
        }
    }
}

impl<'a> Node<'a> {
    #[inline]
    pub fn append_text_contents(&self, text: impl Into<StrTendril>) {
        self.tree.append_text_contents(&self.id, text)
    }
    #[inline]
    pub fn set_text_contents(&self, text: impl Into<StrTendril>) {
        self.tree.set_text_contents(&self.id, text)
    }
    #[inline]
    pub fn set_text_attrs(&self, attrs: Vec<Attrib>) {
        self.tree.set_text_attrs(&self.id, attrs)
    }

    pub fn append_first_child(&self, content: NodeData) {
        let new_node = self.tree.create_node(content);
        if let Some(first_child) = self.tree.first_child_of(&self.id) {
            self.tree.append_prev_sibling_of(&first_child.id, &new_node)
        } else {
            self.tree.append_child_of(&self.id, &new_node)
        }
    }

    pub fn append_last_child(&self, content: NodeData) {
        let new_node = self.tree.create_node(content);
        self.tree.append_child_of(&self.id, &new_node)
    }

    pub fn insert_before(&self, content: NodeData) {
        let new_node = self.tree.create_node(content);
        self.tree.append_prev_sibling_of(&self.id, &new_node)
    }

    pub fn insert_after(&self, content: NodeData) {
        let new_node = self.tree.create_node(content);

        // get the next sibling of the current node
        if let Some(next_sibling) = self.tree.next_sibling_of(&self.id) {
            eprint!("next_sibling {:?}", next_sibling);
            self.tree
                .append_prev_sibling_of(&next_sibling.id, &new_node)
        } else {
            // if there is no next sibling, append the new node to the parent
            if let Some(parent) = self.tree.parent_of(&self.id) {
                eprint!("parent {:?}", parent.id);
                self.tree.append_child_of(&parent.id, &new_node)
            } else {
                // the parent is the root
                eprint!("root");
                self.tree.append_child_of(&self.tree.root_id(), &new_node)
            }
        }
    }
}

impl Tree<NodeData> {
    fn append_text_contents(&self, id: &NodeId, text: impl Into<StrTendril>) {
        self.update_node(id, |node| match node.data {
            NodeData::Text(ref mut text_node) => {
                text_node.contents.push_slice(&text.into());
            }
            _ => {}
        });
    }
    fn set_text_contents(&self, id: &NodeId, text: impl Into<StrTendril>) {
        self.update_node(id, |node| match node.data {
            NodeData::Text(ref mut text_node) => {
                text_node.contents = text.into();
            }
            _ => {}
        });
    }
    fn set_text_attrs(&self, id: &NodeId, attrs: Vec<Attrib>) {
        self.update_node(id, |node| match node.data {
            NodeData::Text(ref mut text_node) => {
                text_node.attrs = attrs;
            }
            _ => {}
        });
    }
}
