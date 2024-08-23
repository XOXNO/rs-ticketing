#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";


#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "buyTicket" => interact.buy().await,
        "issueFreeTicket" => interact.giveaway().await,
        "issuePaidTicket" => interact.giveaway_admin().await,
        "setFees" => interact.set_cut_fees().await,
        "getAllEvents" => interact.events().await,
        "getEvent" => interact.event_by_id().await,
        "getAllTicketTypes" => interact.ticket_types().await,
        "getTicketType" => interact.ticket_type_by_id().await,
        "getTicketStages" => interact.ticket_stages().await,
        "getAllowedUsers" => interact.whitelist_wallets().await,
        "buysPerEvent" => interact.buys_per_event().await,
        "buysPerTicketType" => interact.buys_per_ticket_type().await,
        "buysPerTicketStage" => interact.buys_per_ticket_stage().await,
        "getNonce" => interact.next_nonce().await,
        "collections" => interact.collections().await,
        "getTokenByEventId" => interact.token_manager().await,
        "getFees" => interact.fees().await,
        "getIncome" => interact.income().await,
        "isWhitelisted" => interact.is_whitelisted().await,
        "getEvents" => interact.get_events().await,
        "getTypes" => interact.get_types().await,
        "getTypeStages" => interact.get_type_stages().await,
        "getAllStages" => interact.get_all_stages().await,
        "getAllIncomeTokens" => interact.get_all_income_tokens().await,
        "getIncomePayment" => interact.get_all_income_payments().await,
        "whitelistSize" => interact.whitelisted_size().await,
        "createEvent" => interact.create_event().await,
        "createTicketType" => interact.create_ticket_type().await,
        "createTicketStage" => interact.create_ticket_stage().await,
        "removeTicketType" => interact.remove_ticket_type().await,
        "removeTicketStage" => interact.remove_ticket_stage().await,
        "editTicketType" => interact.edit_ticket_type().await,
        "editTicketStage" => interact.edit_ticket_stage().await,
        "addWhitelists" => interact.add_to_whitelist().await,
        "removeWhitelists" => interact.remove_from_whitelist().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>
}

impl State {
        // Deserializes state from file
        pub fn load_state() -> Self {
            if Path::new(STATE_FILE).exists() {
                let mut file = std::fs::File::open(STATE_FILE).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                toml::from_str(&content).unwrap()
            } else {
                Self::default()
            }
        }
    
        /// Sets the contract address
        pub fn set_address(&mut self, address: Bech32Address) {
            self.contract_address = Some(address);
        }
    
        /// Returns the contract address
        pub fn current_address(&self) -> &Bech32Address {
            self.contract_address
                .as_ref()
                .expect("no known contract, deploy first")
        }
    }
    
    impl Drop for State {
        // Serializes state to file
        fn drop(&mut self) {
            let mut file = std::fs::File::create(STATE_FILE).unwrap();
            file.write_all(toml::to_string(self).unwrap().as_bytes())
                .unwrap();
        }
    }

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());
        
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/ticketing.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state()
        }
    }

    async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
    }

    async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn buy(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let quantity = 0u32;
        let signature = OptionalValue::Some(ManagedBuffer::new_from_bytes(&b""[..]));
        let data = OptionalValue::Some(ManagedBuffer::new_from_bytes(&b""[..]));
        let swaps = OptionalValue::Some(ManagedVec::<StaticApi, AggregatorStep<StaticApi>>::new());
        let limits = OptionalValue::Some(ManagedVec::<StaticApi, TokenAmount<StaticApi>>::new());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .buy(event_id, ticket_type_id, ticket_stage_id, quantity, signature, data, swaps, limits)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn giveaway(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let users = MultiValueVec::from(vec![MultiValue2::<ManagedAddress<StaticApi>, u32>::from((bech32::decode(""), 0u32))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .giveaway(event_id, ticket_type_id, users)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn giveaway_admin(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let to = bech32::decode("");
        let quantity = 0u32;
        let external_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let signature = OptionalValue::Some(ManagedBuffer::new_from_bytes(&b""[..]));
        let data = OptionalValue::Some(ManagedBuffer::new_from_bytes(&b""[..]));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .giveaway_admin(event_id, ticket_type_id, ticket_stage_id, to, quantity, external_id, signature, data)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_cut_fees(&mut self) {
        let fees = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .set_cut_fees(fees)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn events(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .events()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn event_by_id(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .event_by_id(event_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn ticket_types(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .ticket_types(event_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn ticket_type_by_id(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .ticket_type_by_id(event_id, ticket_type_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn ticket_stages(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .ticket_stages(event_id, ticket_type_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn whitelist_wallets(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .whitelist_wallets(event_id, ticket_type_id, ticket_stage_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn buys_per_event(&mut self) {
        let user = bech32::decode("");
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .buys_per_event(user, event_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn buys_per_ticket_type(&mut self) {
        let user = bech32::decode("");
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .buys_per_ticket_type(user, event_id, ticket_type_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn buys_per_ticket_stage(&mut self) {
        let user = bech32::decode("");
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .buys_per_ticket_stage(user, event_id, ticket_type_id, ticket_stage_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn next_nonce(&mut self) {
        let ticker = TokenIdentifier::from_esdt_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .next_nonce(ticker)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn collections(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .collections()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn token_manager(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .token_manager(event_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn fees(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .fees()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn income(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .income()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn is_whitelisted(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .is_whitelisted(event_id, ticket_type_id, ticket_stage_id, address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_events(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .get_events()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_types(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .get_types(event_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_type_stages(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .get_type_stages(event_id, ticket_type_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_all_stages(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .get_all_stages(event_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_all_income_tokens(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .get_all_income_tokens()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_all_income_payments(&mut self) {
        let token = EgldOrEsdtTokenIdentifier::esdt(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .get_all_income_payments(token)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn whitelisted_size(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::TicketingProxy)
            .whitelisted_size(event_id, ticket_type_id, ticket_stage_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn create_event(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(0u128);

        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let token_name = ManagedBuffer::new_from_bytes(&b""[..]);
        let token_ticker = ManagedBuffer::new_from_bytes(&b""[..]);
        let args = EventArgs::<StaticApi>::default();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .create_event(event_id, token_name, token_ticker, args)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_ticket_type(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let args = TicketTypeArgs::<StaticApi>::default();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .create_ticket_type(event_id, args)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_ticket_stage(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let args = TicketStageArgs::<StaticApi>::default();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .create_ticket_stage(event_id, ticket_type_id, args)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_ticket_type(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .remove_ticket_type(event_id, ticket_type_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_ticket_stage(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .remove_ticket_stage(event_id, ticket_type_id, ticket_stage_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn edit_ticket_type(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let args = TicketTypeArgs::<StaticApi>::default();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .edit_ticket_type(event_id, ticket_type_id, args)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn edit_ticket_stage(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let args = TicketStageArgs::<StaticApi>::default();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .edit_ticket_stage(event_id, ticket_type_id, ticket_stage_id, args)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_to_whitelist(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let wallets = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .add_to_whitelist(event_id, ticket_type_id, ticket_stage_id, wallets)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_from_whitelist(&mut self) {
        let event_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_type_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let ticket_stage_id = ManagedBuffer::new_from_bytes(&b""[..]);
        let wallets = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::TicketingProxy)
            .remove_from_whitelist(event_id, ticket_type_id, ticket_stage_id, wallets)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

}
