multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Event<M: ManagedTypeApi> {
    pub token: TokenIdentifier<M>,
    pub transfer_role: bool,
    pub id: ManagedBuffer<M>,
    pub max_capacity: u32,
    pub max_per_user: u32,
    pub fees: BigUint<M>,
    pub mint_count: u32,
    pub has_kyc: bool,
    pub refund_policy: bool,
    pub append_number: bool,
    pub bot_protection: bool,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct EventArgs {
    pub max_capacity: u32,
    pub max_per_user: u32,
    pub has_kyc: bool,
    pub refund_policy: bool,
    pub append_number: bool,
    pub bot_protection: bool,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Attributes<M: ManagedTypeApi> {
    pub is_check_in: bool,
    pub event_id: ManagedBuffer<M>,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct TicketType<M: ManagedTypeApi> {
    pub base_name: ManagedBuffer<M>,
    pub image: ManagedBuffer<M>,
    pub royalties: BigUint<M>,
    pub id: ManagedBuffer<M>,
    pub max_per_user: u32,
    pub mint_limit: u32,
    pub mint_count: u32,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct TicketTypeArgs<M: ManagedTypeApi> {
    pub base_name: ManagedBuffer<M>,
    pub image: ManagedBuffer<M>,
    pub royalties: BigUint<M>,
    pub id: ManagedBuffer<M>,
    pub max_per_user: u32,
    pub mint_limit: u32,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct TicketStage<M: ManagedTypeApi> {
    pub prices: ManagedVec<M, EsdtTokenPayment<M>>,
    pub id: ManagedBuffer<M>,
    pub ticket_type_id: ManagedBuffer<M>,
    pub has_whitelist: bool,
    pub max_per_user: u32,
    pub mint_limit: u32,
    pub mint_count: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub active: bool,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct TicketStageArgs<M: ManagedTypeApi> {
    pub prices: ManagedVec<M, EsdtTokenPayment<M>>,
    pub id: ManagedBuffer<M>,
    pub has_whitelist: bool,
    pub max_per_user: u32,
    pub mint_limit: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub active: bool,
}

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
