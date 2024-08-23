use crate::structs::{Event, TicketStage, TicketType};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ViewsModule:
    crate::storage::StorageModule + crate::events::EventsModule + crate::common::CommonModule
{
    #[view(getEvents)]
    fn get_events(&self) -> ManagedVec<Event<Self::Api>> {
        let mut results = ManagedVec::new();
        let events = self.events();

        for event_id in events.iter() {
            results.push(self.does_event_exists(&event_id));
        }

        return results;
    }

    #[view(getTypes)]
    fn get_types(&self, event_id: &ManagedBuffer) -> ManagedVec<TicketType<Self::Api>> {
        let mut results = ManagedVec::new();
        let ticket_types = self.ticket_types(event_id);

        for ticket_type_id in ticket_types.iter() {
            results.push(self.ticket_type_by_id(event_id, &ticket_type_id).get());
        }

        results
    }

    #[view(getTypeStages)]
    fn get_type_stages(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
    ) -> ManagedVec<TicketStage<Self::Api>> {
        let mut results = ManagedVec::new();
        let ticket_stages = self.ticket_stages(event_id, ticket_type_id);

        for ticket_type_id in ticket_stages.values() {
            results.push(ticket_type_id);
        }

        results
    }

    #[view(getAllStages)]
    fn get_all_stages(&self, event_id: &ManagedBuffer) -> ManagedVec<TicketStage<Self::Api>> {
        let mut results = ManagedVec::new();
        let ticket_types = self.ticket_types(event_id);

        for ticket_type_id in ticket_types.iter() {
            let ticket_stages = self.ticket_stages(event_id, &ticket_type_id);

            for ticket_type_id in ticket_stages.values() {
                results.push(ticket_type_id);
            }
        }

        results
    }

    #[view(getAllIncomeTokens)]
    fn get_all_income_tokens(&self) -> ManagedVec<EgldOrEsdtTokenIdentifier> {
        return self.income().keys().collect();
    }

    #[view(getIncomePayment)]
    fn get_all_income_payments(&self, token: &EgldOrEsdtTokenIdentifier) -> EgldOrEsdtTokenPayment {
        return self.income().get(token).unwrap();
    }

    #[view(whitelistSize)]
    fn whitelisted_size(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
    ) -> usize {
        self.whitelist_wallets(event_id, ticket_type_id, ticket_stage_id)
            .len()
    }
}
