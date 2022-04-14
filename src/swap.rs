#![no_std]


const ROYALTIES: u32 = 10_00;
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
    fn issue_collection(&self,#[payment] issue_cost:BigUint,collection_name:ManagedBuffer,collection_ticker:ManagedBuffer){
        require!(self.nft_token_id().is_empty(), "Token already issued!");
        self.nft_token_name().set(&collection_name);
        self.send().esdt_system_sc_proxy().issue_non_fungible(
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
            }
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
        let roles = [
            EsdtLocalRole::NftCreate,
            EsdtLocalRole::NftBurn,
        ];
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                roles.iter().cloned()
            )
            .async_call()
            .call_and_exit()
    }

    #[only_owner]
    #[endpoint(populateIndexes)]
    fn populate_indexes(&self, total_number_of_nfts_to_add:u32)->u32{
        let mut indexes=self.indexes();
        let total_number_of_nfts=self.total_number_of_nfts().get();
        require!(indexes.len()==0,"Indexes already populated");
        require!(&total_number_of_nfts_to_add>=&0,"Can't declare total number of NFTs as 0");
        for i in 0..total_number_of_nfts_to_add{
            indexes.push(&(total_number_of_nfts+i+1));
        }
        self.total_number_of_nfts().set(total_number_of_nfts+total_number_of_nfts_to_add);
        self.total_number_of_nfts().get()
    }

    #[only_owner]
    #[endpoint(setCid)]
    fn set_cid(&self, cid:ManagedBuffer){
        let indexes=self.indexes();
        let total_number_of_nfts=self.total_number_of_nfts().get();
        require!(total_number_of_nfts as usize==indexes.len(),"Can't change cid after minting started");
        self.nft_token_cid().set(cid);
    }

    #[only_owner]
    #[endpoint(setRefPercent)]
    fn set_ref_percent(&self, reff:BigUint){
        self.ref_percent().set(reff);
    }

    #[payable("*")]
    #[endpoint(mintRandomNft)]
    fn mint_random_nft(&self,#[var_args] ref_address: OptionalValue<ManagedAddress>){
        require!(self.is_paused().get()==false,"Contract is paused");
        require!(self.indexes().len()>0usize,"Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(),"CID is not set");
        require!(self.max_per_tx().get()>0u64,"Max per tx not set");
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_amount > 0u64, "Payment must be more than 0");

        let price=self.selling_price(payment_token).get();
        require!(&price>&BigUint::from(0u64),"Can't mint with this token");
        require!(&payment_amount%&price==0u64,"Wrong payment amount sent");

        let nr_of_tokens=&payment_amount/&price;
        require!(&nr_of_tokens>=&1u64,"Minimum amount to buy is 1");
        require!(&nr_of_tokens<=&self.max_per_tx().get(),"Can't mint more than max per tx");
        
        let tokens_available=self.indexes().len();
        require!(&nr_of_tokens<=&BigUint::from(tokens_available),"Not enough NFTs to mint");

        let mut payments = ManagedVec::new();
        let mut rand_source = RandomnessSource::<Self::Api>::new();
        let mut i=BigUint::from(1u32);
        let step=BigUint::from(1u32);
        while i<=nr_of_tokens{
            let number=rand_source.next_usize_in_range(1,tokens_available+1) as u32;
        
            let index=self.indexes().get(number.try_into().unwrap());
            let token_id = self.nft_token_id().get();
            let token_name=self.create_name(index);
            let attributes=self.create_attributes(index);
            let hash_buffer=self.crypto().sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
            let attributes_hash = hash_buffer.as_managed_buffer();
            let uris=self.create_uris(index);

            let nonce=self.send().esdt_nft_create(
                        &token_id,
                        &BigUint::from(1u64),
                        &token_name,
                        &BigUint::from(ROYALTIES),
                        &attributes_hash,
                        &attributes,
                        &uris);
            
            self.indexes().swap_remove(number.try_into().unwrap());
            payments.push(EsdtTokenPayment::new(token_id, nonce, BigUint::from(1u64)));
            i+=&step;
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &payments, &[]);

        let owner = self.blockchain().get_owner_address();
        let (pay_amount,pay_token)=self.call_value().payment_token_pair();

        let mut ref_amount=BigUint::from(0u32);
        let ref_percent=self.ref_percent().get();
        if ref_percent>BigUint::from(0u32){
            if let OptionalValue::Some(ref_addr)=ref_address{
                require!(caller!=ref_addr,"Caller can't refer themselves");
                if self.is_first_mint(&ref_addr).is_empty()||self.is_first_mint(&ref_addr).get(){
                    ref_amount=&pay_amount*&ref_percent/BigUint::from(100u32);
                    self.send().direct(&ref_addr,&pay_token,0,&ref_amount,&[]);
                    self.is_first_mint(&ref_addr).set(false);
                }
            }
        }

        self.send().direct(&owner, &pay_token, 0, &(pay_amount-ref_amount), &[]);
    }

    #[payable("*")]
    #[endpoint(mintSpecificNft)]
    fn mint_specific_nft(&self,number:u32,#[var_args] ref_address: OptionalValue<ManagedAddress>){
        require!(self.is_paused().get()==false,"Contract is paused");
        require!(self.indexes().len()>0usize,"Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(),"CID is not set");
        require!(self.max_per_tx().get()>0u64,"Max per tx not set");
        
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_amount > 0u64, "Payment must be more than 0");

        let price=self.selling_price(payment_token).get();
        require!(&price>&BigUint::from(0u64),"Can't mint with this token");
        require!(&payment_amount%&price==0u64,"Wrong payment amount sent");

        let nr_of_tokens=&payment_amount/&price;
        require!(&nr_of_tokens==&1u64,"Can only mint one specific NFT at a time");
        require!(&nr_of_tokens<=&self.max_per_tx().get(),"Can't mint more than max per tx");
        
        let tokens_available=self.indexes().len();
        require!(&nr_of_tokens<=&BigUint::from(tokens_available),"Not enough NFTs to mint");
        
        let indexes=self.indexes();
        let mut index=0u32;
        for i in indexes.iter(){
            if number==i{
                index=i;
                break;
            }}
        require!(index>0,"NFT already minted");

        let token_id = self.nft_token_id().get();
        let token_name=self.create_name(index);
        let attributes=self.create_attributes(index);
        let hash_buffer=self.crypto().sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
        let attributes_hash = hash_buffer.as_managed_buffer();
        let uris=self.create_uris(index);

        let nonce=self.send().esdt_nft_create(
                    &token_id,
                    &BigUint::from(1u64),
                    &token_name,
                    &BigUint::from(ROYALTIES),
                    &attributes_hash,
                    &attributes,
                    &uris);
        
        self.indexes().swap_remove(index.try_into().unwrap());

        let caller = self.blockchain().get_caller();
        self.send().direct(&caller, &token_id, nonce, &BigUint::from(1u32), &[]);

        let owner = self.blockchain().get_owner_address();
        let (pay_amount,pay_token)=self.call_value().payment_token_pair();

        let mut ref_amount=BigUint::from(0u32);
        let ref_percent=self.ref_percent().get();
        if ref_percent>BigUint::from(0u32){
            if let OptionalValue::Some(ref_addr)=ref_address{
                require!(caller!=ref_addr,"Caller can't refer themselves");
                if self.is_first_mint(&ref_addr).is_empty()||self.is_first_mint(&ref_addr).get(){
                    ref_amount=&pay_amount*&ref_percent/BigUint::from(100u32);
                    self.send().direct(&ref_addr,&pay_token,0,&ref_amount,&[]);
                    self.is_first_mint(&ref_addr).set(false);
                }
            }
        }
        self.send().direct(&owner, &pay_token, 0, &(pay_amount-ref_amount), &[]);
    }

    //STATE
    #[only_owner]
    #[endpoint(pause)]
    fn pause(&self){
        require!(self.indexes().len()>0usize,"Indexes are not populated");
        require!(!self.nft_token_cid().is_empty(),"CID is not set");
        require!(self.max_per_tx().get()>0u64,"Max per tx not set");
        let pause_value=&self.is_paused().get();
        if self.is_paused().is_empty(){
            self.is_paused().set(true);
        }else{
            self.is_paused().set(!pause_value);
        }
        
    }

    #[only_owner]
    #[endpoint(setPrice)]
    fn set_price(&self, token_id:TokenIdentifier, price: BigUint) {
        require!(price>BigUint::from(0u32),"Can't set price to 0");
        self.selling_price(token_id).set(&price);
    }

    #[only_owner]
    #[endpoint(setMaxPerTx)]
    fn set_max_per_tx(&self, max_per_tx: BigUint) {
        self.max_per_tx().set(&max_per_tx);
    }


    //HELPERS
    fn create_attributes(&self,number:u32) -> ManagedBuffer{
        let cid=self.nft_token_cid().get();
        let mut attributes=ManagedBuffer::new_from_bytes("metadata:".as_bytes());
        attributes.append(&cid);
        attributes.append(&ManagedBuffer::new_from_bytes("/".as_bytes()));
        attributes.append(&self.decimal_to_ascii(number));
        attributes.append(&ManagedBuffer::new_from_bytes(".json;".as_bytes()));
        attributes
    }

    fn create_uris(&self,number:u32)->ManagedVec<ManagedBuffer>{
        let cid=self.nft_token_cid().get();
        let mut uris=ManagedVec::new();
        let mut media_uri=ManagedBuffer::new_from_bytes("https://ipfs.io/ipfs/".as_bytes());
        media_uri.append(&cid);
        media_uri.append(&ManagedBuffer::new_from_bytes("/".as_bytes()));
        media_uri.append(&self.decimal_to_ascii(number));
        media_uri.append(&ManagedBuffer::new_from_bytes(".jpg".as_bytes()));
        uris.push(media_uri);
        let mut metadata_uri=ManagedBuffer::new_from_bytes("https://ipfs.io/ipfs/".as_bytes());
        metadata_uri.append(&cid);
        metadata_uri.append(&ManagedBuffer::new_from_bytes("/".as_bytes()));
        metadata_uri.append(&self.decimal_to_ascii(number));
        metadata_uri.append(&ManagedBuffer::new_from_bytes(".json".as_bytes()));
        uris.push(metadata_uri);
        uris
    }

    fn create_name(&self,number:u32)->ManagedBuffer{
        let mut full_token_name = ManagedBuffer::new();
        let token_name_from_storage = self.nft_token_name().get();
        let token_index = self.decimal_to_ascii(number);
        let hash_sign = ManagedBuffer::new_from_bytes(" #".as_bytes());
        full_token_name.append(&token_name_from_storage);
        full_token_name.append(&hash_sign);
        full_token_name.append(&token_index);
        full_token_name
    }

    fn decimal_to_ascii(&self, mut number: u32) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: u32 = 10;
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

    fn caller_from_option_or_owner(
        &self,
        caller: OptionalValue<ManagedAddress>,
    ) -> ManagedAddress {
        caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_owner_address())
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
    fn total_number_of_nfts(&self) -> SingleValueMapper<u32>;

    #[view(getIndexes)]
    #[storage_mapper("indexes")]
    fn indexes(&self) -> VecMapper<u32>;

    //SELLING
    #[storage_mapper("is_paused")]
    fn is_paused(&self) -> SingleValueMapper<bool>;

    #[view(getSftPrice)]
    #[storage_mapper("sftPrice")]
    fn selling_price(&self, token_id:TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getMaxPerTx)]
    #[storage_mapper("getMaxPerTx")]
    fn max_per_tx(&self) -> SingleValueMapper<BigUint>;

    #[view(getRefPercent)]
    #[storage_mapper("getRefPercent")]
    fn ref_percent(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("isFirstMint")]
    fn is_first_mint(&self, address:&ManagedAddress) -> SingleValueMapper<bool>;
}