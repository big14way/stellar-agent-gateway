#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Vec, vec,
};

#[contracttype]
pub enum DataKey {
    Admin,
    ServiceCount,
    Service(u32),
    OwnerServices(Address),
}

#[contracttype]
#[derive(Clone)]
pub struct ServiceEntry {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub description: String,
    pub price_usdc: i128,    // USDC base units. 10000 = $0.001 (7 decimals)
    pub owner: Address,
    pub tags: Vec<String>,
    pub active: bool,
}

#[contract]
pub struct ServiceRegistry;

#[contractimpl]
impl ServiceRegistry {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ServiceCount, &0u32);
    }

    pub fn register_service(
        env: Env,
        owner: Address,
        name: String,
        url: String,
        description: String,
        price_usdc: i128,
        tags: Vec<String>,
    ) -> u32 {
        owner.require_auth();
        let count: u32 = env.storage().instance()
            .get(&DataKey::ServiceCount).unwrap_or(0);
        let id = count;
        let entry = ServiceEntry {
            id, name, url, description, price_usdc,
            owner: owner.clone(), tags, active: true,
        };
        env.storage().persistent().set(&DataKey::Service(id), &entry);
        env.storage().persistent().extend_ttl(&DataKey::Service(id), 100, 518400);
        let mut ids: Vec<u32> = env.storage().persistent()
            .get(&DataKey::OwnerServices(owner.clone()))
            .unwrap_or_else(|| vec![&env]);
        ids.push_back(id);
        env.storage().persistent().set(&DataKey::OwnerServices(owner), &ids);
        env.storage().instance().set(&DataKey::ServiceCount, &(id + 1));
        env.events().publish((symbol_short!("register"), id), &entry);
        id
    }

    pub fn get_service(env: Env, id: u32) -> Option<ServiceEntry> {
        env.storage().persistent().get(&DataKey::Service(id))
    }

    pub fn get_all_services(env: Env) -> Vec<ServiceEntry> {
        let count: u32 = env.storage().instance()
            .get(&DataKey::ServiceCount).unwrap_or(0);
        let mut result = vec![&env];
        for i in 0..count {
            if let Some(entry) = env.storage().persistent()
                .get::<DataKey, ServiceEntry>(&DataKey::Service(i)) {
                if entry.active { result.push_back(entry); }
            }
        }
        result
    }

    pub fn get_services_by_owner(env: Env, owner: Address) -> Vec<ServiceEntry> {
        let ids: Vec<u32> = env.storage().persistent()
            .get(&DataKey::OwnerServices(owner.clone()))
            .unwrap_or_else(|| vec![&env]);
        let mut result = vec![&env];
        for id in ids.iter() {
            if let Some(e) = env.storage().persistent()
                .get::<DataKey, ServiceEntry>(&DataKey::Service(id)) {
                result.push_back(e);
            }
        }
        result
    }

    pub fn deregister_service(env: Env, id: u32) {
        let mut entry: ServiceEntry = env.storage().persistent()
            .get(&DataKey::Service(id)).expect("Service not found");
        entry.owner.require_auth();
        entry.active = false;
        env.storage().persistent().set(&DataKey::Service(id), &entry);
        env.events().publish((symbol_short!("deregist"), id), id);
    }

    pub fn update_price(env: Env, id: u32, new_price: i128) {
        let mut entry: ServiceEntry = env.storage().persistent()
            .get(&DataKey::Service(id)).expect("Service not found");
        entry.owner.require_auth();
        entry.price_usdc = new_price;
        env.storage().persistent().set(&DataKey::Service(id), &entry);
    }

    pub fn get_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::ServiceCount).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup<'a>() -> (Env, Address, ServiceRegistryClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(ServiceRegistry, (&admin,));
        let client = ServiceRegistryClient::new(&env, &contract_id);
        (env, admin, client)
    }

    #[test]
    fn test_register_and_get() {
        let (env, _admin, client) = setup();
        let owner = Address::generate(&env);
        let id = client.register_service(
            &owner,
            &String::from_str(&env, "weather-api"),
            &String::from_str(&env, "http://localhost:3001/weather"),
            &String::from_str(&env, "Weather data"),
            &10000i128,
            &soroban_sdk::vec![&env, String::from_str(&env, "weather")],
        );
        assert_eq!(id, 0);
        let e = client.get_service(&0).unwrap();
        assert_eq!(e.price_usdc, 10000);
        assert!(e.active);
    }

    #[test]
    fn test_deregister() {
        let (env, _admin, client) = setup();
        let owner = Address::generate(&env);
        client.register_service(&owner,
            &String::from_str(&env, "svc"),
            &String::from_str(&env, "http://x"),
            &String::from_str(&env, "desc"),
            &10000, &soroban_sdk::vec![&env]);
        client.deregister_service(&0);
        let all = client.get_all_services();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn test_update_price() {
        let (env, _admin, client) = setup();
        let owner = Address::generate(&env);
        client.register_service(&owner,
            &String::from_str(&env, "svc"),
            &String::from_str(&env, "http://x"),
            &String::from_str(&env, "d"),
            &10000, &soroban_sdk::vec![&env]);
        client.update_price(&0, &99999);
        assert_eq!(client.get_service(&0).unwrap().price_usdc, 99999);
    }
}
