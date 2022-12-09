use crate::AppWallet;
use bdk::bitcoin;
use bdk::miniscript::policy::Concrete;
use std::str::FromStr;
use yew::functional::*;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use crate::evt::EventBus;

pub struct App {
    wallet: Option<AppWallet>,
    _recv: Box<dyn Bridge<EventBus>>,
    current_tab: Tabs,
}

pub enum Msg {
    Reload,
    TabChange(Tabs),
    Descriptor(String),
}

#[derive(Copy, Clone)]
enum Tabs {
    Home,
    KeyManagement,
    CreateTx,
    SignTx,
    Merge,
}

fn parse_policy(policy: &str) -> Result<AppWallet, Box<dyn std::error::Error>> {
    let policy = Concrete::<String>::from_str(policy)?;
    let policy: bdk::miniscript::Miniscript<String, bdk::miniscript::Tap> = policy.compile()?;

    Ok(AppWallet::new(
        &format!(
            "tr(89de7c56ecdf6c400295a57a203d87a53ed28f74735d2373a3e034781338f259,{})",
            policy
        ),
        None,
        bitcoin::Network::Regtest,
    )?)
}

impl App {
    fn create_tab(&self) -> Html {
        match self.current_tab {
            Tabs::Home => {
                html! { < crate::home::Home wallet={self.wallet.as_ref().unwrap().clone()} /> }
            }
            Tabs::KeyManagement => html! {< crate::keymanager::Keymanager />},
            Tabs::CreateTx => {
                html! { < crate::tab_create_tx::TabCreateTx wallet={self.wallet.as_ref().unwrap().clone()} /> }
            }
            Tabs::SignTx => {
                html! { < crate::sign::Sign wallet={self.wallet.as_ref().unwrap().clone()} /> }
            }
            Tabs::Merge => {
                html! { < crate::merge::Merge wallet={self.wallet.as_ref().unwrap().clone()} /> }
            }
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        App {
            wallet: None,
            _recv: EventBus::bridge(ctx.link().callback(Msg::Descriptor)),
            current_tab: Tabs::KeyManagement,
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
            Msg::TabChange(t) => {
                self.current_tab = t;
                true
            }
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let disabled_link = match self.wallet {
            Some(_) => None,
            None => Some("disabled"),
        };
        let link = ctx.link().clone();
        let onclick = move |t: Tabs| ctx.link().callback(move |_| Msg::TabChange(t));

        html! {
            <div>
                <div class="container">
                    <header class="d-flex flex-wrap justify-content-center py-3 mb-4 border-bottom">
                        <a href="/" class="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-dark text-decoration-none">
                            <span class="fs-4">{ "Elephant workshop" }</span>
                        </a>
                        <ul class="nav nav-pills">
                        // TODO: active tab?
                            <li class="nav-item"><a onclick={onclick(Tabs::KeyManagement)} class="nav-link" aria-current="page">{ "Key Manager" }</a></li>
                            <li class="nav-item"><a onclick={onclick(Tabs::Home)} class={classes!("nav-link", disabled_link)}>{ "Wallet home" }</a></li>
                            <li class="nav-item"><a onclick={onclick(Tabs::CreateTx)} class={classes!("nav-link", disabled_link)}>{ "Create transaction" }</a></li>
                            <li class="nav-item"><a onclick={onclick(Tabs::SignTx)} class={classes!("nav-link", disabled_link)}>{ "Sign transaction" }</a></li>
                            <li class="nav-item"><a onclick={onclick(Tabs::Merge)} class="nav-link">{ "Merge and broadcast" }</a></li>
                        </ul>
                    </header>
                </div>
                { self.create_tab() }
            </div>
        }
    }
}
