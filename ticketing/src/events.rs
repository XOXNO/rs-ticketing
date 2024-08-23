use crate::structs::{Event, TicketStage, TicketType};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait EventsModule: crate::storage::StorageModule {
    #[event("emit_whitelist_event")]
    fn emit_whitelist_event(&self, #[indexed] wallets: &ManagedVec<ManagedAddress>);

    #[event("emit_create_event")]
    fn emit_event(&self, #[indexed] event: &Event<Self::Api>);

    fn emit_remove_ticket_stage(
        &self,
        event_id: &ManagedBuffer,
        ticket_type: TicketStage<Self::Api>,
    ) {
        let ticker = self.token_manager(event_id).get_token_id();
        self._emit_remove_ticket_stage(event_id, ticket_type, ticker);
    }

    #[event("emit_remove_ticket_stage")]
    fn _emit_remove_ticket_stage(
        &self,
        #[indexed] event_id: &ManagedBuffer,
        #[indexed] ticket_stage: TicketStage<Self::Api>,
        #[indexed] ticker: TokenIdentifier<Self::Api>,
    );

    fn emit_remove_ticket_type(&self, event_id: &ManagedBuffer, ticket_type_id: &ManagedBuffer) {
        let ticker = self.token_manager(event_id).get_token_id();
        self._emit_remove_ticket_type(event_id, ticket_type_id, ticker);
    }

    #[event("emit_remove_ticket_type_event")]
    fn _emit_remove_ticket_type(
        &self,
        #[indexed] event_id: &ManagedBuffer,
        #[indexed] ticket_type_id: &ManagedBuffer,
        #[indexed] ticker: TokenIdentifier<Self::Api>,
    );

    fn emit_ticket_type(&self, ticket_type: &TicketType<Self::Api>, event_id: &ManagedBuffer) {
        let ticker = self.token_manager(event_id).get_token_id();
        self._emit_ticket_type_event(ticket_type, ticker);
    }

    #[event("emit_ticket_type_event")]
    fn _emit_ticket_type_event(
        &self,
        #[indexed] ticket_type: &TicketType<Self::Api>,
        #[indexed] ticker: TokenIdentifier<Self::Api>,
    );

    fn emit_ticket_stage(&self, ticket_stage: &TicketStage<Self::Api>, event_id: &ManagedBuffer) {
        let ticker = self.token_manager(event_id).get_token_id();
        self._emit_ticket_stage_event(ticket_stage, ticker);
    }

    #[event("emit_ticket_stage_event")]
    fn _emit_ticket_stage_event(
        &self,
        #[indexed] ticket_stage: &TicketStage<Self::Api>,
        #[indexed] ticker: TokenIdentifier<Self::Api>,
    );

    #[event("emit_stage_mint_event")]
    fn emit_stage_mint(
        &self,
        #[indexed] ticket_type: &TicketType<Self::Api>,
        #[indexed] ticket_stage: &TicketStage<Self::Api>,
        #[indexed] event: &Event<Self::Api>,
        #[indexed] is_global_sold_out: bool,
    );

    #[event("emit_type_mint_event")]
    fn emit_type_mint(
        &self,
        #[indexed] ticket_type: &TicketType<Self::Api>,
        #[indexed] event: &Event<Self::Api>,
        #[indexed] is_global_sold_out: bool,
    );

    #[event("emit_buy_event")]
    fn emit_buy_event(
        &self,
        #[indexed] payments: &ManagedVec<EsdtTokenPayment>,
        #[indexed] payment_token: &EgldOrEsdtTokenIdentifier,
        #[indexed] buyer: &ManagedAddress,
        #[indexed] price: &BigUint,
        #[indexed] token_identifier: &TokenIdentifier,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
        #[indexed] external_id: &ManagedBuffer,
    );

    fn emit_buy(
        &self,
        payments: &ManagedVec<EsdtTokenPayment>,
        payment_token: &EgldOrEsdtTokenIdentifier,
        buyer: &ManagedAddress,
        price: &BigUint,
        token_identifier: &TokenIdentifier,
        external_id: &ManagedBuffer,
    ) {
        let epoch = self.blockchain().get_block_epoch();
        self.emit_buy_event(
            payments,
            payment_token,
            buyer,
            price,
            token_identifier,
            self.blockchain().get_block_timestamp(),
            epoch,
            external_id,
        );
    }
}
