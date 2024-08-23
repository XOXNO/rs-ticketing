#![no_std]

use aggregator::{AggregatorStep, TokenAmount};

#[allow(unused_imports)]
use multiversx_sc::imports::*;

const NFT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
const ROYALTIES_MAX: u32 = 10_000;
const NFT_AMOUNT: u32 = 1;

pub mod aggregator;
pub mod common;
pub mod events;
pub mod manage;
pub mod manager_proxy;
pub mod settings;
pub mod storage;
pub mod structs;
pub mod views;

#[multiversx_sc::contract]
pub trait Ticketing:
    events::EventsModule
    + settings::SettingsModule
    + storage::StorageModule
    + common::CommonModule
    + views::ViewsModule
    + manage::ManageModule
{
    #[init]
    fn init(&self, fees: BigUint) {
        self.set_cut_fees(fees);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[allow_multiple_var_args]
    #[endpoint(buyTicket)]
    #[payable("*")]
    fn buy(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
        quantity: usize,
        signature: OptionalValue<ManagedBuffer>,
        data: OptionalValue<ManagedBuffer>,
        swaps: OptionalValue<ManagedVec<AggregatorStep<Self::Api>>>,
        limits: OptionalValue<ManagedVec<TokenAmount<Self::Api>>>,
    ) -> ManagedVec<EsdtTokenPayment> {
        let caller = self.blockchain().get_caller();

        let (mut event, mut ticket_type, mut ticket_stage) = self
            .common_buy_check(event_id, ticket_type_id, ticket_stage_id, quantity, &caller)
            .into_tuple();

        let (payment, price_per_nft) = self
            .common_payment_check(&ticket_stage, quantity, swaps, limits)
            .into_tuple();

        self.check_kyc(
            &event,
            &ticket_type,
            &ticket_stage,
            &caller,
            quantity,
            signature,
            data,
        );

        let payments = self.send_nft(
            &mut event,
            &mut ticket_type,
            Option::Some(&mut ticket_stage),
            &caller,
            quantity,
        );

        self.emit_buy(
            &payments,
            &payment.token_identifier,
            &caller,
            &price_per_nft,
            &event.token,
            &ManagedBuffer::new(),
            &event,
            &ticket_type,
        );
        self.distribute_income(payment);
        payments
    }

    #[endpoint(issueFreeTicket)]
    #[only_owner]
    fn giveaway(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        users: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) -> ManagedVec<EsdtTokenPayment> {
        let mut event = self.does_event_exists(event_id);
        let mut ticket_type = self.does_ticket_type_exists(event_id, ticket_type_id);
        let mut all_payments = ManagedVec::new();
        for user in users {
            let (to, quantity) = user.into_tuple();
            self.check_type_sold_out(&event, &ticket_type, quantity);
            let payments = self.send_nft(&mut event, &mut ticket_type, Option::None, &to, quantity);

            self.emit_buy(
                &payments,
                &EgldOrEsdtTokenIdentifier::egld(),
                &to,
                &BigUint::zero(),
                &event.token,
                &ManagedBuffer::new(),
                &event,
                &ticket_type,
            );

            all_payments.append_vec(payments);
        }
        all_payments
    }

    #[allow_multiple_var_args]
    #[only_owner]
    #[endpoint(issuePaidTicket)]
    fn giveaway_admin(
        &self,
        event_id: &ManagedBuffer,
        ticket_type_id: &ManagedBuffer,
        ticket_stage_id: &ManagedBuffer,
        to: &ManagedAddress,
        quantity: usize,
        external_id: &ManagedBuffer,
        signature: OptionalValue<ManagedBuffer>,
        data: OptionalValue<ManagedBuffer>,
    ) -> ManagedVec<EsdtTokenPayment> {
        let (mut event, mut ticket_type, mut ticket_stage) = self
            .common_buy_check(event_id, ticket_type_id, ticket_stage_id, quantity, to)
            .into_tuple();

        self.check_kyc(
            &event,
            &ticket_type,
            &ticket_stage,
            to,
            quantity,
            signature,
            data,
        );

        let payments = self.send_nft(
            &mut event,
            &mut ticket_type,
            Option::Some(&mut ticket_stage),
            to,
            quantity,
        );

        self.emit_buy(
            &payments,
            &EgldOrEsdtTokenIdentifier::egld(),
            to,
            &BigUint::zero(),
            &event.token,
            external_id,
            &event,
            &ticket_type,
        );
        payments
    }
}
