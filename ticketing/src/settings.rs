use crate::{
    structs::{Event, EventArgs},
    ROYALTIES_MAX,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait SettingsModule:
    crate::storage::StorageModule + crate::common::CommonModule + crate::events::EventsModule
{
    #[callback]
    fn issue_callback(
        &self,
        event_id: &ManagedBuffer,
        args: EventArgs,
        caller: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                if !EgldOrEsdtTokenIdentifier::esdt(token_id.clone()).is_egld() {
                    self.next_nonce(&token_id).set_if_empty(&1u32);
                    self.collections().insert(token_id.clone());
                    self.token_manager(event_id).set_token_id(token_id.clone());
                    let event = Event {
                        token: token_id,
                        transfer_role: false,
                        max_capacity: args.max_capacity,
                        max_per_user: args.max_per_user,
                        mint_count: 0,
                        id: event_id.clone(),
                        fees: self.fees().get(),
                        has_kyc: args.has_kyc,
                        refund_policy: args.refund_policy,
                        append_number: args.append_number,
                        bot_protection: args.bot_protection,
                    };
                    self.event_by_id(event_id).set(&event);
                    self.emit_event(&event);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.events().swap_remove(event_id);
                let amount = self.call_value().egld_value();
                if &amount.clone_value() > &0 {
                    self.send().direct_egld(&caller, &amount);
                }
            }
        }
    }

    #[only_owner]
    #[endpoint(setFees)]
    fn set_cut_fees(&self, fees: BigUint) {
        require!(
            fees < ROYALTIES_MAX,
            "Invalid percentage value, should be under 10,000"
        );
        self.fees().set(&fees);
    }
}
