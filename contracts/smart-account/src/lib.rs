#![no_std]
// placeholder — filled in Prompt 4

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SmartAccount;

#[contractimpl]
impl SmartAccount {
    pub fn __constructor(_env: Env) {}
}
