use crate::{aggregator::*, manager_proxy};
use crate::{structs::*, NFT_AMOUNT, ROYALTIES_MAX};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CommonModule: crate::storage::StorageModule + crate::events::EventsModule {
    fn is_event_valid(&self, event_id: &ManagedBuffer) -> SingleValueMapper<Event<Self::Api>> {
        let map = self.event_by_id(event_id);
        require!(
            !self.event_by_id(event_id).is_empty(),
            "Your event ID: {} is not valid!",
            (event_id)
        );
        return map;
    }

    fn is_ticket_type_valid(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
    ) -> SingleValueMapper<TicketType<Self::Api>> {
        let map = self.ticket_type_by_id(event_id, ticket_type_id);
        require!(
            !self.ticket_type_by_id(event_id, ticket_type_id).is_empty(),
            "Your ticket type ID: {} is not valid!",
            (ticket_type_id)
        );

        return map;
    }

    fn is_ticket_stage_valid(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
    ) -> TicketStage<Self::Api> {
        let map = self.ticket_stages(event_id, ticket_type_id);
        require!(
            map.contains_key(&ticket_stage_id),
            "Your ticket stage ID: {} is not valid!",
            (ticket_stage_id)
        );

        return map.get(&ticket_stage_id).unwrap();
    }

    fn does_event_exists(&self, event_id: &ManagedBuffer) -> Event<Self::Api> {
        let map = self.event_by_id(event_id);
        require!(!map.is_empty(), "The event is invalid!");
        map.get()
    }

    fn does_ticket_type_exists(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
    ) -> TicketType<Self::Api> {
        let map = self.ticket_type_by_id(event_id, ticket_type_id);
        require!(!map.is_empty(), "The ticket type is invalid!");
        map.get()
    }

    fn send_nft(
        &self,
        event: &mut Event<Self::Api>,
        ticket_type: &mut TicketType<Self::Api>,
        ticket_stage_opt: Option<&mut TicketStage<Self::Api>>,
        to: &ManagedAddress,
        nfts_to_send: usize,
    ) -> PaymentsVec<Self::Api> {
        require!(
            !self.blockchain().is_smart_contract(to),
            "Only user accounts are allowed to mint"
        );
        let has_stage = ticket_stage_opt.is_some();
        let ticket_stage = ticket_stage_opt.unwrap();
        let mut nft_output_payments = ManagedVec::new();
        let map_nonce = self.next_nonce(&event.token);
        let mut nonce = map_nonce.get();
        let nft_amount = BigUint::from(NFT_AMOUNT);
        for _ in 0..nfts_to_send {
            let base_name = ticket_type.base_name.clone();
            let nft_name = self.get_nft_name(nonce, &base_name, &event);
            let attributes = ManagedBuffer::new();
            let mut uris = ManagedVec::new();
            let url_image = ticket_type.image.clone();

            uris.push(url_image);

            let nft_nonce = self.send().esdt_nft_create(
                &event.token,
                &nft_amount,
                &nft_name,
                &ticket_type.royalties,
                &ManagedBuffer::new(),
                &attributes,
                &uris,
            );

            nonce += 1;
            nft_output_payments.push(EsdtTokenPayment::new(
                event.token.clone(),
                nft_nonce,
                nft_amount.clone(),
            ));
        }

        self.buys_per_event(to, &event.id)
            .update(|counts| *counts += nfts_to_send as u32);
        self.buys_per_ticket_type(to, &event.id, &ticket_type.id)
            .update(|counts| *counts += nfts_to_send as u32);
        if has_stage {
            self.buys_per_ticket_stage(to, &event.id, &ticket_type.id, &ticket_stage.id)
                .update(|counts| *counts += nfts_to_send as u32);
        }

        // set the last nonce of the minted NFT
        map_nonce.set(nonce);

        self.send().direct_multi(to, &nft_output_payments);

        event.mint_count += nfts_to_send as u32;
        ticket_type.mint_count += nfts_to_send as u32;
        if has_stage {
            ticket_stage.mint_count += nfts_to_send as u32;
        }

        if has_stage {
            self.emit_stage_mint(
                &ticket_type,
                &ticket_stage,
                &event,
                event.mint_count == event.max_capacity,
            );
        } else {
            self.emit_type_mint(&ticket_type, &event, event.mint_count == event.max_capacity);
        }

        if has_stage {
            self.ticket_stages(&event.id, &ticket_type.id)
                .insert(ticket_stage.id.clone(), ticket_stage.clone());
        }
        self.ticket_type_by_id(&event.id, &ticket_type.id)
            .set(ticket_type);
        self.event_by_id(&event.id).set(event);

        nft_output_payments
    }

    #[allow_multiple_var_args]
    fn check_kyc(
        &self,
        event: &Event<Self::Api>,
        ticket_type: &TicketType<Self::Api>,
        ticket_stage: &TicketStage<Self::Api>,
        caller: &ManagedAddress,
        quantity: usize,
        signature: OptionalValue<ManagedBuffer>,
        data: OptionalValue<ManagedBuffer>,
    ) {
        let sign = signature.into_option();
        let message = data.into_option();
        if event.has_kyc {
            let mut computed = ManagedBuffer::new();
            let msg = &message.unwrap();
            computed.append(caller.as_managed_buffer());
            computed.append(&event.id);
            computed.append(&ticket_type.id);
            computed.append(&ticket_stage.id);
            computed.append(&sc_format!("{}", quantity));
            computed.append(&sc_format!("has_kyc"));
            require!(computed.eq(msg), "The payload is invalid!");
            self.crypto().verify_ed25519(
                self.get_signer().as_managed_buffer(),
                msg,
                &sign.unwrap(),
            );
        } else if event.bot_protection {
            require!(sign.is_some(), "Signature required!");
            require!(message.is_some(), "Data required!");
            let msg = &message.unwrap();
            let mut computed = ManagedBuffer::new();
            computed.append(caller.as_managed_buffer());
            computed.append(&event.id);
            computed.append(&ticket_type.id);
            computed.append(&ticket_stage.id);
            computed.append(&sc_format!("{}", quantity));
            computed.append(&sc_format!("bot_protection"));

            require!(computed.eq(msg), "The payload is invalid!");
            self.crypto().verify_ed25519(
                self.get_signer().as_managed_buffer(),
                msg,
                &sign.unwrap(),
            );
        }
    }

    fn common_buy_check(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
        quantity: usize,
        caller: &ManagedAddress,
    ) -> MultiValue3<Event<Self::Api>, TicketType<Self::Api>, TicketStage<Self::Api>> {
        let event = self.does_event_exists(event_id);
        let ticket_type = self.does_ticket_type_exists(event_id, ticket_type_id);
        let ticket_stage = self.is_ticket_stage_valid(event_id, ticket_type_id, ticket_stage_id);

        self.require_is_minting(&ticket_stage);
        let time_now = self.blockchain().get_block_timestamp();

        require!(
            time_now >= ticket_stage.start_time,
            "The stage mint starts in the future!"
        );

        require!(
            ticket_stage.end_time == 0 || time_now <= ticket_stage.end_time,
            "The stage mint has ended!"
        );

        if ticket_stage.has_whitelist {
            require!(
                self.is_whitelisted(event_id, ticket_type_id, ticket_stage_id, caller),
                "You are not on the whitelist!"
            );
        }

        require!(
            quantity > 0,
            "The quantity {} has to be higer than 0!",
            quantity
        );

        self.check_buys_limits(caller, quantity, &event, &ticket_type, &ticket_stage);

        self.check_sold_out(&event, &ticket_type, &ticket_stage, quantity);

        (event, ticket_type, ticket_stage).into()
    }

    fn check_buys_limits(
        &self,
        caller: &ManagedAddress,
        quantity: usize,
        event: &Event<Self::Api>,
        ticket_type: &TicketType<Self::Api>,
        ticket_stage: &TicketStage<Self::Api>,
    ) {
        if ticket_stage.max_per_user > 0u32 {
            let stage_counts = self
                .buys_per_ticket_stage(caller, &event.id, &ticket_type.id, &ticket_stage.id)
                .get();
            require!(
                stage_counts + quantity as u32 <= ticket_stage.max_per_user,
                "Max buys per ticket stage will be over the maximum of {}!",
                (&ticket_stage.max_per_user)
            );
        }

        if ticket_type.max_per_user > 0u32 {
            let ticket_type_counts = self
                .buys_per_ticket_type(caller, &event.id, &ticket_type.id)
                .get();

            require!(
                ticket_type_counts + quantity as u32 <= ticket_type.max_per_user,
                "Max buys per stage will be over the maximum of {}!",
                (&ticket_type.max_per_user)
            );
        }

        if event.max_per_user > 0u32 {
            let event_counts = self.buys_per_event(caller, &event.id).get();
            require!(
                event_counts + quantity as u32 <= event.max_per_user,
                "Max buys per event will be over the maximum of {}!",
                (&event.max_per_user)
            );
        }
    }

    fn common_payment_check(
        &self,
        ticket_stage: &TicketStage<Self::Api>,
        quantity: usize,
        swaps: OptionalValue<ManagedVec<AggregatorStep<Self::Api>>>,
        limits: OptionalValue<ManagedVec<TokenAmount<Self::Api>>>,
    ) -> MultiValue2<EgldOrEsdtTokenPayment, BigUint> {
        let payment = self.call_value().egld_or_single_esdt();

        let has_swap = swaps.is_some() && limits.is_some();

        if has_swap {
            let output = self.aggregate(
                &payment.token_identifier,
                payment.amount,
                swaps.into_option().unwrap(),
                limits.into_option().unwrap(),
            );
            let swap_index_price: Option<usize> = ticket_stage.prices.iter().position(|r| {
                r.token_identifier == output.token_identifier.clone()
                    && r.token_nonce == output.token_nonce
            });
            require!(swap_index_price.is_some(), "Swap invalid!");
            let price_per_nft = ticket_stage.prices.get(swap_index_price.unwrap()).amount;
            let total_value = BigUint::from(quantity).mul(&price_per_nft);

            require!(
                &total_value <= &output.amount,
                "The payment amount is under the total value required for the buy!"
            );
            let token = EgldOrEsdtTokenIdentifier::parse(
                output.token_identifier.as_managed_buffer().clone(),
            );
            if &output.amount > &total_value {
                self.send().direct(
                    &self.blockchain().get_caller(),
                    &token,
                    output.token_nonce,
                    &(output.amount - &total_value),
                );
            }
            (
                EgldOrEsdtTokenPayment::new(token, output.token_nonce, total_value),
                price_per_nft,
            )
                .into()
        } else {
            let index_price = ticket_stage.prices.iter().position(|r| {
                r.token_identifier
                    == TokenIdentifier::from(payment.token_identifier.clone().into_name())
                    && r.token_nonce == payment.token_nonce
            });

            require!(index_price.is_some(), "Payment invalid!");
            let price_per_nft = ticket_stage.prices.get(index_price.unwrap()).amount;
            let total_value = BigUint::from(quantity).mul(&price_per_nft);
            require!(
                &total_value == &payment.amount,
                "The payment amount is wrong!"
            );
            (payment, price_per_nft).into()
        }
    }

    fn check_sold_out(
        &self,
        event: &Event<Self::Api>,
        ticket_type: &TicketType<Self::Api>,
        ticket_stage: &TicketStage<Self::Api>,
        count: usize,
    ) {
        require!(
            ticket_type.mint_limit >= ticket_type.mint_count + count as u32,
            "The ticket type capacity is sold out!"
        );

        require!(
            ticket_stage.mint_limit >= ticket_stage.mint_count + count as u32,
            "The ticket stage capacity is sold out!"
        );

        require!(
            event.max_capacity >= event.mint_count + count as u32,
            "The event capacity would be over the maximum!"
        );
    }

    fn check_type_sold_out(
        &self,
        event: &Event<Self::Api>,
        ticket_type: &TicketType<Self::Api>,
        count: usize,
    ) {
        require!(
            ticket_type.mint_limit >= ticket_type.mint_count + count as u32,
            "The ticket type capacity is sold out!"
        );

        require!(
            event.max_capacity >= event.mint_count + count as u32,
            "The event capacity would be over the maximum!"
        );
    }

    fn distribute_income(&self, payment: EgldOrEsdtTokenPayment) {
        if payment.token_nonce > 0 {
            // self.send().direct(
            //     &self.local_owner().get(),
            //     &payment_type,
            //     payment_nonce,
            //     &payment_amount,
            // );
        } else if payment.amount > 0 {
            let cut = self.fees().get();
            let owner_cut = BigUint::from(ROYALTIES_MAX) - &cut;
            let platform_cut = self.calculate_cut_amount(&payment.amount, &cut);
            let owner_revenue = self.calculate_cut_amount(&payment.amount, &owner_cut);

            if platform_cut.gt(&BigUint::zero()) {
                self.send().direct(
                    &self.blockchain().get_owner_address(),
                    &payment.token_identifier,
                    payment.token_nonce,
                    &platform_cut,
                );
            }

            if owner_revenue.gt(&BigUint::zero()) {
                let mut map = self.income();
                if map.contains_key(&payment.token_identifier) {
                    let mut data = map.get(&payment.token_identifier).unwrap();
                    data.amount += payment.amount;
                    map.insert(payment.token_identifier, data);
                } else {
                    map.insert(payment.token_identifier.clone(), payment.clone());
                }
            }
        }
    }

    fn get_nft_name(
        &self,
        nonce: u32,
        name: &ManagedBuffer,
        event: &Event<Self::Api>,
    ) -> ManagedBuffer {
        if event.append_number {
            sc_format!("{}{}", name, nonce)
        } else {
            sc_format!("{}", name)
        }
    }

    fn require_is_minting(&self, stage: &TicketStage<Self::Api>) {
        require!(stage.active, "The sale is not active yet for this stage!");
    }

    fn calculate_cut_amount(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / ROYALTIES_MAX
    }

    fn aggregate(
        &self,
        token: &EgldOrEsdtTokenIdentifier,
        amount: BigUint,
        steps: ManagedVec<AggregatorStep<Self::Api>>,
        limits: ManagedVec<TokenAmount<Self::Api>>,
    ) -> EsdtTokenPayment<Self::Api> {
        let call = self
            .tx()
            .to(self.get_aggregator())
            .typed(AggregatorContractProxy);

        if token.is_esdt() {
            let all_payments = call
                .aggregate_esdt(steps, limits, true, OptionalValue::<ManagedAddress>::None)
                .esdt((token.clone().unwrap_esdt(), 0, amount))
                .returns(ReturnsBackTransfers)
                .sync_call();
            all_payments.esdt_payments.get(0)
        } else {
            let all_payments = call
                .aggregate_egld(steps, limits, OptionalValue::<ManagedAddress>::None)
                .egld(amount)
                .returns(ReturnsBackTransfers)
                .sync_call();
            all_payments.esdt_payments.get(0)
        }
    }

    #[view(isWhitelisted)]
    fn is_whitelisted(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
        address: &ManagedAddress,
    ) -> bool {
        self.whitelist_wallets(event_id, ticket_type_id, ticket_stage_id)
            .contains(&address)
    }

    fn get_signer(&self) -> ManagedAddress {
        return self
            .tx()
            .to(self.blockchain().get_owner_address())
            .typed(manager_proxy::ManagerProxy)
            .signer()
            .returns(ReturnsResult)
            .sync_call_readonly();
    }

    fn get_aggregator(&self) -> ManagedAddress {
        return self
            .tx()
            .to(self.blockchain().get_owner_address())
            .typed(manager_proxy::ManagerProxy)
            .aggregator_sc()
            .returns(ReturnsResult)
            .sync_call_readonly();
    }
}
