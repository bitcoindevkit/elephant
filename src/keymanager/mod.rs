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
    AddKey,

    LocalKeyInputChanged(InputEvent),
    SetLocalKey,

    RemoveKey(usize),

    Compiled(String),
}

pub struct Keymanager {
    new_input_name: String,
    local_key_input: String,

    state: Rc<RefCell<State>>,

    compiled_cb: Closure<dyn FnMut(String)>,
    dropdown_cb: Closure<dyn FnMut() -> JsValue>,

    dispatcher: Dispatcher<EventBus>,
    workspace: Option<Workspace>,

    is_editing: bool,
}

impl Keymanager {
    fn local_key(&self, ctx: &Context<Self>) -> Html {
        let state = self.state.borrow();

        let icon = if self.is_editing {
            html! {
                <i class="bi bi-check-lg"></i>
            }
        } else {
            html! {
                <i class="bi bi-pencil-square"></i>
            }
        };

        let value = if self.is_editing {
            self.local_key_input.clone()
        } else {
            state
                .local_key
                .as_ref()
                .map(|k| k.1.clone())
                .unwrap_or("".to_string())
        };

        let oninput_name = ctx
            .link()
            .callback(move |e: InputEvent| KeymanagerMsg::LocalKeyInputChanged(e));
        let onclick_add = ctx.link().callback(|_| KeymanagerMsg::SetLocalKey);
        html! {
            <div class="row input-group has-validation">
                <input type={"text"} oninput={oninput_name} {value} disabled={!self.is_editing} class="form-control col-10" />
                <button type={"button"} class="btn btn-primary col-2" onclick={onclick_add} disabled={self.is_editing && self.local_key_input.is_empty()}>
                { icon }
                </button>
            </div>
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    local_key: Option<(PrivateKey, String)>,
    keys: Vec<(String, PrivateKey)>,
    remote_keys_serial: usize,
}

impl State {
    pub fn add_alias(&mut self, alias: String) {
        use bdk::bitcoin::hashes::Hash;

        let hash = bdk::bitcoin::hashes::sha256::Hash::hash(alias.as_bytes());
        let sk = PrivateKey {
            compressed: true,
            network: Network::Testnet,
            inner: SecretKey::from_slice(&hash).expect("32 bytes, within curve order"),
        };

        self.keys.push((alias, sk));
    }

    pub fn set_local(&mut self, alias: String) {
        use bdk::bitcoin::hashes::Hash;

        let hash = bdk::bitcoin::hashes::sha256::Hash::hash(alias.as_bytes());
        let sk = PrivateKey {
            compressed: true,
            network: Network::Testnet,
            inner: SecretKey::from_slice(&hash).expect("32 bytes, within curve order"),
        };

        self.local_key = Some((sk, alias));
    }
}

impl State {
    fn new() -> Self {
        let mut state = State {
            local_key: None,
            keys: Vec::new(),
            remote_keys_serial: 0,
        };
        state.add_alias("Alice".to_string());

        state
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
                .keys
                .iter()
                .map(|(k, v)| {
                    let key = v.public_key(&bdk::bitcoin::secp256k1::Secp256k1::new());
                    (k, key)
                })
                .collect::<Vec<_>>();
            log::debug!("{:?}", dropdown);
            serde_wasm_bindgen::to_value(&dropdown).unwrap()
        });

        Keymanager {
            new_input_name: String::new(),
            local_key_input: String::new(),

            state,

            compiled_cb,
            dropdown_cb,

            dispatcher: EventBus::dispatcher(),
            workspace: None,

            is_editing: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput_name = ctx
            .link()
            .callback(move |e: InputEvent| KeymanagerMsg::NewInputNameChanged(e));
        let onclick_add = ctx.link().callback(|_| KeymanagerMsg::AddKey);

        html! {
            <div class="container" style="height: 800px;">
                <div class="row" style="height: 100%;">
                    <div id="blocklyArea" class="col-6 px-5" style="height: 100%;">
                        <div id="blocklyDiv" style="position: absolute;"></div>
                    </div>
                    <div class="col-6 px-5">
                        <div>
                            <h2>{ "Local Key" }</h2>
                            { self.local_key(ctx) }
                        </div>

                        <div style="margin-top: 20px">
                        <h2>{ "Remote Keys" }</h2>
                        { for self.state.borrow().keys.iter().enumerate().map(|(i, (name, _key))| {
                                let name_cloned = name.clone();
                                let remove_onclick = ctx.link().callback_once(move |_| KeymanagerMsg::RemoveKey(i));
                                // let key = key.public_key(&bdk::bitcoin::secp256k1::Secp256k1::new());
                                html! {
                                    <div class="input-group row mb-1">
                                        <input type={"text"} disabled=true value={name.clone()} class="form-control col-10" />
                                        // <span class="col-7">{ key.clone() }</span>
                                        <button type="button" onclick={remove_onclick} disabled={self.state.borrow().keys.len() == 1} class="col-2 btn btn-primary"><i class="bi bi-trash"></i></button>
                                    </div>
                                }
                            })
                        }
                        </div>
                        <div class="row input-group has-validation">
                            <input type={"text"} oninput={oninput_name} placeholder={"Name"} value={self.new_input_name.clone()} class="col-10 form-control" />
                            <button type={"button"} class="btn btn-primary col-2" onclick={onclick_add} disabled={self.new_input_name.is_empty()}>
                              <i class="bi bi-check-lg"></i>
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

                self.workspace = Some(workspace);

                false
            }
            KeymanagerMsg::NewInputNameChanged(e) => {
                self.new_input_name = e.target_unchecked_into::<HtmlInputElement>().value();
                true
            }
            KeymanagerMsg::AddKey => {
                self.state
                    .borrow_mut()
                    .add_alias(self.new_input_name.clone());
                self.new_input_name = String::new();

                storage::save(&self.state.borrow());

                true
            }
            KeymanagerMsg::LocalKeyInputChanged(e) => {
                self.local_key_input = e.target_unchecked_into::<HtmlInputElement>().value();
                true
            }
            KeymanagerMsg::SetLocalKey => {
                if self.is_editing {
                    self.state
                        .borrow_mut()
                        .set_local(self.local_key_input.clone());
                    self.local_key_input = String::new();

                    storage::save(&self.state.borrow());

                    self.is_editing = false;
                } else {
                    self.local_key_input = self
                        .state
                        .borrow()
                        .local_key
                        .as_ref()
                        .map(|k| k.1.clone())
                        .unwrap_or("".to_string());
                    self.is_editing = true;
                }

                true
            }
            KeymanagerMsg::RemoveKey(i) => {
                self.state.borrow_mut().keys.remove(i);
                storage::save(&self.state.borrow());

                true
            }

            KeymanagerMsg::Compiled(mut desc) => {
                if let Some((local_key, _)) = self.state.borrow().local_key {
                    desc = desc.replace("_MY_KEY", &local_key.to_string());
                }
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

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(workspace) = &self.workspace {
            save_blockly(workspace);
        }
    }
}
