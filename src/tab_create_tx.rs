use yew::prelude::*;

use crate::AppWallet;

#[derive(PartialEq, Properties)]
pub struct CreateTxProperties {
    pub wallet: AppWallet,
}

pub struct TabCreateTx {
    wallet: AppWallet,
}

impl Component for TabCreateTx {
    type Message = ();

    type Properties = CreateTxProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            wallet: props.wallet.clone(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <button>{"This is a test!"}</button>
            </div>
        }
    }
}
