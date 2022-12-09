use crate::AppWallet;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::bitcoin::*;
use bdk::wallet::AddressIndex;
use bdk::*;
use yew::prelude::*;

pub enum Msg {
    ReloadTriggered,
    ReloadFinished(Result<(), bdk::Error>),
}

#[derive(Default, Properties, PartialEq)]
pub struct Props {}

pub struct Home {
    wallet: AppWallet,
    balance: bdk::Balance,
    address: String,
    transactions: Vec<(String, i64)>,
    is_loading: bool,
}

impl Home {}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        let wallet = AppWallet::new(
            "tr(cVd1Ew5616o4FQaXDpq2LcdqGUgMVpbVa2MkqqmWibsQ8g4pH4qc)",
            None,
            bitcoin::Network::Testnet,
        )
        .unwrap();
        let address = wallet
            .borrow()
            .0
            .get_address(AddressIndex::New)
            .unwrap()
            .to_string();
        Self {
            wallet,
            balance: Balance::default(),
            address,
            transactions: vec![],
            is_loading: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
        let onclick = ctx.link().callback(|_| Msg::ReloadTriggered);
        let disabled = self.is_loading;
        html! {
            <div>
                <div class="daniela-home text-center">
                    <div class="balance-wrapper">
                        <div class="balance"> { format!("{} sats", satcommify(self.balance.get_spendable())) } </div>
                        <div class="balance-unconfirmed"> { format!("+ {} sats unconfirmed", satcommify(self.balance.untrusted_pending)) } </div>
                    </div>
                    <div class="address"> { format!("Send money to: {}", self.address) } </div>
                    <button type="button" class="btn btn-primary daniela-button" {onclick} {disabled}>{if disabled { "Loading..." } else { "Reload" }}</button>
                </div>
                <div class="table-responsive">
                    <table class="table-sm daniela-table">
                        <thead>
                        </thead>
                        <tbody>
                            {
                                for self.transactions.iter().map(|tx| { html! {
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReloadTriggered => {
                self.is_loading = true;
                let wallet_cloned = self.wallet.0.clone();
                ctx.link().send_future(async move {
                    let res = wallet_cloned
                        .borrow()
                        .0
                        .sync(
                            &wallet_cloned.borrow().1,
                            bdk::wallet::SyncOptions::default(),
                        )
                        .await;
                    Msg::ReloadFinished(res)
                });
                true
            }
            Msg::ReloadFinished(res) => {
                self.balance = self.wallet.borrow().0.get_balance().unwrap();
                let mut temp_tx = self.wallet.borrow().0.list_transactions(false).unwrap();
                temp_tx.sort_by(|a, b| {
                    b.confirmation_time
                        .as_ref()
                        .map(|t| t.height)
                        .cmp(&a.confirmation_time.as_ref().map(|t| t.height))
                });
                self.transactions = temp_tx
                    .into_iter()
                    .map(|tx| (tx.txid.to_string(), tx.received as i64 - tx.sent as i64))
                    .collect();
                self.is_loading = false;
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let wallet_cloned = self.wallet.0.clone();
            ctx.link().send_future(async move {
                let res = wallet_cloned
                    .borrow()
                    .0
                    .sync(
                        &wallet_cloned.borrow().1,
                        bdk::wallet::SyncOptions::default(),
                    )
                    .await;
                Msg::ReloadFinished(res)
            });
        }
    }
}
