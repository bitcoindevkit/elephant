use crate::AppWallet;
use bdk::bitcoin;
use bdk::miniscript::policy::Concrete;
use std::str::FromStr;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use crate::evt::EventBus;

pub struct App {
    wallet: Option<AppWallet>,
    _recv: Box<dyn Bridge<EventBus>>,
}

pub enum Msg {
    Descriptor(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/keys")]
    KeyManagement,
    #[at("/create_tx")]
    CreateTx,
    #[at("/sign_tx")]
    SignTx,
    #[at("/merge")]
    Merge,
}

fn parse_policy(policy: &str) -> Result<AppWallet, Box<dyn std::error::Error>> {
    let policy = Concrete::<String>::from_str(policy)?;
    let segwit_policy: bdk::miniscript::Miniscript<String, bdk::miniscript::Segwitv0> =
        policy.compile()?;

    Ok(AppWallet::new(
        &format!("wsh({})", segwit_policy),
        None,
        bitcoin::Network::Testnet,
    )?)
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        App {
            wallet: None,
            _recv: EventBus::bridge(ctx.link().callback(Msg::Descriptor)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Descriptor(s) => {
                match parse_policy(&s) {
                    Ok(w) => {
                        self.wallet = Some(w);
                    }
                    Err(e) => {
                        log::warn!("{:?}", e);
                        self.wallet = None;
                    }
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let disabled_link = match self.wallet {
            Some(_) => None,
            None => Some("disabled"),
        };
        let link = ctx.link().clone();

        html! {
            <div>
                <div class="container">
                    <header class="d-flex flex-wrap justify-content-center py-3 mb-4 border-bottom">
                        <a href="/" class="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-dark text-decoration-none">
                            <span class="fs-4">{ "Elephant workshop" }</span>
                        </a>
                        <ul class="nav nav-pills">
                        // TODO: active tab?
                            <li class="nav-item"><a href="/keys" class="nav-link" aria-current="page">{ "Key Manager" }</a></li>
                            <li class="nav-item"><a href="/" class={classes!("nav-link", disabled_link)}>{ "Wallet home" }</a></li>
                            <li class="nav-item"><a href="/create_tx" class={classes!("nav-link", disabled_link)}>{ "Create transaction" }</a></li>
                            <li class="nav-item"><a href="/sign_tx" class={classes!("nav-link", disabled_link)}>{ "Sign transaction" }</a></li>
                            <li class="nav-item"><a href="/merge" class="nav-link">{ "Merge and broadcast" }</a></li>
                        </ul>
                    </header>
                </div>
                <BrowserRouter>
                    <Switch<Route> render={Switch::render(move |routes| {
                        match routes {
                            Route::Home => html! { < crate::home::Home /> },
                            Route::KeyManagement => html! {
                                < crate::keymanager::Keymanager />
                            },
                            Route::CreateTx => html! { { "TODO" } },
                            Route::SignTx => html! { < crate::sign::Sign /> },
                            Route::Merge => html! { < crate::merge::Merge /> },
                        }
                    })} />
                </BrowserRouter>
            </div>
        }
    }
}
