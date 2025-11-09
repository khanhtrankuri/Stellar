#![no_std]

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, IntoVal, Symbol, Vec, Val,
};

/// Tên keys để lưu storage dạng tuple (Symbol, BytesN<32>)
fn prop_key() -> Symbol {
    symbol_short!("PROPERTY")
}
fn lease_key() -> Symbol {
    symbol_short!("LEASE")
}
fn escrow_key() -> Symbol {
    symbol_short!("ESCROW")
}

/// Trạng thái escrow
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EscrowStatus {
    Pending,
    Released,
    Cancelled,
}

impl EscrowStatus {
    fn as_u32(&self) -> u32 {
        match self {
            EscrowStatus::Pending => 0,
            EscrowStatus::Released => 1,
            EscrowStatus::Cancelled => 2,
        }
    }
    fn from_u32(v: u32) -> EscrowStatus {
        match v {
            1 => EscrowStatus::Released,
            2 => EscrowStatus::Cancelled,
            _ => EscrowStatus::Pending,
        }
    }
}

#[contract]
pub struct RealEstateContract;

#[contractimpl]
impl RealEstateContract {
    // -------------------------
    // Property registry
    // -------------------------
    /// Register a property with id (BytesN<32>), owner address and metadata (e.g., IPFS hash or JSON string)
    pub fn register_property(env: Env, property_id: BytesN<32>, owner: Address, metadata: Symbol) {
        let key = (prop_key(), property_id.clone());
        // store tuple (owner, metadata)
        env.storage()
            .persistent()
            .set(&key, &(owner.clone(), metadata.clone()));

        // emit an event
        // NOTE: publish is currently available but deprecated in favor of #[contractevent]
        env.events().publish(
            (symbol_short!("PROPREG"),),
            Vec::<Val>::from_array(
                &env,
                [
                    property_id.into_val(&env),
                    owner.clone().into_val(&env),
                    metadata.into_val(&env),
                ],
            ),
        );
    }

    /// Get property info: returns Option<(Address, Symbol)>
    pub fn get_property(env: Env, property_id: BytesN<32>) -> Option<(Address, Symbol)> {
        let key = (prop_key(), property_id);
        env.storage().persistent().get(&key)
    }

    /// Transfer ownership (sale) — only current owner can call
    pub fn transfer_property(
        env: Env,
        invoker: Address,
        property_id: BytesN<32>,
        new_owner: Address,
    ) {
        let caller = invoker;
        let key = (prop_key(), property_id.clone());
        let opt: Option<(Address, Symbol)> = env.storage().persistent().get(&key);
        match opt {
            None => panic!("Property not found"),
            Some((current_owner, metadata)) => {
                // verify invoker is current owner
                if caller != current_owner.clone() {
                    panic!("Only current owner can transfer");
                }
                env.storage()
                    .persistent()
                    .set(&key, &(new_owner.clone(), metadata.clone()));
                env.events().publish(
                    (symbol_short!("PROPXFER"),),
                    Vec::<Val>::from_array(
                        &env,
                        [
                            property_id.into_val(&env),
                            current_owner.into_val(&env),
                            new_owner.into_val(&env),
                        ],
                    ),
                );
            }
        }
    }

    // -------------------------
    // Lease management
    // -------------------------
    /// Create lease record
    /// lease_id: BytesN<32>, property_id: BytesN<32>, tenant: Address, start_ts: i64, end_ts: i64, rent: i64
    pub fn create_lease(
        env: Env,
        lease_id: BytesN<32>,
        property_id: BytesN<32>,
        tenant: Address,
        start_ts: i64,
        end_ts: i64,
        rent: i64,
    ) {
        let key = (lease_key(), lease_id.clone());
        // tuple stored: (property_id, tenant, start_ts, end_ts, rent, active_i32)
        let active: i32 = 1;
        env.storage().persistent().set(
            &key,
            &(
                property_id.clone(),
                tenant.clone(),
                start_ts,
                end_ts,
                rent,
                active,
            ),
        );
        env.events().publish(
            (symbol_short!("LEASENEW"),),
            Vec::<Val>::from_array(
                &env,
                [
                    lease_id.into_val(&env),
                    property_id.into_val(&env),
                    tenant.clone().into_val(&env),
                    start_ts.into_val(&env),
                    end_ts.into_val(&env),
                    rent.into_val(&env),
                ],
            ),
        );
    }

    /// Get lease info, returns Option<(BytesN<32>, Address, i64, i64, i64, i32)>
    pub fn get_lease(
        env: Env,
        lease_id: BytesN<32>,
    ) -> Option<(BytesN<32>, Address, i64, i64, i64, i32)> {
        let key = (lease_key(), lease_id);
        env.storage().persistent().get(&key)
    }

    /// End lease (set active to 0) — can be called by tenant or property owner
    pub fn end_lease(env: Env, invoker: Address, lease_id: BytesN<32>) {
        let caller = invoker;
        let key = (lease_key(), lease_id.clone());
        let opt: Option<(BytesN<32>, Address, i64, i64, i64, i32)> =
            env.storage().persistent().get(&key);
        match opt {
            None => panic!("Lease not found"),
            Some((property_id, tenant, start_ts, end_ts, rent, _active)) => {
                // allow only tenant or property owner
                let prop_key = (prop_key(), property_id.clone());
                let prop_opt: Option<(Address, Symbol)> = env.storage().persistent().get(&prop_key);
                if prop_opt.is_none() {
                    panic!("Property not found");
                }
                let (owner, _meta) = prop_opt.unwrap();
                if caller != tenant.clone() && caller != owner.clone() {
                    panic!("Only tenant or owner can end lease");
                }
                env.storage().persistent().set(
                    &key,
                    &(property_id, tenant, start_ts, end_ts, rent, 0i32),
                );
                env.events().publish(
                    (symbol_short!("LEASEEND"),),
                    Vec::<Val>::from_array(&env, [lease_id.into_val(&env)]),
                );
            }
        }
    }

    // -------------------------
    // Escrow (ký gửi) basic flow
    // -------------------------
    /// Create escrow: store escrow details; money transfer must be handled off-chain/through token transfer calls or extended later.
    pub fn create_escrow(
        env: Env,
        escrow_id: BytesN<32>,
        property_id: BytesN<32>,
        buyer: Address,
        seller: Address,
        arbiter: Address,
        amount: i64,
    ) {
        let key = (escrow_key(), escrow_id.clone());
        // store as tuple (property_id, buyer, seller, arbiter, amount, status_u32)
        let status_u32: u32 = EscrowStatus::Pending.as_u32();
        env.storage().persistent().set(
            &key,
            &(
                property_id.clone(),
                buyer.clone(),
                seller.clone(),
                arbiter.clone(),
                amount,
                status_u32,
            ),
        );
        env.events().publish(
            (symbol_short!("ESCROWNEW"),),
            Vec::<Val>::from_array(
                &env,
                [
                    escrow_id.into_val(&env),
                    property_id.into_val(&env),
                    buyer.clone().into_val(&env),
                    seller.clone().into_val(&env),
                    arbiter.clone().into_val(&env),
                    amount.into_val(&env),
                ],
            ),
        );
    }

    /// Release escrow: arbiter only -> set status Released and optionally transfer ownership
    pub fn release_escrow(
        env: Env,
        invoker: Address,
        escrow_id: BytesN<32>,
        transfer_ownership: bool,
    ) {
        let caller = invoker;
        let key = (escrow_key(), escrow_id.clone());
        let opt: Option<(BytesN<32>, Address, Address, Address, i64, u32)> =
            env.storage().persistent().get(&key);
        match opt {
            None => panic!("Escrow not found"),
            Some((property_id, buyer, seller, arbiter, amount, _status_u32)) => {
                // require caller == arbiter
                if caller != arbiter.clone() {
                    panic!("Only arbiter can release escrow");
                }

                // change status
                let new_status = EscrowStatus::Released.as_u32();
                env.storage().persistent().set(
                    &key,
                    &(
                        property_id.clone(),
                        buyer.clone(),
                        seller.clone(),
                        arbiter.clone(),
                        amount,
                        new_status,
                    ),
                );
                env.events().publish(
                    (symbol_short!("ESCROWREL"),),
                    Vec::<Val>::from_array(&env, [escrow_id.into_val(&env), amount.into_val(&env)]),
                );

                // optionally transfer ownership seller -> buyer
                if transfer_ownership {
                    // ensure property exists and seller is current owner
                    let pkey = (prop_key(), property_id.clone());
                    let prop_opt: Option<(Address, Symbol)> = env.storage().persistent().get(&pkey);
                    if prop_opt.is_none() {
                        panic!("Property not found");
                    }
                    let (current_owner, metadata) = prop_opt.unwrap();
                    if current_owner != seller.clone() {
                        panic!("Seller is not current owner");
                    }
                    env.storage()
                        .persistent()
                        .set(&pkey, &(buyer.clone(), metadata.clone()));
                    env.events().publish(
                        (symbol_short!("XFERESC"),),
                        Vec::<Val>::from_array(
                            &env,
                            [
                                property_id.into_val(&env),
                                seller.clone().into_val(&env),
                                buyer.clone().into_val(&env),
                            ],
                        ),
                    );
                }
            }
        }
    }

    /// Cancel escrow: arbiter only
    pub fn cancel_escrow(env: Env, invoker: Address, escrow_id: BytesN<32>) {
        let caller = invoker;
        let key = (escrow_key(), escrow_id.clone());
        let opt: Option<(BytesN<32>, Address, Address, Address, i64, u32)> =
            env.storage().persistent().get(&key);
        match opt {
            None => panic!("Escrow not found"),
            Some((property_id, buyer, seller, arbiter, amount, _status)) => {
                if caller != arbiter.clone() {
                    panic!("Only arbiter can cancel escrow");
                }
                let new_status = EscrowStatus::Cancelled.as_u32();
                env.storage().persistent().set(
                    &key,
                    &(
                        property_id.clone(),
                        buyer.clone(),
                        seller.clone(),
                        arbiter.clone(),
                        amount,
                        new_status,
                    ),
                );
                env.events().publish(
                    (symbol_short!("ESCROWCAN"),),
                    Vec::<Val>::from_array(&env, [escrow_id.into_val(&env), amount.into_val(&env)]),
                );
            }
        }
    }

    /// Query escrow info
    pub fn get_escrow(
        env: Env,
        escrow_id: BytesN<32>,
    ) -> Option<(BytesN<32>, Address, Address, Address, i64, u32)> {
        let key = (escrow_key(), escrow_id);
        env.storage().persistent().get(&key)
    }
}

mod test;