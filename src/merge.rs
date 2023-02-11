use crate::AppWallet;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::*;
use std::str::FromStr;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub enum Msg {
    PsbtInputChanged(InputEvent),
    DeletePsbt(usize),
    //NewPsbtField,
    AddPsbt,
    Merge,
    BroadcastTriggered,
    BroadcastFinished(Result<(), String>),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub wallet: AppWallet,
}

pub struct Merge {
    psbt_input: Option<Result<PartiallySignedTransaction, String>>,
    psbt_input_text: String,
    psbts: Vec<(PartiallySignedTransaction, usize)>,
    merged_psbt: Option<Result<PartiallySignedTransaction, String>>,
    wallet: AppWallet,
    is_broadcasting: bool,
    broadcast_result: Option<Result<(), String>>,
    key_n: usize,
}

impl Component for Merge {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            psbt_input: None,
            psbt_input_text: "".to_string(),
            psbts: vec![],
            merged_psbt: None,
            wallet: props.wallet.clone(),
            key_n: 0,
            is_broadcasting: false,
            broadcast_result: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let merge = ctx.link().callback(|_| Msg::Merge);
        let add_psbt = ctx.link().callback(|_| Msg::AddPsbt);
        let broadcast = ctx.link().callback(|_| Msg::BroadcastTriggered);
        let merge_disabled = self.psbts.is_empty();
        let (merged_psbt, merge_error) = match &self.merged_psbt {
            Some(Ok(p)) => (p.clone().to_string(), "".to_string()),
            Some(Err(e)) => ("".to_string(), e.to_string()),
            _ => ("".to_string(), "".to_string()),
        };
        let broadcast_disabled =
            self.merged_psbt.is_none() || merge_error != "" || self.is_broadcasting;
        let is_invalid = if merge_error != "" { "is-invalid" } else { "" };
        let (add_psbt_disabled, psbt_input_is_invalid) = match self.psbt_input {
            Some(Err(_)) => (true, "is-invalid"),
            Some(Ok(_)) => (false, ""),
            None => (true, ""),
        };

        let broadcast_result_msg = match &self.broadcast_result {
            Some(Ok(())) => format!(
                "Successfully broadcasted transaction with txid: {:?}",
                self.merged_psbt
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .unsigned_tx
                    .txid()
            ),
            Some(Err(err)) => format!("Broadcast failed with error: {}", err.to_string()),
            None => format!(""),
        };
        let oninput = ctx
            .link()
            .callback(move |e: InputEvent| Msg::PsbtInputChanged(e));

        html! {
            <div class="daniela">
                <div class="same-line">
                    <input type="text" style="margin-right: 5px" class={classes!("form-control", psbt_input_is_invalid)} id="addPsbt" {oninput} value={self.psbt_input_text.clone()} placeholder="Paste your PSBT here..."/>
                    <button class="btn btn-primary" onclick={add_psbt} disabled={ add_psbt_disabled }> <i class="bi bi-plus-lg"></i> </button>
                </div>
                {
                    for self.psbts.iter().enumerate().map(|(i, (psbt, key))| {
                        let delete_psbt = ctx.link().callback(move |_| Msg::DeletePsbt(i));
                        html! {
                            <div class="same-line">
                                <input type="text" style="margin-right: 5px" class="form-control" disabled=true key={*key} value={psbt.to_string()}/>
                                <button class="btn btn-danger" onclick={delete_psbt} ><i class="bi bi-trash"></i></button>
                            </div>
                        }
                    })
                }
                <button class="btn btn-primary daniela-button" onclick={merge} disabled={ merge_disabled }>{ "Merge" }</button>
                <textarea class={classes!("form-control","daniela-textarea",is_invalid)} id="merged-psbt-textarea" rows="10" readonly=true value={merged_psbt}></textarea>
                <div class="invalid-feedback">
                { format!("Error merging PSBTs: {}", merge_error) }
                </div>
                <button class="btn btn-primary daniela-button" onclick={broadcast} disabled={ broadcast_disabled }>{ if self.is_broadcasting { "Broadcasting..." } else { "Broadcast" } }</button>
                <div>
                    <label> { broadcast_result_msg } </label>
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DeletePsbt(i) => {
                log::info!("Delete");
                self.psbts.remove(i);
                true
            }
            Msg::Merge => {
                log::info!("Merge");
                let mut temp = self.psbts[0].clone().0;
                for psbt in &self.psbts[1..] {
                    if let Err(e) = temp.combine(psbt.clone().0) {
                        self.merged_psbt = Some(Err(e.to_string()));
                        return true;
                    }
                }
                self.merged_psbt = Some(Ok(temp));
                true
            }
            Msg::BroadcastTriggered => {
                log::info!("Broadcast");
                self.is_broadcasting = true;
                let mut merged_psbt = self.merged_psbt.clone().unwrap().unwrap();
                let finalized = self
                    .wallet
                    .borrow()
                    .0
                    .finalize_psbt(&mut merged_psbt, SignOptions::default());
                match finalized {
                    Ok(true) => {
                        let tx = merged_psbt.extract_tx();
                        let wallet_cloned = self.wallet.0.clone();
                        ctx.link().send_future(async move {
                            let res = wallet_cloned.borrow().1.broadcast(&tx).await;
                            Msg::BroadcastFinished(res.map_err(|e| e.to_string()))
                        });
                    }
                    Ok(false) => {
                        self.broadcast_result = Some(Err("Can't finalize PSBT".to_string()));
                    }
                    Err(e) => {
                        self.broadcast_result =
                            Some(Err(format!("Error when finalizing PSBT: {}", e)));
                    }
                }
                true
            }
            Msg::BroadcastFinished(res) => {
                self.is_broadcasting = false;
                self.broadcast_result = Some(res);
                true
            }
            Msg::AddPsbt => {
                let psbt = self.psbt_input.clone().unwrap().unwrap();
                self.key_n += 1;
                self.psbts.push((psbt, self.key_n));
                self.psbt_input_text = "".to_string();
                true
            }
            Msg::PsbtInputChanged(e) => {
                let psbt = e
                    .target_unchecked_into::<HtmlInputElement>()
                    .value()
                    .trim()
                    .to_string();
                self.psbt_input_text = psbt.clone();
                log::info!("Psbt changed! {:?}", &psbt);
                if &psbt == "" {
                    self.psbt_input = None;
                } else {
                    self.psbt_input = Some(
                        PartiallySignedTransaction::from_str(&psbt).map_err(|e| e.to_string()),
                    );
                }
                log::info!("Psbt parsing: {:?}", self.psbts);
                true
            }
        }
    }
}
