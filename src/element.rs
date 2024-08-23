use std::ops::Deref;

use markup5ever::{namespace_url, ns};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::context::MatchingContext;
use selectors::matching::{matches_selector_list, ElementSelectorFlags};
use selectors::parser::SelectorImpl;
use selectors::{OpaqueElement, SelectorList};

use crate::css::CssLocalName;
use crate::dom_tree::{Node, NodeRef};
use crate::matcher::{InnerSelector, NonTSPseudoClass};
use crate::{Attrib, NodeData};

impl<'a> selectors::Element for Node<'a> {
    type Impl = InnerSelector;

    // Converts self into an opaque representation.
    #[inline]
    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(&self.id)
    }
    #[inline]
    fn parent_element(&self) -> Option<Self> {
        self.parent()
    }

    // Whether the parent node of this element is a shadow root.
    #[inline]
    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    // The host of the containing shadow root, if any.
    #[inline]
    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    // Whether we're matching on a pseudo-element.
    #[inline]
    fn is_pseudo_element(&self) -> bool {
        false
    }

    // Skips non-element nodes.
    #[inline]
    fn prev_sibling_element(&self) -> Option<Self> {
        self.prev_element_sibling()
    }

    // Skips non-element nodes.
    #[inline]
    fn next_sibling_element(&self) -> Option<Self> {
        self.next_element_sibling()
    }

    #[inline]
    fn is_html_element_in_html_document(&self) -> bool {
        self.query(|node| match node.data {
            NodeData::Element(_) => true,
            NodeData::Text(_) => true,
            _ => false,
        })
        .unwrap_or(false)
    }

    #[inline]
    fn has_local_name(&self, local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return &e.name[..] == local_name.deref();
            }
            if let NodeData::Text(_) = node.data {
                return local_name.deref() == "text";
            }
            false
        })
        .unwrap_or(false)
    }

    // Empty string for no namespace.
    #[inline]
    fn has_namespace(&self, ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        self.query(|node| match node.data {
            NodeData::Element(_) => {
                return ns == &ns!(html);
            }
            NodeData::Text(_) => {
                return ns == &ns!(html);
            }
            _ => false,
        })
        .unwrap_or(false)
    }

    // Whether this element and the `other` element have the same local name and namespace.

    fn is_same_type(&self, other: &Self) -> bool {
        //TODO: maybe we should unpack compare_node directly here
        self.tree
            .compare_node(&self.id, &other.id, |a, b| match (&a.data, &b.data) {
                (NodeData::Element(ref e1), NodeData::Element(ref e2)) => e1.name == e2.name,
                (NodeData::Text(_), NodeData::Text(_)) => true,
                _ => false,
            })
            .unwrap_or(false)
    }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        self.query(|node| match &node.data {
            NodeData::Element(ref e) => attr_matches(&e.attrs, local_name, operation),
            NodeData::Text(ref t) => attr_matches(&t.attrs, local_name, operation),
            _context => false,
        })
        .unwrap_or(false)
    }

    fn match_non_ts_pseudo_class(
        &self,
        pseudo: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        use self::NonTSPseudoClass::*;
        match pseudo {
            Active | Focus | Hover | Enabled | Disabled | Checked | Indeterminate | Visited => {
                false
            }
            AnyLink | Link => match self.node_name() {
                // todo! make the document define what element types are 'links'
                Some(node_name) => {
                    matches!(node_name.deref(), "a" | "area" | "link")
                        && self.attr("href").is_some()
                }
                None => false,
            },
            Has(list) => {
                //it checks only in descendants
                has_descendant_match(self, list, context)
            }
            HasText(s) => self.has_text(s.as_str()),
            Contains(s) => {
                let text = self.text();
                text.contains(s.as_str())
            }
        }
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    // Whether this element is a `link`.
    fn is_link(&self) -> bool {
        // todo! make the document define what element types are 'links'
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return matches!(&e.name[..], "a" | "area" | "link")
                    && e.attrs.iter().any(|attr| &attr.name[..] == "href");
            }

            false
        })
        .unwrap_or(false)
    }

    // Whether the element is an HTML element.
    fn is_html_slot_element(&self) -> bool {
        true
    }

    fn has_id(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return e.attrs.iter().any(|attr| {
                    if let Some(str) = attr.get_value_as_str_if_string() {
                        &attr.name[..] == "id"
                            && case_sensitivity.eq(name.as_bytes(), str.as_bytes())
                    } else {
                        false
                    }
                });
            }

            false
        })
        .unwrap_or(false)
    }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::LocalName,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return e
                    .attrs
                    .iter()
                    .find(|a| &a.name[..] == "class")
                    .and_then(|v| v.get_value_as_str_if_string())
                    .map_or(false, |a| {
                        a.split_whitespace()
                            .any(|c| case_sensitivity.eq(name.as_bytes(), c.as_bytes()))
                    });
            }

            false
        })
        .unwrap_or(false)
    }
    // Returns the mapping from the `exportparts` attribute in the regular direction, that is, outer-tree->inner-tree.
    fn imported_part(&self, _name: &CssLocalName) -> Option<CssLocalName> {
        None
    }

    fn is_part(&self, _name: &CssLocalName) -> bool {
        false
    }

    // Whether this element matches `:empty`.
    fn is_empty(&self) -> bool {
        !self
            .children()
            .iter()
            .any(|child| child.is_element() || child.is_text())
    }

    // Whether this element matches `:root`, i.e. whether it is the root element of a document.
    fn is_root(&self) -> bool {
        self.is_document()
    }

    fn first_element_child(&self) -> Option<Self> {
        self.children()
            .iter()
            .find(|&child| child.is_element())
            .cloned()
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {}
}

fn attr_matches(
    attribs: &Vec<Attrib>,
    local_name: &CssLocalName,
    operation: &AttrSelectorOperation<&crate::css::CssString>,
) -> bool {
    return attribs.iter().any(|attr| {
        if local_name.deref() != &attr.name[..] {
            return false;
        }
        match &attr.value {
            serde_json::Value::String(v) => operation.eval_str(&v),
            serde_json::Value::Number(v) => operation.eval_str(&v.to_string()),
            serde_json::Value::Bool(v) => operation.eval_str(&v.to_string()),
            _ => false,
        }
    });
}

fn has_descendant_match(
    n: &NodeRef<NodeData>,
    selectors_list: &SelectorList<InnerSelector>,
    ctx: &mut MatchingContext<InnerSelector>,
) -> bool {
    let mut node = n.first_child();
    while let Some(ref n) = node {
        if matches_selector_list(selectors_list, n, ctx)
            || ((n.is_element() || n.is_text()) && has_descendant_match(n, selectors_list, ctx))
        {
            return true;
        }
        node = n.next_sibling();
    }
    false
}
