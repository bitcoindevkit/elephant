use crate::AppWallet;
use yew::prelude::*;
use yew_router::prelude::*;
use yew::functional::*;

pub struct App {
    wallet: Option<AppWallet>,
}

pub enum Msg {
    Reload,
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

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App { wallet: None }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                            <li class="nav-item"><a href="/" class="nav-link">{ "Wallet home" }</a></li>
                            <li class="nav-item"><a href="/create_tx" class="nav-link">{ "Create transaction" }</a></li>
                            <li class="nav-item"><a href="/sign_tx" class="nav-link">{ "Sign transaction" }</a></li>
                            <li class="nav-item"><a href="/merge" class="nav-link">{ "Merge and broadcast" }</a></li>
                        </ul>
                    </header>
                </div>
                <BrowserRouter>
                    <Switch<Route> render={Switch::render(switch)} />
                </BrowserRouter>
            </div>
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { < crate::home::Home /> },
        Route::KeyManagement => html! {
            < crate::keymanager::Keymanager />
        },
        Route::CreateTx => html! { { "TODO" } },
        Route::SignTx => html! { < crate::sign::Sign /> },
        Route::Merge => html! { < crate::merge::Merge /> },
    }
}
