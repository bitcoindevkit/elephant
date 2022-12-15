use crate::AppWallet;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::*;
use std::str::FromStr;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub enum Msg {
    PsbtChanged(usize, InputEvent),
    DeletePsbt(usize),
    NewPsbtField,
    Merge,
    BroadcastTriggered,
    BroadcastFinished(Result<(), String>),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub wallet: AppWallet,
}

pub struct Merge {
    psbts: Vec<(Option<Result<PartiallySignedTransaction, String>>, usize)>,
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
            psbts: vec![(None, 0)],
            merged_psbt: None,
            wallet: props.wallet.clone(),
            key_n: 0,
            is_broadcasting: false,
            broadcast_result: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let merge = ctx.link().callback(|_| Msg::Merge);
        let add_psbt = ctx.link().callback(|_| Msg::NewPsbtField);
        let broadcast = ctx.link().callback(|_| Msg::BroadcastTriggered);
        let mut add_psbt_disabled = false;
        let mut merge_disabled = false;
        let (merged_psbt, merge_error) = match &self.merged_psbt {
            Some(Ok(p)) => (p.clone().to_string(), "".to_string()),
            Some(Err(e)) => ("".to_string(), e.to_string()),
            _ => ("".to_string(), "".to_string()),
        };
        let broadcast_disabled =
            self.merged_psbt.is_none() || merge_error != "" || self.is_broadcasting;
        let is_invalid = if merge_error != "" { "is-invalid" } else { "" };

        let broadcast_result_msg = match &self.broadcast_result {
            Some(Ok(())) => format!("Broadcast is successful! Yayyyy!"),
            Some(Err(err)) => format!("You failed! Error: {}", err.to_string()),
            None => format!(""),
        };

        html! {
            <div class="daniela">
                <label for="accordionExample" class="form-label">{"Paste here the PSBTs to merge:"}</label>
                <div class="accordion" id="accordionExample">
                    {
                        for self.psbts.iter().enumerate().map(|(i, (psbt, key))| {
                            let oninput = ctx.link().callback(move |e: InputEvent| Msg::PsbtChanged(i, e));
                            let delete_psbt = ctx.link().callback(move |_| Msg::DeletePsbt(i));
                            let (is_invalid, button_disabled, annotation_title) = match psbt {
                                Some(Ok(_)) => ("", false, "Transaction"),
                                Some(Err(_)) => ("is-invalid", true, "Invalid PSBT!"),
                                None => ("", true, "(Empty)"),
                            };
                            add_psbt_disabled |= button_disabled;
                            merge_disabled |= button_disabled;
                            let heading_name = format!("heading{}", i);
                            let collapse_name = format!("collapse{}", i);
                            let collapse_name_id = format!("#collapse{}", i);
                            let psbt_error = match psbt {
                                Some(Err(e)) => e.to_string(),
                                _ => "".to_string(),
                            };

                            html! {
                                <div class="accordion-item" key={*key}>
                                <h2 class="accordion-header" id={heading_name.clone()}>
                                  <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target={collapse_name_id.clone()} aria-expanded="true" aria-controls={collapse_name.clone()}>
                                  { annotation_title }
                                  </button>
                                </h2>
                                <div id={collapse_name.clone()} class="accordion-collapse collapse show" aria-labelledby={heading_name.clone()} data-bs-parent="#accordionExample">
                                  <div class="accordion-body">
                                    <div>
                                        <textarea class={classes!("form-control", "daniela-textarea", is_invalid).to_string()} rows="5" {oninput}></textarea>
                                        <div class="invalid-feedback">
                                        { format!("Please enter a valid PSBT: {}", psbt_error) }
                                        </div>
                                    </div>
                                    <button class="btn btn-danger" onclick={delete_psbt} disabled={ self.psbts.len() == 1 }>{ "Delete" }</button>
                                  </div>
                                </div>
                                </div>
                            }
                        })
                    }
                </div>
                <div>
                    <button class="btn btn-primary daniela-button" onclick={add_psbt} disabled={ add_psbt_disabled }>{ "Add PSBT" }</button>
                </div>
                <textarea class={classes!("form-control","daniela-textarea",is_invalid)} id="merged-psbt-textarea" rows="10" readonly=true value={merged_psbt}></textarea>
                <div class="invalid-feedback">
                { format!("Error merging PSBTs: {}", merge_error) }
                </div>
                <button class="btn btn-primary daniela-button" onclick={merge} disabled={ merge_disabled }>{ "Finish" }</button>
                <button class="btn btn-primary daniela-button" onclick={broadcast} disabled={ broadcast_disabled }>{ if self.is_broadcasting { "Broadcasting..." } else { "Broadcast" } }</button>
                <div>
                    <label> { broadcast_result_msg } </label>
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PsbtChanged(i, e) => {
                let psbt = e.target_unchecked_into::<HtmlInputElement>().value();
                log::info!("Psbt changed! {:?}", psbt);
                self.merged_psbt = None;
                if psbt == "" {
                    self.psbts[i].0 = None;
                } else {
                    self.psbts[i].0 = Some(
                        PartiallySignedTransaction::from_str(&psbt).map_err(|e| e.to_string()),
                    );
                }
                log::info!("Psbt parsing: {:?}", self.psbts);
                true
            }
            Msg::NewPsbtField => {
                log::info!("Add new PSBT field");
                self.key_n += 1;
                self.psbts.push((None, self.key_n));
                true
            }
            Msg::DeletePsbt(i) => {
                log::info!("Delete");
                self.psbts.remove(i);
                true
            }
            Msg::Merge => {
                log::info!("Merge");
                let mut temp = self.psbts[0].clone().0.unwrap().unwrap();
                for psbt in &self.psbts[1..] {
                    if let Err(e) = temp.combine(psbt.clone().0.unwrap().unwrap()) {
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
                let tx = self
                    .wallet
                    .borrow()
                    .0
                    .finalize_psbt(&mut merged_psbt, SignOptions::default());
                let tx = merged_psbt.extract_tx();
                let wallet_cloned = self.wallet.0.clone();
                ctx.link().send_future(async move {
                    let res = wallet_cloned.borrow().1.broadcast(&tx).await;
                    Msg::BroadcastFinished(res.map_err(|e| e.to_string()))
                });
                true
            }
            Msg::BroadcastFinished(res) => {
                self.is_broadcasting = false;
                self.broadcast_result = Some(res);
                true
            }
        }
    }
}
