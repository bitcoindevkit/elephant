mod app;
mod app_wallet;
mod evt;
mod home;
mod keymanager;
mod merge;
mod policy_node;
mod policy_view;
mod sign;
mod tab_create_tx;

use app::App;
use app_wallet::AppWallet;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
