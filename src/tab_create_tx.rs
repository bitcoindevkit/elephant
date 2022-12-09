use std::str::FromStr;

use bdk::{
    bitcoin::{psbt::Psbt, Address},
    FeeRate, KeychainKind, TransactionDetails,
};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::policy_view::{PolicyView, Selection};
use crate::AppWallet;

pub enum CreateTxMsg {
    CreateButtonClicked,
    AddressInputEvent(InputEvent),
    AmountInputEvent(InputEvent),
}

#[derive(PartialEq, Properties)]
pub struct CreateTxProps {
    pub wallet: AppWallet,
}

pub struct TabCreateTx {
    addr: String,
    amount: u64,
    policy_selection: Selection,
    psbt_result: Option<Result<(Psbt, TransactionDetails), bdk::Error>>,
}

impl Component for TabCreateTx {
    type Message = CreateTxMsg;
    type Properties = CreateTxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            addr: "".into(),
            amount: 0,
            policy_selection: Selection::default(),
            psbt_result: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CreateTxMsg::CreateButtonClicked => {
                let props = ctx.props();
                let (wallet, _) = &mut *props.wallet.borrow_mut();

                let spk = match Address::from_str(&self.addr) {
                    Ok(addr) => addr.script_pubkey(),
                    Err(err) => {
                        self.psbt_result = Some(Err(bdk::Error::Generic(format!(
                            "invalid recipient address: {}",
                            err.to_string()
                        ))));
                        return true;
                    }
                };

                let mut builder = wallet.build_tx();

                builder
                    .add_recipient(spk, self.amount)
                    .fee_rate(FeeRate::from_sat_per_vb(1.0))
                    .policy_path(self.policy_selection.extract(), KeychainKind::External)
                    .enable_rbf();

                self.psbt_result = Some(builder.finish());
                true
            }
            CreateTxMsg::AddressInputEvent(e) => {
                self.addr = e.target_unchecked_into::<HtmlInputElement>().value();
                false
            }
            CreateTxMsg::AmountInputEvent(e) => {
                self.amount = e
                    .target_unchecked_into::<HtmlInputElement>()
                    .value()
                    .parse::<u64>()
                    .unwrap_or_default();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let (wallet, _) = &*props.wallet.borrow();

        let policy = wallet.policies(KeychainKind::External).unwrap().unwrap();
        let policy_selection = self.policy_selection.clone();

        let onclick_create_button = ctx.link().callback(|_| CreateTxMsg::CreateButtonClicked);
        let oninput_address = ctx.link().callback(|e| CreateTxMsg::AddressInputEvent(e));
        let oninput_amount = ctx.link().callback(|e| CreateTxMsg::AmountInputEvent(e));

        let result_html = match &self.psbt_result {
            Some(Ok((psbt, details))) => html! {
                <div>
                    <label>{ format!("PSBT Created with txid: {}", details.txid) }</label>
                    <br/>
                    <textarea class="form-control daniela-textarea" rows="10" disabled=true value={ psbt.to_string() }></textarea>
                </div>
            },
            Some(Err(err)) => html! {
                <label>{ format!("Failed to create PSBT: {}", err.to_string()) }</label>
            },
            None => html! {
                <label>{ "Nothing created yet!" }</label>
            },
        };

        html! {
            <div class = "daniela" >
                <label>{ "Destination Address: " }</label>
                <input type="text" class="form-control" oninput={oninput_address}/>
                <br/>
                <label>{ "Amount (sats): " }</label>
                <input type="number" class="form-control" oninput={oninput_amount}/>
                <br/>
                <PolicyView selection={policy_selection} node={policy}/>
                <br/>
                <button class="btn btn-primary" onclick={onclick_create_button}> { "Create" } </button>
                <br/>
                { result_html }
            </div>
        }
    }
}
