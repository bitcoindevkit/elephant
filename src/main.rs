mod app;
mod app_wallet;
mod keymanager;
mod merge;
mod home;
mod sign;

use app::App;
use app_wallet::AppWallet;
use merge::Merge;
use home::Home;
use sign::Sign;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
