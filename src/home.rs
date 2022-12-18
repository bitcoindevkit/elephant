use crate::AppWallet;
use bdk::wallet::AddressIndex;
use yew::prelude::*;

pub enum Msg {
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub wallet: AppWallet,
    pub transactions: Vec<(String, i64)>,
    pub balance: bdk::Balance,
}

pub struct Home {
    props: Props,
    address: String,
}

impl Home {}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props().clone();
        let address = props
            .wallet
            .borrow()
            .0
            .get_address(AddressIndex::New)
            .unwrap()
            .to_string();
        Self {
            props,
            address,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let satcommify = |mut n: u64| {
            let sats_3 = n % 1000;
            n /= 1000;
            let sats_2 = n % 1000;
            n /= 1000;
            let sats_1 = n % 100;
            n /= 100;
            let btc = n;
            format!("{}.{:02} {:03} {:03}", btc, sats_1, sats_2, sats_3)
        };
        html! {
            <div>
                <div class="daniela-home text-center">
                    <div class="balance-wrapper">
                        <div class="balance"> { format!("{} sats", satcommify(self.props.balance.get_spendable())) } </div>
                        <div class="balance-unconfirmed"> { format!("+ {} sats unconfirmed", satcommify(self.props.balance.untrusted_pending)) } </div>
                    </div>
                    <div class="address"> { format!("Receiving address: {}", self.address) } </div>
                </div>
                <div class="table-responsive">
                    <table class="table-sm daniela-table">
                        <thead>
                        </thead>
                        <tbody>
                            {
                                for self.props.transactions.iter().map(|tx| { html! {
                                    <tr>
                                        <td scope="row">{tx.0.clone()}</td>
                                        <td class="daniela-table-align-right">{format!("{} {} sats", if tx.1 >= 0 { "+" } else { "-" }, satcommify(tx.1.abs() as u64))}</td>
                                    </tr>
                                }})
                            }
                        </tbody>
                    </table>
                </div>
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if &self.props != ctx.props() {
            self.props = ctx.props().clone();
            true
        } else {
            false
        }
    }
}
