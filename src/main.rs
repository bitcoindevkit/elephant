mod app;
mod app_wallet;
mod keymanager;
mod sign;

use app::App;
use app_wallet::AppWallet;
use sign::Sign;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
