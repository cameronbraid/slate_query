use core::panic;
use std::borrow::Cow;
use std::ops::Deref;

use crate::dom_tree::{children_of, InnerNode, NodeRef, Tree};
use crate::Node;
use html5ever::serialize::{Serialize, Serializer, TraversalScope};
use html5ever::{parse_document, LocalName};
use markup5ever::interface::tree_builder;
use markup5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use markup5ever::Attribute;
use markup5ever::ExpandedName;
use markup5ever::QualName;
use tendril::StrTendril;
use tendril::TendrilSink;

use crate::entities::{HashSetFx, NodeId, NodeIdMap};

use markup5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use markup5ever::{local_name, namespace_url, ns};
use std::{io, usize};

/// Document represents an HTML document to be manipulated.
pub struct Document {
    /// The document's dom tree.
    pub(crate) tree: Tree<NodeData>,

    /// Errors that occurred during parsing.
    pub errors: Vec<Cow<'static, str>>,

    /// The document's quirks mode.
    pub quirks_mode: QuirksMode,
}

impl Default for Document {
    fn default() -> Document {
        Self {
            tree: Tree::new(NodeData::Document),
            errors: vec![],
            quirks_mode: tree_builder::NoQuirks,
        }
    }
}

impl Document {
    pub fn from_slate_html(html: &str) -> Document {
        parse_document(DocumentTreeSink::default(), Default::default()).one(html)
    }
}

impl Document {
    /// Return the underlying root document node.
    #[inline]
    pub fn root(&self) -> NodeRef<NodeData> {
        self.tree.root()
    }
}

pub struct DocumentTreeSink(Document, NodeIdMap);
impl Default for DocumentTreeSink {
    fn default() -> Self {
        Self(
            Document::default(),
            Default::default(),
            // AtomicUsize::new(usize::MAX),
        )
    }
}
impl DocumentTreeSink {
    fn ignored(&self) -> NodeId {
        NodeId::new(usize::MAX)
        // NodeId::new(self.2.fetch_sub(1, std::sync::atomic::Ordering::Release))
    }
    fn is_ignored(&self, id: &NodeId) -> bool {
        // id.value > self.2.load(std::sync::atomic::Ordering::Acquire)
        id.value == usize::MAX
    }
}
impl TreeSink for DocumentTreeSink {
    // The overall result of parsing.
    type Output = Document;

    // Consume this sink and return the overall result of parsing.
    #[inline]
    fn finish(self) -> Document {
        self.0
    }

    // Handle is a reference to a DOM node. The tree builder requires that a `Handle` implements `Clone` to get
    // another reference to the same node.
    type Handle = NodeId;

    // Signal a parse error.
    #[inline]
    fn parse_error(&mut self, msg: Cow<'static, str>) {
        self.0.errors.push(msg);
    }

    // Get a handle to the `Document` node.
    #[inline]
    fn get_document(&mut self) -> NodeId {
        self.0.tree.root_id()
    }

    // Get a handle to a template's template contents. The tree builder promises this will never be called with
    // something else than a template element.
    #[inline]
    fn get_template_contents(&mut self, _target: &NodeId) -> NodeId {
        self.ignored()
    }

    // Set the document's quirks mode.
    #[inline]
    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.0.quirks_mode = mode;
    }

    // Do two handles refer to the same node?.
    #[inline]
    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        *x == *y
    }

    // What is the name of the element?
    // Should never be called on a non-element node; Feel free to `panic!`.
    #[inline]
    fn elem_name(&self, target: &NodeId) -> ExpandedName {
        self.1.get(target).expect("not an element").expanded()
    }

    // Create an element.
    // When creating a template element (`name.ns.expanded() == expanded_name!(html"template")`), an
    // associated document fragment called the "template contents" should also be created. Later calls to
    // self.get_template_contents() with that given element return it. See `the template element in the whatwg spec`,
    #[inline]
    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> NodeId {
        if flags.template {
            panic!("not implemented")
        }

        if name.local.deref() == "text" {
            let id = self.0.tree.create_node(NodeData::Text(Text::with_attrs(
                "",
                attrs.into_iter().map(Into::into).collect(),
            )));
            self.1.insert(id, name.clone());
            return id;
        }

        let id = self.0.tree.create_node(NodeData::Element(Element::with_attrs(
            name.local.to_string(),
            attrs.into_iter().map(Into::into).collect(),
        )));
        self.1.insert(id, name.clone());
        id
    }

    // Create a comment node.
    #[inline]
    fn create_comment(&mut self, _text: StrTendril) -> NodeId {
        self.ignored()
    }

    // Create a Processing Instruction node.
    #[inline]
    fn create_pi(&mut self, _target: StrTendril, _data: StrTendril) -> NodeId {
        self.ignored()
    }

    // Append a node as the last child of the given node. If this would produce adjacent sibling text nodes, it
    // should concatenate the text instead.
    // The child node will not already have a parent.
    fn append(&mut self, parent: &NodeId, child: NodeOrText<NodeId>) {
        // Append to an existing Text node if we have one.
        if self.is_ignored(parent) {
            return;
        }

        match child {
            NodeOrText::AppendNode(node_id) => {
                if self.is_ignored(&node_id) {
                    return;
                }
                self.0.tree.append_child_of(parent, &node_id)
            }
            NodeOrText::AppendText(text) => {
                // text should only come as a childof <text>, if not wrap in <text>
                if let Some(node) = self.0.tree.get(parent) {
                    if node.is_text() {
                        self.0
                            .tree
                            .update_node(parent, |node| append_to_existing_text(node, &text));
                    } else {
                        if let Some(first_child) = self.0.tree.first_child_of(parent) {
                            self.0.tree.update_node(&first_child.id, |node| {
                                append_to_existing_text(node, &text)
                            });
                        } else {
                            let node = self.0.tree.create_node(NodeData::Text(Text::new(text)));
                            self.0.tree.append_child_of(parent, &node);
                        }
                    }
                }
            }
        }
    }

    // Append a node as the sibling immediately before the given node.
    // The tree builder promises that `sibling` is not a text node. However its old previous sibling, which would
    // become the new node's previous sibling, could be a text node. If the new node is also a text node, the two
    // should be merged, as in the behavior of `append`.
    fn append_before_sibling(&mut self, sibling: &NodeId, child: NodeOrText<NodeId>) {
        if self.is_ignored(sibling) {
            return;
        }
        match child {
            NodeOrText::AppendText(text) => {
                let prev_sibling = self.0.tree.prev_sibling_of(sibling);
                let merged = prev_sibling
                    .and_then(|sibling| {
                        self.0
                            .tree
                            .update_node(&sibling.id, |node| append_to_existing_text(node, &text))
                    })
                    .unwrap_or(false);

                if merged {
                    return;
                }

                let id = self.0.tree.create_node(NodeData::Text(Text::new(text)));
                self.0.tree.append_prev_sibling_of(sibling, &id);
            }

            // The tree builder promises we won't have a text node after
            // the insertion point.

            // Any other kind of node.
            NodeOrText::AppendNode(id) => {
                if self.is_ignored(&id) {
                    return;
                }
                self.0.tree.append_prev_sibling_of(sibling, &id)
            }
        };
    }

    // When the insertion point is decided by the existence of a parent node of the element, we consider both
    // possibilities and send the element which will be used if a parent node exists, along with the element to be
    // used if there isn't one.
    fn append_based_on_parent_node(
        &mut self,
        element: &NodeId,
        prev_element: &NodeId,
        child: NodeOrText<NodeId>,
    ) {
        if self.is_ignored(element) {
            return;
        }

        let has_parent = self.0.tree.parent_of(element).is_some();

        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    // Append a `DOCTYPE` element to the `Document` node.
    #[inline]
    fn append_doctype_to_document(
        &mut self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
    }

    // Add each attribute to the given element, if no attribute with that name already exists. The tree builder
    // promises this will never be called with something else than an element.
    fn add_attrs_if_missing(&mut self, target: &NodeId, attrs: Vec<Attribute>) {
        if self.is_ignored(target) {
            return;
        }

        let attrs: Vec<Attrib> = attrs.into_iter().map(Into::into).collect();

        self.0.tree.update_node(target, |node| {
            let existing = if let NodeData::Element(Element { ref mut attrs, .. }) = node.data {
                attrs
            } else {
                panic!("not an element")
            };
            let existing_names = existing
                .iter()
                .map(|e| e.name.to_string())
                .collect::<HashSetFx<_>>();

            existing.extend(
                attrs
                    .into_iter()
                    .filter(|attr| !existing_names.contains(&attr.name[..])),
            );
        });
    }

    // Detach the given node from its parent.
    #[inline]
    fn remove_from_parent(&mut self, target: &NodeId) {
        if self.is_ignored(target) {
            return;
        }

        self.0.tree.remove_from_parent(target);
    }

    // Remove all the children from node and append them to new_parent.
    #[inline]
    fn reparent_children(&mut self, node: &NodeId, new_parent: &NodeId) {
        if self.is_ignored(node) {
            return;
        }
        if self.is_ignored(new_parent) {
            self.0.tree.reparent_children_of(node, None);
        }
        self.0.tree.reparent_children_of(node, Some(*new_parent));
    }
}

fn append_to_existing_text(prev: &mut InnerNode<NodeData>, text: &str) -> bool {
    match prev.data {
        NodeData::Text(ref mut t) => {
            t.contents.push_slice(text);
            true
        }
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct Attrib {
    pub name: StrTendril,
    pub value: serde_json::Value,
}

impl Attrib {
    pub fn get_value_as_str_if_string(&self) -> Option<StrTendril> {
        match self.value {
            serde_json::Value::String(ref s) => Some(StrTendril::from(s.as_str())),
            _ => None,
        }
    }
    pub fn set_value(&mut self, s: StrTendril) {
        self.value = parse_json_or_use_as_string(&s);
    }
}

fn parse_json_or_use_as_string(value: &str) -> serde_json::Value {
    if let Ok(value) = serde_json::from_str(&value) {
        return value;
    }
    serde_json::Value::String(value.into())
}
impl From<html5ever::Attribute> for Attrib {
    fn from(attr: html5ever::Attribute) -> Self {
        let name = attr.name.local.to_string().into();
        Self {
            name,
            value: parse_json_or_use_as_string(&attr.value),
        }
    }
}

/// The different kinds of nodes in the DOM.
#[derive(Debug, Clone)]
pub enum NodeData {
    /// The `Tree` itself - the root node of a HTML tree.
    Document,

    /// Text with attributes.
    Text(Text),

    /// An element with attributes.
    Element(Element),
    // // a ghost node for representing comment/pi/doctype which are ignored when parsing
    // Ghost,
}

impl From<Text> for NodeData {
    fn from(text: Text) -> Self {
        NodeData::Text(text)
    }
}
impl From<Element> for NodeData {
    fn from(element: Element) -> Self {
        NodeData::Element(element)
    }
}
/// An element with attributes.
#[derive(Debug, Clone)]
pub struct Text {
    pub contents: StrTendril,
    pub attrs: Vec<Attrib>,
}

/// An element with attributes.
#[derive(Debug, Clone)]
pub struct Element {
    pub name: StrTendril,
    pub attrs: Vec<Attrib>,
}

impl Element {
    pub fn new(name: impl Into<StrTendril>) -> Self {
        Self { name : name.into(), attrs:vec![] }
    }
    pub fn with_attrs(name: impl Into<StrTendril>, attrs: Vec<Attrib>) -> Self {
        Self {
            name: name.into(),
            attrs,
        }
    }
    pub(crate) fn html_attribs(&self) -> Vec<(QualName, String)> {
        html_attribs(&self.attrs)
    }
    // pub(crate) fn set_attr_parse_json_or_use_as_string(&mut self, name: &str, value: &str) {
    //     set_attr_parse_json_or_use_as_string(&mut self.attribs, name, value);
    // }

    pub fn set_attr(&mut self, name: &str, value: serde_json::Value) {
        set_attr(&mut self.attrs, name, value);
    }

    pub fn remove_attr(&mut self, name: &str) {
        remove_attr(&mut self.attrs, name);
    }
}
impl Text {
    pub fn new(contents: impl Into<StrTendril>) -> Self {
        Self {
            contents: contents.into(),
            attrs: vec![],
        }
    }
    pub fn with_attrs(contents: impl Into<StrTendril>, attrs: Vec<Attrib>) -> Self {
        Self {
            contents: contents.into(),
            attrs,
        }
    }
    pub(crate) fn html_attribs(&self) -> Vec<(QualName, String)> {
        html_attribs(&self.attrs)
    }
    // pub(crate) fn set_attr_parse_json_or_use_as_string(&mut self, name: &str, value: &str) {
    //     set_attr_parse_json_or_use_as_string(&mut self.attrs, name, value);
    // }

    pub fn set_attr(&mut self, name: &str, value: serde_json::Value) {
        set_attr(&mut self.attrs, name, value);
    }

    pub fn remove_attr(&mut self, name: &str) {
        remove_attr(&mut self.attrs, name);
    }
}

fn remove_attr(attrs: &mut Vec<Attrib>, name: &str) {
    attrs.retain(|a| &a.name[..] != name);
}
// fn set_attr_parse_json_or_use_as_string(attrs: &mut Vec<Attrib>, name: &str, value: &str) {
//     if let Some(attr) = attrs.iter_mut().find(|a| &a.name[..] == name) {
//         attr.value = parse_json_or_use_as_string(value);
//     } else {
//         attrs.push(Attrib {
//             name: name.into(),
//             value: parse_json_or_use_as_string(value),
//         });
//     }
// }

fn set_attr(attrs: &mut Vec<Attrib>, name: &str, value: serde_json::Value) {
    if let Some(attr) = attrs.iter_mut().find(|a| &a.name[..] == name) {
        attr.value = value.to_owned();
    } else {
        attrs.push(Attrib {
            name: name.into(),
            value: value.to_owned(),
        });
    }
}

fn html_attribs(attrs: &Vec<Attrib>) -> Vec<(QualName, String)> {
    attrs
        .iter()
        .map(|attr| {
            if let Some(str) = attr.get_value_as_str_if_string() {
                let qual = QualName::new(None, ns!(), LocalName::from(&attr.name[..]));
                (qual, str.to_string())
            } else {
                let qual = QualName::new(None, ns!(), LocalName::from(&attr.name[..]));
                (qual, attr.value.to_string())
            }
        })
        .collect::<Vec<_>>()
}

enum SerializeOp {
    Open(NodeId),
    Close(StrTendril),
}
/// Serializable wrapper of Node.
pub struct SerializableNodeRef<'a>(Node<'a>);

impl<'a> From<NodeRef<'a, NodeData>> for SerializableNodeRef<'a> {
    fn from(h: NodeRef<'a, NodeData>) -> SerializableNodeRef {
        SerializableNodeRef(h)
    }
}

impl<'a> Serialize for SerializableNodeRef<'a> {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        let nodes = self.0.tree.nodes.borrow();
        let id = self.0.id;
        let mut ops = match traversal_scope {
            IncludeNode => vec![SerializeOp::Open(id)],
            ChildrenOnly(_) => children_of(&nodes, &id)
                .into_iter()
                .map(SerializeOp::Open)
                .collect(),
        };

        while !ops.is_empty() {
            match ops.remove(0) {
                SerializeOp::Open(id) => {
                    let node_opt = &nodes.get(id.value);
                    let node = match node_opt {
                        Some(node) => node,
                        None => continue,
                    };

                    match node.data {
                        // NodeData::Ghost => Ok(()),
                        NodeData::Element(ref e) => {
                            let html_attribs = e.html_attribs();
                            serializer.start_elem(
                                QualName::new(None, ns!(html), LocalName::from(&e.name[..])),
                                html_attribs.iter().map(|v| (&v.0, &v.1[..])),
                            )?;

                            ops.insert(0, SerializeOp::Close(e.name.clone()));

                            for child_id in children_of(&nodes, &id).into_iter().rev() {
                                ops.insert(0, SerializeOp::Open(child_id));
                            }

                            Ok(())
                        }
                        NodeData::Text(ref e) => {
                            let html_attribs = e.html_attribs();
                            serializer.start_elem(
                                QualName::new(None, ns!(html), local_name!("text")),
                                html_attribs.iter().map(|v| (&v.0, &v.1[..])),
                            )?;

                            ops.insert(0, SerializeOp::Close("text".into()));

                            serializer.write_text(&e.contents)?;

                            Ok(())
                        }
                        NodeData::Document => {
                            for child_id in children_of(&nodes, &id).into_iter().rev() {
                                ops.insert(0, SerializeOp::Open(child_id));
                            }
                            continue;
                        }
                    }
                }
                SerializeOp::Close(name) => {
                    serializer.end_elem(QualName::new(None, ns!(html), LocalName::from(&name[..])))
                }
            }?
        }

        Ok(())
    }
}

impl Tree<NodeData> {
    pub fn set_name(&mut self, id: NodeId, name: StrTendril) {
        self.update_node(&id, |node| {
            if let NodeData::Element(ref mut e) = node.data {
                e.name = name;
            }
        });
    }

    pub fn get_name(&self, id: &NodeId) -> Option<StrTendril> {
        self.query_node(id, |node| match node.data {
            NodeData::Element(ref e) => Some(e.name.clone()),
            _ => None,
        })
        .flatten()
    }
}
