use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use bdk::bitcoin::secp256k1::{rand, SecretKey};
use bdk::bitcoin::{Network, PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};

mod blockly;
mod storage;

use crate::evt::{EventBus, Request};
use blockly::*;

#[derive(Debug, PartialEq, Properties)]
pub struct KeymanagerProps;

#[derive(Debug, PartialEq)]
pub enum KeymanagerMsg {
    FirstRender,
    NewInputNameChanged(InputEvent),
    NewInputKeyChanged(InputEvent),
    AddKey,
    RemoveKey(String),

    Compiled(String),
}

pub struct Keymanager {
    new_input_name: String,
    new_input_key: String,
    key_err: Option<String>,

    state: Rc<RefCell<State>>,

    compiled_cb: Closure<dyn FnMut(String)>,
    dropdown_cb: Closure<dyn FnMut() -> JsValue>,

    dispatcher: Dispatcher<EventBus>,
}

impl Keymanager {
    fn local_key(&self, _ctx: &Context<Self>) -> Html {
        // TODO: to public key
        html! {
            <div class="row">
                <input type="text" disabled=true value={self.state.borrow().local_key.to_string()} />
            </div>
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    local_key: PrivateKey,
    map: HashMap<String, PublicKey>,
}

impl State {
    fn new() -> Self {
        let local_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            inner: SecretKey::new(&mut rand::thread_rng()),
        };

        let map = vec![(
            "example_key".into(),
            PublicKey::from_str(
                "038375785e012a64dbf86fb4cfe6b9c71e2d4677fddf7a15ceed7de5d36e23ec1a",
            )
            .unwrap(),
        )]
        .into_iter()
        .collect();
        State { local_key, map }
    }
}

impl Component for Keymanager {
    type Message = KeymanagerMsg;
    type Properties = KeymanagerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let compiled_cb = Closure::new(move |s: String| {
            link.send_message(KeymanagerMsg::Compiled(s));
        });

        let state = Rc::new(RefCell::new(State::new()));
        let state_cloned = Rc::clone(&state);
        let dropdown_cb = Closure::new(move || {
            let state_cloned = state_cloned.borrow();
            let dropdown = state_cloned
                .map
                .iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect::<Vec<_>>();
            log::debug!("{:?}", dropdown);
            serde_wasm_bindgen::to_value(&dropdown).unwrap()
        });

        Keymanager {
            new_input_name: String::new(),
            new_input_key: String::new(),
            key_err: None,

            state,

            compiled_cb,
            dropdown_cb,

            dispatcher: EventBus::dispatcher(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput_name = ctx
            .link()
            .callback(move |e: InputEvent| KeymanagerMsg::NewInputNameChanged(e));
        let oninput_key = ctx
            .link()
            .callback(move |e: InputEvent| KeymanagerMsg::NewInputKeyChanged(e));
        let onclick_add = ctx.link().callback(|_| KeymanagerMsg::AddKey);

        html! {
            <div class="container" style="height: 800px;">
                <div class="row" style="height: 100%;">
                    <div id="blocklyArea" class="col-8" style="height: 100%;">
                        <div id="blocklyDiv" style="position: absolute;"></div>
                    </div>
                    <div class="col-4">
                        <h2>{ "Local Key" }</h2>
                        { self.local_key(ctx) }

                        <h2>{ "Remote Keys" }</h2>
                        { for self.state.borrow().map.iter().map(|(name, key)| {
                                let name_cloned = name.clone();
                                let remove_onclick = ctx.link().callback_once(move |_| KeymanagerMsg::RemoveKey(name_cloned));
                                html! {
                                    <div class="row mb-1">
                                        <span class="col-3">{ name.clone() }</span>
                                        <span class="col-7">{ key.clone() }</span>
                                        <button type="button" onclick={remove_onclick} disabled={name == "example_key"} class="col-2 btn btn-primary"><i class="bi bi-trash"></i></button>
                                    </div>
                                }
                            })
                        }
                        <div class="row input-grup has-validation">
                            <input type={"text"} oninput={oninput_name} placeholder={"Name"} value={self.new_input_name.clone()} class="col-3" />
                            <input type={"text"} oninput={oninput_key} placeholder={"Key"} value={self.new_input_key.clone()} class={classes!("col-7", self.key_err.as_ref().map(|_| "is-invalid"))} />
                            <button type={"button"} class="btn btn-primary col-2" onclick={onclick_add} disabled={self.new_input_name.is_empty() || self.new_input_key.is_empty() || self.key_err.is_some()}>
                              <i class="ms-2 bi bi-plus-square"></i>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            KeymanagerMsg::FirstRender => {
                let control_flow = vec![
                    BlocklyBlock {
                        ty: "and",
                        message0: "%1 AND %2 %3",
                        args0: vec![BlocklyBlockArg::input_statement("A", ValueType::Policy), BlocklyBlockArg::dummy(), BlocklyBlockArg::input_statement("B", ValueType::Policy)],
                        output: None,
                        next_statement: Some(ValueType::Policy), // disabled on creation by blockly-ext
                        previous_statement: Some(ValueType::Policy),
                        inputs_inline: false,
                        tooltip: Some("Requires both sub-policies to be satisfied"),
                        colour: 230,
                        extensions: vec!["allow_chain_in_thresh"],
                    },
                    BlocklyBlock {
                        ty: "or",
                        message0: "Weight %1 %2 OR %3 Weight %4 %5",
                        args0: vec![
                            BlocklyBlockArg::field_number("A_weight", "1"),
                            BlocklyBlockArg::input_statement("A", ValueType::Policy),
                            BlocklyBlockArg::dummy(),
                            BlocklyBlockArg::field_number("B_weight", "1"),
                            BlocklyBlockArg::input_statement("B", ValueType::Policy),
                        ],
                        output: None,
                        next_statement: Some(ValueType::Policy), // disabled on creation by blockly-ext
                        previous_statement: Some(ValueType::Policy),
                        inputs_inline: false,
                        tooltip: Some("Requires either one of the two sub-policies to be satisfied. Weights can be used to indicate the relative probability of each sub-policy"),
                        colour: 230,
                        extensions: vec!["allow_chain_in_thresh"],
                    },
                    BlocklyBlock {
                        ty: "thresh",
                        message0: "Threshold %1 %2 %3",
                        args0: vec![
                            BlocklyBlockArg::field_number("Threshold", "1"),
                            BlocklyBlockArg::dummy(),
                            BlocklyBlockArg::input_statement("Statements", ValueType::Policy),
                        ],
                        output: None,
                        next_statement: Some(ValueType::Policy), // disabled on creation by blockly-ext
                        previous_statement: Some(ValueType::Policy),
                        inputs_inline: false,
                        tooltip: Some("Creates a threshold element (m-of-n), where the 'm' field is manually set and 'n' is implied by the number of sub-policies added"),
                        colour: 230,
                        extensions: vec!["allow_chain_in_thresh"],
                    },
                ];
                let begin = vec![BlocklyBlock {
                    ty: "begin",
                    message0: "Begin %1",
                    args0: vec![BlocklyBlockArg::dummy()],
                    output: None,
                    next_statement: Some(ValueType::Policy),
                    previous_statement: None,
                    inputs_inline: false,
                    tooltip: Some("Sets the beginning of the policy"),
                    colour: 160,
                    extensions: vec![],
                }];
                let keys = vec![
                    BlocklyBlock {
                        ty: "my_key",
                        message0: "My Key",
                        args0: vec![],
                        output: Some(ValueType::Key),
                        next_statement: None,
                        previous_statement: None,
                        inputs_inline: false,
                        tooltip: Some("My private key"),
                        colour: 22,
                        extensions: vec![],
                    },
                    BlocklyBlock {
                        ty: "key",
                        message0: "%1",
                        args0: vec![BlocklyBlockArg::field_number("Key", "")],
                        output: Some(ValueType::Key),
                        next_statement: None,
                        previous_statement: None,
                        inputs_inline: false,
                        tooltip: Some("Somebody else's public key"),
                        colour: 65,
                        extensions: vec!["dynamic_options"],
                    },
                ];
                let leaves = vec![
                    BlocklyBlock {
                        ty: "pk",
                        message0: "Key %1",
                        args0: vec![BlocklyBlockArg::input_value("Key", ValueType::Key)],
                        output: None,
                        next_statement: Some(ValueType::Policy), // disabled on creation by blockly-ext
                        previous_statement: Some(ValueType::Policy),
                        inputs_inline: false,
                        tooltip: Some(
                            "Require a signature from a given key to satisfy this fragment",
                        ),
                        colour: 120,
                        extensions: vec!["allow_chain_in_thresh"],
                    },
                    BlocklyBlock {
                        ty: "older",
                        message0: "Older %1 %2",
                        args0: vec![
                            BlocklyBlockArg::field_number("value", "6"),
                            BlocklyBlockArg::dummy(),
                        ],
                        output: None,
                        next_statement: Some(ValueType::Policy), // disabled on creation by blockly-ext
                        previous_statement: Some(ValueType::Policy),
                        inputs_inline: false,
                        tooltip: Some("Add a relative timelock expressed in number of blocks"),
                        colour: 150,
                        extensions: vec!["allow_chain_in_thresh"],
                    },
                    BlocklyBlock {
                        ty: "after",
                        message0: "After %1 %2",
                        args0: vec![
                            BlocklyBlockArg::field_number("value", "10000"),
                            BlocklyBlockArg::dummy(),
                        ],
                        output: None,
                        next_statement: Some(ValueType::Policy), // disabled on creation by blockly-ext
                        previous_statement: Some(ValueType::Policy),
                        inputs_inline: false,
                        tooltip: Some("Add a relative timelock expressed in absolute block height"),
                        colour: 150,
                        extensions: vec!["allow_chain_in_thresh"],
                    },
                ];

                define_blocks(
                    control_flow
                        .iter()
                        .chain(begin.iter())
                        .chain(keys.iter())
                        .chain(leaves.iter()),
                );

                let workspace = inject_blockly(
                    "blocklyDiv",
                    &BlocklyOptions {
                        toolbox: BlocklyToolbox {
                            kind: "categoryToolbox",
                            contents: vec![
                                BlocklyToolboxCategory::new(
                                    "Control Flow",
                                    230,
                                    control_flow.iter().map(|b| b.ty),
                                ),
                                BlocklyToolboxCategory::new(
                                    "Leaves",
                                    120,
                                    leaves.iter().map(|b| b.ty),
                                ),
                                BlocklyToolboxCategory::new("Keys", 65, keys.iter().map(|b| b.ty)),
                            ],
                        },
                        trashcan: true,
                    },
                );

                if let Some(state) = storage::load() {
                    *self.state.borrow_mut() = state;
                } else {
                    storage::save(&self.state.borrow());
                }

                init_js(&workspace, &self.compiled_cb, &self.dropdown_cb);

                enable_backup();
                restore_blocks();

                insert_begin(&workspace);

                false
            }
            KeymanagerMsg::NewInputNameChanged(e) => {
                self.new_input_name = e.target_unchecked_into::<HtmlInputElement>().value();
                true
            }
            KeymanagerMsg::NewInputKeyChanged(e) => {
                self.key_err = None;
                self.new_input_key = e.target_unchecked_into::<HtmlInputElement>().value();
                true
            }
            KeymanagerMsg::AddKey => {
                match PublicKey::from_str(&self.new_input_key) {
                    Ok(key) => {
                        self.state
                            .borrow_mut()
                            .map
                            .insert(self.new_input_name.clone(), key);

                        self.new_input_name = String::new();
                        self.new_input_key = String::new();

                        storage::save(&self.state.borrow());
                    }
                    Err(e) => {
                        self.key_err = Some(e.to_string());
                    }
                }

                true
            }
            KeymanagerMsg::RemoveKey(key) => {
                self.state.borrow_mut().map.remove(&key);
                storage::save(&self.state.borrow());

                true
            }

            KeymanagerMsg::Compiled(desc) => {
                let desc = desc.replace("_MY_KEY", &self.state.borrow().local_key.to_string());
                log::info!("{}", desc);
                self.dispatcher.send(Request::EventBusMsg(desc));
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // Delay to the next tick to ensure the DOM has been updated before we try to
            // initialize blockly
            ctx.link().send_message(KeymanagerMsg::FirstRender);
        }
    }
}
