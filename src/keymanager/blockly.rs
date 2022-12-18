use serde::Serialize;
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Blockly)]
    fn inject(id: &str, opts: &JsValue) -> JsValue;
    #[wasm_bindgen(js_namespace = Blockly)]
    fn defineBlocksWithJsonArray(val: &JsValue);

    #[wasm_bindgen(js_namespace = BlocklyStorage)]
    fn backupOnUnload();
    #[wasm_bindgen(js_namespace = BlocklyStorage)]
    fn restoreBlocks();

    #[wasm_bindgen(js_namespace = BlocklyExt)]
    fn initJs(workspace: &JsValue, compiled_cb: &JsValue, dropdown_cb: &JsValue);
    #[wasm_bindgen(js_namespace = BlocklyExt)]
    fn insertBegin(workspace: &JsValue);
    #[wasm_bindgen(js_namespace = BlocklyExt)]
    fn saveBlockly(workspace: &JsValue);
}

pub struct Workspace(JsValue);

pub fn define_blocks<'b>(blocks: impl Iterator<Item = &'b BlocklyBlock>) {
    let blocks = blocks.into_iter().collect::<Vec<_>>();
    // log::debug!("{}", serde_json::to_string(&blocks).unwrap());

    let jsval = serde_wasm_bindgen::to_value(&blocks).unwrap();
    defineBlocksWithJsonArray(&jsval);
}
pub fn inject_blockly(id: &str, opts: &BlocklyOptions) -> Workspace {
    let jsval = serde_wasm_bindgen::to_value(opts).unwrap();
    Workspace(inject(id, &jsval))
}

pub fn enable_backup() {
    backupOnUnload();
}
pub fn restore_blocks() {
    restoreBlocks();
}

pub fn init_js(
    workspace: &Workspace,
    compiled_cb: &Closure<dyn FnMut(String)>,
    dropdown_cb: &Closure<dyn FnMut() -> JsValue>,
) {
    initJs(&workspace.0, compiled_cb.as_ref(), dropdown_cb.as_ref());
}
pub fn insert_begin(workspace: &Workspace) {
    insertBegin(&workspace.0);
}
pub fn save_blockly(workspace: &Workspace) {
    saveBlockly(&workspace.0)
}

#[derive(Debug, Serialize)]
pub enum ValueType {
    Policy,
    Key,
}

#[derive(Debug, Serialize)]
pub struct BlocklyToolboxCategory {
    pub name: &'static str,
    pub kind: &'static str,
    pub contents: Vec<BlocklyToolboxBlock>,
    pub colour: u16,
}

impl BlocklyToolboxCategory {
    pub fn new(
        name: &'static str,
        colour: u16,
        blocks: impl Iterator<Item = &'static str>,
    ) -> Self {
        BlocklyToolboxCategory {
            kind: "category",
            name,
            colour,
            contents: blocks.into_iter().map(BlocklyToolboxBlock::new).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BlocklyToolboxBlock {
    pub kind: &'static str,
    #[serde(rename = "type")]
    pub ty: &'static str,
}

impl BlocklyToolboxBlock {
    pub fn new(ty: &'static str) -> Self {
        BlocklyToolboxBlock { kind: "block", ty }
    }
}

#[derive(Debug, Serialize)]
pub struct BlocklyToolbox {
    pub kind: &'static str,
    pub contents: Vec<BlocklyToolboxCategory>,
}

#[derive(Debug, Serialize)]
pub struct BlocklyOptions {
    pub toolbox: BlocklyToolbox,
    pub trashcan: bool,
}

#[derive(Debug, Serialize)]
pub struct BlocklyBlockArg {
    #[serde(rename = "type")]
    pub ty: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<&'static str>,
}

impl BlocklyBlockArg {
    pub fn input_statement(name: &'static str, check: ValueType) -> Self {
        BlocklyBlockArg {
            name: Some(name),
            check: Some(check),
            ty: "input_statement",
            value: None,
        }
    }
    pub fn input_value(name: &'static str, check: ValueType) -> Self {
        BlocklyBlockArg {
            name: Some(name),
            check: Some(check),
            ty: "input_value",
            value: None,
        }
    }
    pub fn dummy() -> Self {
        BlocklyBlockArg {
            name: None,
            check: None,
            ty: "input_dummy",
            value: None,
        }
    }
    pub fn field_number(name: &'static str, value: &'static str) -> Self {
        BlocklyBlockArg {
            name: Some(name),
            check: None,
            ty: "field_number",
            value: Some(value),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlocklyBlock {
    #[serde(rename = "type")]
    pub ty: &'static str,
    pub message0: &'static str,
    pub inputs_inline: bool,
    pub args0: Vec<BlocklyBlockArg>,
    pub output: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_statement: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_statement: Option<ValueType>,
    pub tooltip: Option<&'static str>,
    pub colour: u16,
    pub extensions: Vec<&'static str>,
}
