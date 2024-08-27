use crate::structs::{Event, TicketStage, TicketType};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StorageModule {
    // EVENT //
    #[view(getAllEvents)]
    #[storage_mapper("allEvents")]
    fn events(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(getEvent)]
    #[storage_mapper("event")]
    fn event_by_id(&self, event_id: &ManagedBuffer) -> SingleValueMapper<Event<Self::Api>>;
    // EVENT //

    // TICKET TYPES //
    #[view(getAllTicketTypes)]
    #[storage_mapper("allTicketTypes")]
    fn ticket_types(&self, event_id: &ManagedBuffer) -> UnorderedSetMapper<ManagedBuffer>;

    #[view(getTicketType)]
    #[storage_mapper("ticketType")]
    fn ticket_type_by_id(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
    ) -> SingleValueMapper<TicketType<Self::Api>>;
    // TICKET TYPES //

    // TICKET STAGE //
    #[view(getTicketStages)]
    #[storage_mapper("ticketStages")]
    fn ticket_stages(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
    ) -> MapMapper<ManagedBuffer, TicketStage<Self::Api>>;

    #[view(getAllowedUsers)]
    #[storage_mapper("allowedUsers")]
    fn whitelist_wallets(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
    ) -> UnorderedSetMapper<ManagedAddress>;
    // TICKET STAGE //

    // BUY LIMITS PER USER //
    #[view(buysPerEvent)]
    #[storage_mapper("buysPerEvent")]
    fn buys_per_event(
        &self,
        user: &ManagedAddress,
        event_id: &ManagedBuffer,
    ) -> SingleValueMapper<u32>;

    #[view(buysPerTicketType)]
    #[storage_mapper("buysPerTicketType")]
    fn buys_per_ticket_type(
        &self,
        user: &ManagedAddress,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
    ) -> SingleValueMapper<u32>;

    #[view(buysPerTicketStage)]
    #[storage_mapper("buysPerTicketStage")]
    fn buys_per_ticket_stage(
        &self,
        user: &ManagedAddress,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
    ) -> SingleValueMapper<u32>;
    // BUY LIMITS PER USER //

    // COLLECTION MANAGEMENT //
    #[view(getNonce)]
    #[storage_mapper("nonce")]
    fn next_nonce(&self, ticker: &TokenIdentifier) -> SingleValueMapper<u32>;

    #[view(collections)]
    #[storage_mapper("collections")]
    fn collections(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getTokenByEventId)]
    #[storage_mapper("token")]
    fn token_manager(&self, event_id: &ManagedBuffer) -> NonFungibleTokenMapper<Self::Api>;

    #[view(getTransferWallets)]
    #[storage_mapper("transferWallets")]
    fn transfer_wallets(&self, event_id: &ManagedBuffer) -> UnorderedSetMapper<ManagedAddress>;
    // COLLECTION MANAGEMENT //

    // TICKETING MANAGEMENT //
    #[view(getFees)]
    #[storage_mapper("fees")]
    fn fees(&self) -> SingleValueMapper<BigUint>;

    #[view(getIncome)]
    #[storage_mapper("income")]
    fn income(&self) -> MapMapper<EgldOrEsdtTokenIdentifier, EgldOrEsdtTokenPayment>;
     // TICKETING MANAGEMENT //
}
