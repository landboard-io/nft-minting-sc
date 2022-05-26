#![no_std]

const ROYALTIES: u64 = 10_00;
const HASH_DATA_BUFFER_LEN: usize = 1024;

use core::convert::TryInto;
elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::contract]
pub trait NftMint {
    #[init]
    fn init(&self) {
        self.is_paused().set(true);
    }

    //ISSUANCE ENDPOINTS

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_collection(
        &self,
        #[payment] issue_cost: BigUint,
        collection_name: ManagedBuffer,
        collection_ticker: ManagedBuffer,
    ) {
        require!(self.nft_token_id().is_empty(), "Token already issued!");
        self.nft_token_name().set(&collection_name);
        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                issue_cost,
                &collection_name,
                &collection_ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback())
            .call_and_exit()
    }

    #[callback]
    fn issue_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
            }
        }
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");
        let roles = [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn];
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                roles.iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    #[only_owner]
    #[endpoint(populateIndexes)]
    fn populate_indexes(&self, total_number_of_nfts_to_add: u64) -> u64 {
        let mut indexes = self.s_indexes();
        let total_number_of_nfts = self.total_number_of_nfts().get();
        require!(
            &total_number_of_nfts_to_add >= &0,
            "Can't declare total number of NFTs as 0"
        );
        for i in 0..total_number_of_nfts_to_add {
            indexes.insert(total_number_of_nfts + i + 1);
        }
        self.total_number_of_nfts()
            .set(total_number_of_nfts + total_number_of_nfts_to_add);
        self.total_number_of_nfts().get()
    }

    #[only_owner]
    #[endpoint(populateBridgeIndexes)]
    fn populate_bridge_indexes(
        &self,
        old_collection: TokenIdentifier,
        #[var_args] pairs: MultiValueEncoded<MultiValue2<u64, u64>>,
    ) {
        let mut indexes = self.s_indexes();
        let mut bridge_indexes = self.bridge_indexes(&old_collection);
        for item in pairs.into_iter() {
            let tuple = item.into_tuple();
            require!(indexes.contains(&tuple.1), "Index not found");
            require!(
                !bridge_indexes.contains_key(&tuple.0),
                "Index already in bridge"
            );
            bridge_indexes.insert(tuple.0, tuple.1);
            indexes.swap_remove(&tuple.1);
            self.total_number_of_nfts().update(|v| *v -= 1u64);
        }
    }

    #[only_owner]
    #[endpoint(setCid)]
    fn set_cid(&self, cid: ManagedBuffer) {
        let indexes = self.s_indexes();
        let total_number_of_nfts = self.total_number_of_nfts().get();
        require!(
            total_number_of_nfts as usize == indexes.len(),
            "Can't change cid after minting started"
        );
        self.nft_token_cid().set(cid);
    }

    #[only_owner]
    #[endpoint(setRefPercent)]
    fn set_ref_percent(&self, reff: BigUint) {
        self.ref_percent().set(reff);
    }

    #[only_owner]
    #[endpoint(setDiscountPercent)]
    fn set_discount_percent(&self, disc: BigUint) {
        self.discount_percent().set(disc);
    }

    #[payable("*")]
    #[endpoint(mintRandomNft)]
    fn mint_random_nft(&self, #[var_args] ref_address: OptionalValue<ManagedAddress>) {
        require!(self.is_paused().get() == false, "Contract is paused");
        require!(self.s_indexes().len() > 0usize, "Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(), "CID is not set");
        require!(self.max_per_tx().get() > 0u64, "Max per tx not set");
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_amount > 0u64, "Payment must be more than 0");

        let price = self.selling_price(payment_token).get();
        require!(&price > &BigUint::from(0u64), "Can't mint with this token");
        require!(
            &payment_amount % &price == 0u64,
            "Wrong payment amount sent"
        );

        let nr_of_tokens = &payment_amount / &price;
        require!(&nr_of_tokens >= &1u64, "Minimum amount to buy is 1");
        require!(
            &nr_of_tokens <= &self.max_per_tx().get(),
            "Can't mint more than max per tx"
        );

        let tokens_available = self.s_indexes().len();
        require!(
            &nr_of_tokens <= &BigUint::from(tokens_available),
            "Not enough NFTs to mint"
        );

        let mut payments = ManagedVec::new();
        let mut rand_source = RandomnessSource::<Self::Api>::new();
        let mut i = BigUint::from(1u64);
        let step = BigUint::from(1u64);
        while i <= nr_of_tokens {
            let tokens_available = self.s_indexes().len();
            let number = rand_source.next_usize_in_range(1, tokens_available + 1) as u64;
            let index = self.v_indexes().get(number as usize);
            let token_id = self.nft_token_id().get();
            let token_name = self.create_name(index);
            let attributes = self.create_attributes(index);
            let hash_buffer = self
                .crypto()
                .sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
            let attributes_hash = hash_buffer.as_managed_buffer();
            let uris = self.create_uris(index);

            let nonce = self.send().esdt_nft_create(
                &token_id,
                &BigUint::from(1u64),
                &token_name,
                &BigUint::from(ROYALTIES),
                &attributes_hash,
                &attributes,
                &uris,
            );

            self.s_indexes().swap_remove(&index);
            payments.push(EsdtTokenPayment::new(token_id, nonce, BigUint::from(1u64)));
            i += &step;
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &payments, &[]);

        let owner = self.blockchain().get_owner_address();
        let (pay_amount, pay_token) = self.call_value().payment_token_pair();

        let mut ref_amount = BigUint::from(0u64);
        let mut discount_amount = BigUint::from(0u64);
        let ref_percent = self.ref_percent().get();
        let discount_percent = self.discount_percent().get();
        if ref_percent > BigUint::from(0u64) && discount_percent > BigUint::from(0u64) {
            if let OptionalValue::Some(ref_addr) = ref_address {
                require!(caller != ref_addr, "Caller can't refer themselves");
                if self.is_first_mint(&caller).is_empty() || self.is_first_mint(&caller).get() {
                    ref_amount = &pay_amount * &ref_percent / BigUint::from(100u64);
                    discount_amount = &pay_amount * &discount_percent / BigUint::from(100u64);
                    self.send()
                        .direct(&ref_addr, &pay_token, 0, &ref_amount, &[]);
                    self.send()
                        .direct(&caller, &pay_token, 0, &discount_amount, &[]);
                    self.ref_count(&ref_addr)
                        .set(self.ref_count(&ref_addr).get() + 1u64);
                    self.ref_money(&ref_addr)
                        .set(self.ref_money(&ref_addr).get() + &ref_amount);
                }
            }
        }
        self.is_first_mint(&caller).set(false);
        self.send().direct(
            &owner,
            &pay_token,
            0,
            &(pay_amount - ref_amount - discount_amount),
            &[],
        );
    }

    #[payable("*")]
    #[endpoint(mintSpecificNft)]
    fn mint_specific_nft(
        &self,
        numbers_to_mint: ManagedVec<u64>,
        #[var_args] ref_address: OptionalValue<ManagedAddress>,
    ) {
        let caller = self.blockchain().get_caller();
        require!(self.is_paused().get() == false, "Contract is paused");
        require!(self.s_indexes().len() > 0usize, "Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(), "CID is not set");
        require!(self.max_per_tx().get() > 0u64, "Max per tx not set");

        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_amount > 0u64, "Payment must be more than 0");

        let price = self.selling_specific_price(payment_token).get();
        require!(&price > &BigUint::from(0u64), "Can't mint with this token");
        require!(
            &payment_amount % &price == 0u64,
            "Wrong payment amount sent"
        );

        let nr_of_tokens = &payment_amount / &price;
        require!(
            &nr_of_tokens == &BigUint::from(numbers_to_mint.len()),
            "Wrong payment amount sent"
        );
        require!(
            &nr_of_tokens <= &self.max_per_tx().get(),
            "Can't mint more than max per tx"
        );

        let tokens_available = self.s_indexes().len();
        require!(
            &nr_of_tokens <= &BigUint::from(tokens_available),
            "Not enough NFTs to mint"
        );

        for number_to_mint in numbers_to_mint.iter() {
            require!(
                self.s_indexes().contains(&number_to_mint),
                "One of the NFTs requested was already minted"
            );
            let token_id = self.nft_token_id().get();
            let token_name = self.create_name(number_to_mint);
            let attributes = self.create_attributes(number_to_mint);
            let hash_buffer = self
                .crypto()
                .sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
            let attributes_hash = hash_buffer.as_managed_buffer();
            let uris = self.create_uris(number_to_mint);

            let nonce = self.send().esdt_nft_create(
                &token_id,
                &BigUint::from(1u64),
                &token_name,
                &BigUint::from(ROYALTIES),
                &attributes_hash,
                &attributes,
                &uris,
            );

            self.s_indexes().swap_remove(&number_to_mint);

            self.send()
                .direct(&caller, &token_id, nonce, &BigUint::from(1u64), &[]);
        }

        let owner = self.blockchain().get_owner_address();
        let (pay_amount, pay_token) = self.call_value().payment_token_pair();

        let mut ref_amount = BigUint::from(0u64);
        let mut discount_amount = BigUint::from(0u64);
        let ref_percent = self.ref_percent().get();
        let discount_percent = self.discount_percent().get();
        if ref_percent > BigUint::from(0u64) && discount_percent > BigUint::from(0u64) {
            if let OptionalValue::Some(ref_addr) = ref_address {
                require!(caller != ref_addr, "Caller can't refer themselves");
                if self.is_first_mint(&caller).is_empty() || self.is_first_mint(&caller).get() {
                    ref_amount = &pay_amount * &ref_percent / BigUint::from(100u64);
                    discount_amount = &pay_amount * &discount_percent / BigUint::from(100u64);
                    self.send()
                        .direct(&ref_addr, &pay_token, 0, &ref_amount, &[]);
                    self.send()
                        .direct(&caller, &pay_token, 0, &discount_amount, &[]);
                    self.ref_count(&ref_addr)
                        .set(self.ref_count(&ref_addr).get() + 1u64);
                    self.ref_money(&ref_addr)
                        .set(self.ref_money(&ref_addr).get() + &ref_amount);
                }
            }
        }
        self.is_first_mint(&caller).set(false);
        self.send().direct(
            &owner,
            &pay_token,
            0,
            &(pay_amount - ref_amount - discount_amount),
            &[],
        );
    }

    #[payable("*")]
    #[endpoint(bridgeNfts)]
    fn bridge_nfts(&self) {
        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers();
        for nft in payments.iter() {
            if let Some(index) = self
                .bridge_indexes(&nft.token_identifier)
                .remove(&nft.token_nonce)
            {
                let token_id = self.nft_token_id().get();
                let token_name = self.create_name(index);
                let attributes = self.create_attributes(index);
                let hash_buffer = self
                    .crypto()
                    .sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
                let attributes_hash = hash_buffer.as_managed_buffer();
                let uris = self.create_uris(index);

                let nonce = self.send().esdt_nft_create(
                    &token_id,
                    &BigUint::from(1u64),
                    &token_name,
                    &BigUint::from(ROYALTIES),
                    &attributes_hash,
                    &attributes,
                    &uris,
                );
                self.send()
                    .direct(&caller, &token_id, nonce, &BigUint::from(1u64), &[]);
            } else {
                require!(false, "NFT can't be bridged");
            }
        }
    }

    #[only_owner]
    #[endpoint(giveaway)]
    fn giveaway(&self, number_to_mint: BigUint, giveaway_address: ManagedAddress) {
        require!(self.is_paused().get() == false, "Contract is paused");
        require!(self.s_indexes().len() > 0usize, "Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(), "CID is not set");
        require!(self.max_per_tx().get() > 0u64, "Max per tx not set");
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_amount > 0u64, "Payment must be more than 0");

        let price = self.selling_price(payment_token).get();
        require!(&price > &BigUint::from(0u64), "Can't mint with this token");
        require!(
            &payment_amount % &price == 0u64,
            "Wrong payment amount sent"
        );

        let nr_of_tokens = number_to_mint;
        require!(&nr_of_tokens >= &1u64, "Minimum amount to buy is 1");
        require!(
            &nr_of_tokens <= &self.max_per_tx().get(),
            "Can't mint more than max per tx"
        );

        let tokens_available = self.s_indexes().len();
        require!(
            &nr_of_tokens <= &BigUint::from(tokens_available),
            "Not enough NFTs to mint"
        );

        let mut payments = ManagedVec::new();
        let mut rand_source = RandomnessSource::<Self::Api>::new();
        let mut i = BigUint::from(1u64);
        let step = BigUint::from(1u64);
        while i <= nr_of_tokens {
            let tokens_available = self.s_indexes().len();
            let number = rand_source.next_usize_in_range(1, tokens_available + 1) as u64;
            let index = self.v_indexes().get(number as usize);
            let token_id = self.nft_token_id().get();
            let token_name = self.create_name(index);
            let attributes = self.create_attributes(index);
            let hash_buffer = self
                .crypto()
                .sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
            let attributes_hash = hash_buffer.as_managed_buffer();
            let uris = self.create_uris(index);

            let nonce = self.send().esdt_nft_create(
                &token_id,
                &BigUint::from(1u64),
                &token_name,
                &BigUint::from(ROYALTIES),
                &attributes_hash,
                &attributes,
                &uris,
            );

            self.s_indexes().swap_remove(&index);
            payments.push(EsdtTokenPayment::new(token_id, nonce, BigUint::from(1u64)));
            i += &step;
        }

        let caller = giveaway_address;
        self.send().direct_multi(&caller, &payments, &[]);
    }

    //STATE
    #[only_owner]
    #[endpoint(pause)]
    fn pause(&self) {
        require!(self.s_indexes().len() > 0usize, "Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(), "CID is not set");
        require!(self.max_per_tx().get() > 0u64, "Max per tx not set");
        let pause_value = &self.is_paused().get();
        if self.is_paused().is_empty() {
            self.is_paused().set(true);
        } else {
            self.is_paused().set(!pause_value);
        }
    }

    #[only_owner]
    #[endpoint(setPrice)]
    fn set_price(&self, token_id: TokenIdentifier, price: BigUint) {
        require!(price > BigUint::from(0u64), "Can't set price to 0");
        self.selling_price(token_id).set(&price);
    }

    #[only_owner]
    #[endpoint(setSpecificPrice)]
    fn set_specific_price(&self, token_id: TokenIdentifier, price: BigUint) {
        require!(price > BigUint::from(0u64), "Can't set price to 0");
        self.selling_specific_price(token_id).set(&price);
    }

    #[only_owner]
    #[endpoint(setMaxPerTx)]
    fn set_max_per_tx(&self, max_per_tx: BigUint) {
        self.max_per_tx().set(&max_per_tx);
    }

    //HELPERS
    fn create_attributes(&self, number: u64) -> ManagedBuffer {
        let cid = self.nft_token_cid().get();
        let mut attributes = ManagedBuffer::new_from_bytes("metadata:".as_bytes());
        attributes.append(&cid);
        attributes.append(&ManagedBuffer::new_from_bytes("/".as_bytes()));
        attributes.append(&self.decimal_to_ascii(number));
        attributes.append(&ManagedBuffer::new_from_bytes(".json;".as_bytes()));
        attributes
    }

    fn create_uris(&self, number: u64) -> ManagedVec<ManagedBuffer> {
        let cid = self.nft_token_cid().get();
        let mut uris = ManagedVec::new();
        let mut media_uri = ManagedBuffer::new_from_bytes("https://ipfs.io/ipfs/".as_bytes());
        media_uri.append(&cid);
        media_uri.append(&ManagedBuffer::new_from_bytes("/".as_bytes()));
        media_uri.append(&self.decimal_to_ascii(number));
        media_uri.append(&ManagedBuffer::new_from_bytes(".jpg".as_bytes()));
        uris.push(media_uri);
        let mut metadata_uri = ManagedBuffer::new_from_bytes("https://ipfs.io/ipfs/".as_bytes());
        metadata_uri.append(&cid);
        metadata_uri.append(&ManagedBuffer::new_from_bytes("/".as_bytes()));
        metadata_uri.append(&self.decimal_to_ascii(number));
        metadata_uri.append(&ManagedBuffer::new_from_bytes(".json".as_bytes()));
        uris.push(metadata_uri);
        uris
    }

    fn create_name(&self, number: u64) -> ManagedBuffer {
        let mut full_token_name = ManagedBuffer::new();
        let token_name_from_storage = self.nft_token_name().get();
        let token_index = self.decimal_to_ascii(number);
        let hash_sign = ManagedBuffer::new_from_bytes(" #".as_bytes());
        full_token_name.append(&token_name_from_storage);
        full_token_name.append(&hash_sign);
        full_token_name.append(&token_index);
        full_token_name
    }

    fn decimal_to_ascii(&self, mut number: u64) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: u64 = 10;
        const ZERO_ASCII: u8 = b'0';

        let mut as_ascii = [0u8; MAX_NUMBER_CHARACTERS as usize];
        let mut nr_chars = 0;

        loop {
            unsafe {
                let reminder: u8 = (number % 10).try_into().unwrap_unchecked();
                number /= 10;

                as_ascii[nr_chars] = ZERO_ASCII + reminder;
                nr_chars += 1;
            }

            if number == 0 {
                break;
            }
        }

        let slice = &mut as_ascii[..nr_chars];
        slice.reverse();

        ManagedBuffer::new_from_bytes(slice)
    }

    #[view(getDidMint)]
    fn did_mint(&self, address: ManagedAddress) -> bool {
        if self.is_first_mint(&address).is_empty() || self.is_first_mint(&address).get() == true {
            false
        } else {
            true
        }
    }

    //STORAGE

    //NFTS
    #[view(getNftTokenId)]
    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getNftTokenName)]
    #[storage_mapper("nftTokenName")]
    fn nft_token_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getNftTokenCid)]
    #[storage_mapper("nftTokenCid")]
    fn nft_token_cid(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getNumberOfNfts)]
    #[storage_mapper("totalNumberOfNfts")]
    fn total_number_of_nfts(&self) -> SingleValueMapper<u64>;

    #[view(getIndexes)]
    #[storage_mapper("indexes")]
    fn v_indexes(&self) -> VecMapper<u64>;

    #[storage_mapper("indexes")]
    fn s_indexes(&self) -> UnorderedSetMapper<u64>;

    #[storage_mapper("bridgeIndexes")]
    fn bridge_indexes(&self, old_collection: &TokenIdentifier) -> MapMapper<u64, u64>;

    //SELLING
    #[storage_mapper("is_paused")]
    fn is_paused(&self) -> SingleValueMapper<bool>;

    #[view(getSftPrice)]
    #[storage_mapper("sftPrice")]
    fn selling_price(&self, token_id: TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getSftSpecificPrice)]
    #[storage_mapper("sftSpecificPrice")]
    fn selling_specific_price(&self, token_id: TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getMaxPerTx)]
    #[storage_mapper("getMaxPerTx")]
    fn max_per_tx(&self) -> SingleValueMapper<BigUint>;

    #[view(getRefPercent)]
    #[storage_mapper("getRefPercent")]
    fn ref_percent(&self) -> SingleValueMapper<BigUint>;

    #[view(getDiscountPercent)]
    #[storage_mapper("getDiscountPercent")]
    fn discount_percent(&self) -> SingleValueMapper<BigUint>;

    #[view(getRefMoney)]
    #[storage_mapper("getRefMoney")]
    fn ref_money(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getRefCount)]
    #[storage_mapper("getRefCount")]
    fn ref_count(&self, address: &ManagedAddress) -> SingleValueMapper<u64>;

    #[storage_mapper("isFirstMint")]
    fn is_first_mint(&self, address: &ManagedAddress) -> SingleValueMapper<bool>;
}
