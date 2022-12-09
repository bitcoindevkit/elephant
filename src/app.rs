use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <h1>{ "Hello World!" }</h1>
            <button type="button" class="btn btn-primary">{"Bitcoin"}<i class="bi bi-currency-bitcoin"></i></button>
        </main>
    }
}
