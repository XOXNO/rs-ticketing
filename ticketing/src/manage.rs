use crate::{
    common, events,
    settings::{self, CallbackProxy},
    storage,
    structs::{EventArgs, TicketStage, TicketStageArgs, TicketType, TicketTypeArgs},
    NFT_ISSUE_COST,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ManageModule:
    storage::StorageModule + events::EventsModule + common::CommonModule + settings::SettingsModule
{
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(createEvent)]
    fn create_event(
        &self,
        event_id: &ManagedBuffer,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        args: EventArgs,
    ) {
        let payment_amount = self.call_value().egld_value();
        require!(
            payment_amount.clone_value() == NFT_ISSUE_COST,
            "Invalid payment amount. Issue costs exactly 0.05 EGLD"
        );
        let mut map = self.events();
        require!(!map.contains(event_id), "The ID has been used already");

        map.insert(event_id.clone());

        let caller = self.blockchain().get_caller();

        self.token_manager(event_id).issue_and_set_all_roles(
            EsdtTokenType::NonFungible,
            payment_amount.clone_value(),
            token_name,
            token_ticker,
            0,
            Some(self.callbacks().issue_callback(event_id, args, caller)),
        );
    }

    #[only_owner]
    #[endpoint(createTicketType)]
    fn create_ticket_type(&self, event_id: &ManagedBuffer, args: &TicketTypeArgs<Self::Api>) {
        self.is_event_valid(event_id);

        let mut map_types = self.ticket_types(event_id);
        require!(
            !map_types.contains(&args.id),
            "This ID has been created already!"
        );
        map_types.insert(args.id.clone());

        let ticket_type = TicketType {
            id: args.id.clone(),
            base_name: args.base_name.clone(),
            image: args.image.clone(),
            royalties: args.royalties.clone(),
            max_per_user: args.max_per_user,
            mint_limit: args.mint_limit,
            mint_count: 0,
        };

        self.ticket_type_by_id(event_id, &args.id).set(&ticket_type);
        self.emit_ticket_type(&ticket_type, event_id);
    }

    #[only_owner]
    #[endpoint(createTicketStage)]
    fn create_ticket_stage(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        args: &TicketStageArgs<Self::Api>,
    ) {
        self.is_event_valid(event_id);
        self.is_ticket_type_valid(event_id, ticket_type_id);

        let mut map_stages = self.ticket_stages(event_id, ticket_type_id);
        require!(!map_stages.contains_key(&args.id), "The key already exists");
        let ticket_stage = TicketStage {
            id: args.id.clone(),
            prices: args.prices.clone(),
            has_whitelist: args.has_whitelist,
            start_time: args.start_time,
            end_time: args.end_time,
            active: args.active,
            max_per_user: args.max_per_user,
            mint_limit: args.mint_limit,
            ticket_type_id: ticket_type_id.clone(),
            mint_count: 0,
        };
        self.emit_ticket_stage(&ticket_stage, event_id);
        map_stages.insert(args.id.clone(), ticket_stage);
    }

    #[only_owner]
    #[endpoint(removeTicketType)]
    fn remove_ticket_type(&self, event_id: &ManagedBuffer, ticket_type_id: &ManagedBuffer) {
        self.is_event_valid(event_id);
        self.is_ticket_type_valid(event_id, ticket_type_id);
        self.ticket_stages(event_id, ticket_type_id).clear();
        self.ticket_types(event_id).swap_remove(&ticket_type_id);
        self.ticket_type_by_id(event_id, ticket_type_id).clear();
        self.emit_remove_ticket_type(event_id, ticket_type_id);
    }

    #[only_owner]
    #[endpoint(removeTicketStage)]
    fn remove_ticket_stage(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
    ) {
        self.is_event_valid(event_id);
        self.is_ticket_type_valid(event_id, ticket_type_id);
        let mut map = self.ticket_stages(event_id, ticket_type_id);
        let removed_stage = map.remove(ticket_stage_id);
        if removed_stage.is_some() {
            self.emit_remove_ticket_stage(event_id, removed_stage.unwrap());
        }
    }

    #[only_owner]
    #[endpoint(editTicketType)]
    fn edit_ticket_type(&self, event_id: &ManagedBuffer, args: TicketTypeArgs<Self::Api>) {
        self.is_event_valid(event_id);

        let map = self.is_ticket_type_valid(event_id, &args.id);
        let mut old_value = map.get();

        old_value.base_name = args.base_name;
        old_value.image = args.image;
        old_value.royalties = args.royalties;
        old_value.mint_limit = args.mint_limit;
        old_value.max_per_user = args.max_per_user;

        map.set(&old_value);
        self.emit_ticket_type(&old_value, event_id);
    }

    #[only_owner]
    #[endpoint(editTicketStage)]
    fn edit_ticket_stage(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        args: TicketStageArgs<Self::Api>,
    ) {
        self.is_event_valid(event_id);
        self.is_ticket_type_valid(event_id, ticket_type_id);
        let mut map = self.ticket_stages(event_id, ticket_type_id);

        let option_value = map.get(&args.id);

        if option_value.is_some() {
            let mut old_value = option_value.unwrap();
            old_value.has_whitelist = args.has_whitelist;
            old_value.max_per_user = args.max_per_user;
            old_value.end_time = args.end_time;
            old_value.start_time = args.start_time;
            old_value.mint_limit = args.mint_limit;
            old_value.active = args.active;
            old_value.prices = args.prices;

            self.emit_ticket_stage(&old_value, event_id);
            map.insert(args.id, old_value);
        }
    }

    #[only_owner]
    #[endpoint(editEvent)]
    fn edit_event(&self, event_id: &ManagedBuffer, args: EventArgs) {
        let event_map = self.is_event_valid(event_id);
        let mut event = event_map.get();
        event.max_capacity = args.max_capacity;
        event.max_per_user = args.max_per_user;
        event.has_kyc = args.has_kyc;
        event.refund_policy = args.refund_policy;
        event.append_number = args.append_number;
        event.bot_protection = args.bot_protection;
        self.emit_event(&event);
        event_map.set(event);
    }

    #[only_owner]
    #[endpoint(addWhitelists)]
    fn add_to_whitelist(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
        wallets: MultiValueEncoded<ManagedAddress>,
    ) {
        self.is_event_valid(event_id);
        self.is_ticket_type_valid(event_id, ticket_type_id);

        let mut mapper = self.whitelist_wallets(event_id, ticket_type_id, ticket_stage_id);
        self.emit_whitelist_event(&wallets.to_vec());
        mapper.extend(wallets);
    }

    #[only_owner]
    #[endpoint(removeWhitelists)]
    fn remove_from_whitelist(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
        wallets: MultiValueEncoded<ManagedAddress>,
    ) {
        self.is_event_valid(event_id);
        self.is_ticket_type_valid(event_id, ticket_type_id);

        let mut mapper = self.whitelist_wallets(event_id, ticket_type_id, ticket_stage_id);

        self.emit_whitelist_event(&wallets.to_vec());
        for wallet in wallets {
            mapper.swap_remove(&wallet);
        }
    }
}
