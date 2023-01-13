#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod token {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    use ink_lang as ink;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Token {
        total_supply: u32,
        balances: Mapping<AccountId, u32>,
        allowances: Mapping<(AccountId, AccountId), u32>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: u32,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: u32,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new(initial_supply: u32) -> Self {
            ink::utils::initialize_contract(|contract: &mut Self| {
                Self::new_init(contract, initial_supply)
            })
        }

        pub fn new_init(&mut self, initial_supply: u32) {
            let caller = Self::env().caller();
            self.total_supply = initial_supply;
            self.balances.insert(&caller, &initial_supply);
            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            })
        }

        #[ink(message)]
        pub fn total_supply(&self) -> u32 {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u32 {
            self.balances.get(&owner).unwrap_or_default()
        }

        pub fn transfer(&mut self, to: AccountId, value: u32) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        fn transfer_from_to(&mut self, from: &AccountId, to: &AccountId, value: u32) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            } else {
                self.balances.insert(from, &(from_balance - value));
                let to_balance = self.balance_of_impl(to);
                self.balances.insert(to, &(to_balance + value));
                self.env().emit_event(Transfer {
                    from: Some(*from),
                    to: Some(*to),
                    value,
                });

                Ok(())
            }
        }

        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> u32 {
            self.balances.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: u32) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert(&(owner, spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u32 {
            self.allowance_impl(&owner, &spender)
        }

        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> u32 {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: u32) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            } else {
                self.transfer_from_to(&from, &to, value)?;
                self.allowances.insert(&(from, caller), &(allowance - value));
                Ok(())
            }
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
            let contract = Token::new(4294967000);
            assert_eq!(contract.total_supply(), 4294967000);
        }

        #[ink::test]
        fn balance_works() {
            let contract = Token::new(4294967000);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 4294967000);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Token::new(4294967000);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
            assert_eq!(contract.transfer(AccountId::from([0x0; 32]), 4294967000), Ok(()));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 4294967000);
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Token::new(4294967000);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 4294967000);
            contract.approve(AccountId::from([0x1; 32]), 1000000).unwrap();
            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 69).unwrap();
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 69);
        }

        #[ink::test]
        fn allowance_works() {
            let mut contract = Token::new(4294967000);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 4294967000);
            contract.approve(AccountId::from([0x1; 32]), 1000000).unwrap();
            assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 1000000);
        }
    }
}
