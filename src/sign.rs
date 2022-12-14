use crate::AppWallet;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::wallet::signer::SignOptions;
use std::str::FromStr;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub enum Msg {
    PsbtChanged(InputEvent),
    Sign,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub wallet: AppWallet,
}

pub struct Sign {
    psbt: Option<Result<PartiallySignedTransaction, ()>>,
    signed_psbt: Option<PartiallySignedTransaction>,
    wallet: AppWallet,
}

impl Component for Sign {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            wallet: props.wallet.clone(),
            psbt: None,
            signed_psbt: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| Msg::PsbtChanged(e));
        let onclick = ctx.link().callback(|_| Msg::Sign);
        let (is_invalid, button_disabled) = match self.psbt {
            Some(Ok(_)) => ("", false),
            Some(Err(_)) => ("is-invalid", true),
            None => ("", true),
        };
        html! {
            <div class="daniela">
                <label for="psbtTextArea" class="form-label">{"Paste here your PSBT:"}</label>
                <textarea id="psbtTextArea" class={classes!("form-control", "daniela-textarea", is_invalid).to_string()} rows="5" {oninput}></textarea>
                <div class="invalid-feedback">
                { "Please enter a valid PSBT" }
                </div>
                <button class="btn btn-primary" {onclick} disabled={ button_disabled }>{ "Sign" }</button>
                <textarea class="form-control daniela-textarea" id="signed-psbt-textarea" rows="10" readonly=true value={self.signed_psbt.as_ref().map(|s| s.to_string())}></textarea>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PsbtChanged(e) => {
                let psbt = e.target_unchecked_into::<HtmlInputElement>().value();
                log::info!("Psbt changed! {:?}", psbt);
                self.signed_psbt = None;
                if psbt == "" {
                    self.psbt = None;
                } else {
                    self.psbt = Some(PartiallySignedTransaction::from_str(&psbt).map_err(|_| ()));
                }
                log::info!("Psbt parsing: {:?}", self.psbt);
                true
            }

            Msg::Sign => {
                log::info!("Sign");
                self.wallet
                    .borrow()
                    .0
                    .sign(
                        &mut self.psbt.as_mut().unwrap().as_mut().unwrap(),
                        SignOptions::default(),
                    )
                    .unwrap();
                self.signed_psbt = Some(self.psbt.as_ref().unwrap().as_ref().unwrap().clone());
                true
            }
        }
    }
}
