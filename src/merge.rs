use crate::AppWallet;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::bitcoin::*;
use bdk::*;
use std::str::FromStr;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub enum Msg {
    PsbtChanged(usize, InputEvent),
    DeletePsbt(usize),
    NewPsbtField,
    Merge,
    Broadcast,
}

#[derive(Default, Properties, PartialEq)]
pub struct Props {}

pub struct Merge {
    psbts: Vec<(Option<Result<PartiallySignedTransaction, String>>, usize)>,
    merged_psbt: Option<Result<PartiallySignedTransaction, String>>,
    finalized_tx: Option<Transaction>,
    wallet: AppWallet,
    key_n: usize,
}

impl Component for Merge {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        let wallet = AppWallet::new(
            "tr(cVd1Ew5616o4FQaXDpq2LcdqGUgMVpbVa2MkqqmWibsQ8g4pH4qc)",
            None,
            bitcoin::Network::Testnet,
        )
        .unwrap();
        Self {
            psbts: vec![(None, 0)],
            merged_psbt: None,
            finalized_tx: None,
            wallet,
            key_n: 0,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let merge = ctx.link().callback(|_| Msg::Merge);
        let add_psbt = ctx.link().callback(|_| Msg::NewPsbtField);
        let broadcast = ctx.link().callback(|_| Msg::Broadcast);
        let mut add_psbt_disabled = false;
        let mut merge_disabled = false;
        let broadcast_disabled = true;
        let (merged_psbt, merge_error) = match &self.merged_psbt {
            Some(Ok(p)) => (p.clone().to_string(), "".to_string()),
            Some(Err(e)) => ("".to_string(), e.to_string()),
            _ => ("".to_string(), "".to_string()),
        };
        let is_invalid = if merge_error != "" { "is-invalid" } else { "" };
        html! {
            <div class="daniela">
                <h1>{ "Merge PSBTs" }</h1>
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
                <button class="btn btn-primary" onclick={add_psbt} disabled={ add_psbt_disabled }>{ "Add PSBT" }</button>
                <button class="btn btn-primary" onclick={merge} disabled={ merge_disabled }>{ "Merge" }</button>
                <button class="btn btn-primary" onclick={broadcast} disabled={ broadcast_disabled }>{ "Broadcast" }</button>
                <textarea class={classes!("form-control","daniela-textarea",is_invalid)} rows="10" disabled=true value={merged_psbt}></textarea>
                <div class="invalid-feedback">
                { format!("Error merging PSBTs: {}", merge_error) }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
            Msg::Broadcast => {
                log::info!("Broadcast");
                // TODO!!
                true
            }
        }
    }
}
