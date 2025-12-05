use candid::Principal;

mod api;
mod api_admin;
mod api_init;
mod cca;
mod ecdsa;
mod evm;
mod helper;
mod outcall;
mod schnorr;
mod store;
mod svm;
mod types;

use api_init::CanisterArgs;

ic_cdk::export_candid!();
