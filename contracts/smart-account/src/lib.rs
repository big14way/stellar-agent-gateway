#![no_std]
use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contractimpl, symbol_short,
    Address, Env, Map, String, Val, Vec,
    crypto::Hash,
};
use stellar_accounts::smart_account::{
    add_context_rule, do_check_auth, remove_context_rule,
    update_context_rule_name, update_context_rule_valid_until,
    get_context_rule, get_context_rules, get_context_rules_count,
    add_signer, remove_signer, add_policy, remove_policy,
    ContextRule, ContextRuleType, Signer, Signatures,
    SmartAccount, SmartAccountError,
};

fn require_admin(env: &Env) {
    let admin: Address = env.storage().instance()
        .get::<_, Address>(&symbol_short!("ADMIN"))
        .expect("AgentWallet: not initialized");
    admin.require_auth();
}

fn get_admin(env: &Env) -> Address {
    env.storage().instance()
        .get::<_, Address>(&symbol_short!("ADMIN"))
        .expect("AgentWallet: not initialized")
}

#[contract]
pub struct AgentWallet;

#[contractimpl]
impl AgentWallet {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&symbol_short!("ADMIN"), &admin);
    }

    pub fn get_admin(env: Env) -> Address {
        get_admin(&env)
    }

    /// Grant an agent G-address authority to call any function on the USDC contract.
    /// ContextRuleType::CallContract scopes to the entire contract.
    /// Use Signer::Delegated because the agent has a G-address (Stellar account).
    pub fn grant_agent_usdc_access(
        env: Env,
        agent_address: Address,
        usdc_contract: Address,
        valid_until_ledger: u32,
    ) -> ContextRule {
        require_admin(&env);
        let signers: Vec<Signer> = soroban_sdk::vec![
            &env,
            Signer::Delegated(agent_address),
        ];
        let empty_policies: Map<Address, Val> = Map::new(&env);
        add_context_rule(
            &env,
            &ContextRuleType::CallContract(usdc_contract),
            &String::from_str(&env, "Agent USDC Access"),
            Some(valid_until_ledger),
            &signers,
            &empty_policies,
        )
    }
}

// Implement all 11 SmartAccount trait methods
#[contractimpl]
impl SmartAccount for AgentWallet {
    fn get_context_rule(e: &Env, context_rule_id: u32) -> ContextRule {
        get_context_rule(e, context_rule_id)
    }

    fn get_context_rules(e: &Env, context_rule_type: ContextRuleType) -> Vec<ContextRule> {
        get_context_rules(e, &context_rule_type)
    }

    fn get_context_rules_count(e: &Env) -> u32 {
        get_context_rules_count(e)
    }

    fn add_context_rule(
        e: &Env,
        context_type: ContextRuleType,
        name: String,
        valid_until: Option<u32>,
        signers: Vec<Signer>,
        policies: Map<Address, Val>,
    ) -> ContextRule {
        require_admin(e);
        add_context_rule(e, &context_type, &name, valid_until, &signers, &policies)
    }

    fn update_context_rule_name(e: &Env, context_rule_id: u32, name: String) -> ContextRule {
        require_admin(e);
        update_context_rule_name(e, context_rule_id, &name)
    }

    fn update_context_rule_valid_until(
        e: &Env, context_rule_id: u32, valid_until: Option<u32>,
    ) -> ContextRule {
        require_admin(e);
        update_context_rule_valid_until(e, context_rule_id, valid_until)
    }

    fn remove_context_rule(e: &Env, context_rule_id: u32) {
        require_admin(e);
        remove_context_rule(e, context_rule_id);
    }

    fn add_signer(e: &Env, context_rule_id: u32, signer: Signer) {
        require_admin(e);
        add_signer(e, context_rule_id, &signer);
    }

    fn remove_signer(e: &Env, context_rule_id: u32, signer: Signer) {
        require_admin(e);
        remove_signer(e, context_rule_id, &signer);
    }

    fn add_policy(e: &Env, context_rule_id: u32, policy: Address, install_param: Val) {
        require_admin(e);
        add_policy(e, context_rule_id, &policy, install_param);
    }

    fn remove_policy(e: &Env, context_rule_id: u32, policy: Address) {
        require_admin(e);
        remove_policy(e, context_rule_id, &policy);
    }
}

#[contractimpl]
impl CustomAccountInterface for AgentWallet {
    type Signature = Signatures;
    type Error = SmartAccountError;

    fn __check_auth(
        e: Env,
        signature_payload: Hash<32>,
        signatures: Signatures,
        auth_contexts: Vec<Context>,
    ) -> Result<(), SmartAccountError> {
        do_check_auth(&e, &signature_payload, &signatures, &auth_contexts)
    }
}
