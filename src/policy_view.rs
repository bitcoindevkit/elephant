use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::policy_node::PolicyNode;
use bdk::descriptor::Policy;
use yew::prelude::*;

#[derive(Clone, Default, PartialEq)]
pub struct Selection(Rc<RefCell<BTreeMap<String, Vec<usize>>>>);

impl Selection {
    pub fn select(&self, id: String, index: usize) {
        log::info!("Selected {} {}", id, index);
        let map = &mut *self.0.borrow_mut();
        let selected = map.entry(id).or_default();
        if !selected.contains(&index) {
            selected.push(index)
        }
    }

    pub fn deselect(&self, id: String, index: usize) {
        let map = &mut *self.0.borrow_mut();
        if let Some(selected) = map.get_mut(&id) {
            let rm_index =
                selected
                    .iter()
                    .enumerate()
                    .find_map(|(i, &sel)| if sel == index { Some(i) } else { None });
            if let Some(rm_index) = rm_index {
                selected.remove(rm_index);
            }
        }
    }

    pub fn extract(&self) -> BTreeMap<String, Vec<usize>> {
        self.0.borrow().clone()
    }
}

#[derive(PartialEq, Properties)]
pub struct PolicyViewProps {
    pub selection: Selection,
    pub node: Policy,

    #[prop_or_default]
    pub nodes: ChildrenWithProps<PolicyNode>,
}

pub enum PolicyViewMsg {}

pub struct PolicyView;

impl Component for PolicyView {
    type Message = PolicyViewMsg;

    type Properties = PolicyViewProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let body_html = {
            let selection = props.selection.clone();
            let node = props.node.clone();
            let depth = 0_u32;

            html! {
                <PolicyNode {selection} {node} {depth}/>
            }
        };

        html! {
            <div>
                <h4> { "Spending Policy" } </h4>
                { body_html }
            </div>
        }
    }
}
