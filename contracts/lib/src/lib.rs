#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::vec::Vec;
use ink::storage::Mapping;
use propchain_traits::*;

#[ink::contract]
mod propchain_contracts {
    use super::*;

    /// Error types for contract
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        PropertyNotFound,
        Unauthorized,
        InvalidMetadata,
        NotCompliant, // Recipient is not compliant
        ComplianceCheckFailed, // Compliance registry call failed
    }

    /// Property Registry contract
    #[ink(storage)]
    pub struct PropertyRegistry {
        /// Mapping from property ID to property information
        properties: Mapping<u64, PropertyInfo>,
        /// Mapping from owner to their properties
        owner_properties: Mapping<AccountId, Vec<u64>>,
        /// Property counter
        property_count: u64,
        /// Compliance registry contract address (optional)
        compliance_registry: Option<AccountId>,
        /// Contract owner (for setting compliance registry)
        owner: AccountId,
    }

    #[ink(event)]
    pub struct PropertyRegistered {
        #[ink(topic)]
        property_id: u64,
        owner: AccountId,
    }

    #[ink(event)]
    pub struct PropertyTransferred {
        #[ink(topic)]
        property_id: u64,
        from: AccountId,
        to: AccountId,
    }

    impl PropertyRegistry {
        /// Creates a new PropertyRegistry contract
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                properties: Mapping::default(),
                owner_properties: Mapping::default(),
                property_count: 0,
                compliance_registry: None,
                owner: caller,
            }
        }

        /// Creates a new PropertyRegistry contract with compliance registry
        #[ink(constructor)]
        pub fn new_with_compliance(compliance_registry: AccountId) -> Self {
            let caller = Self::env().caller();
            Self {
                properties: Mapping::default(),
                owner_properties: Mapping::default(),
                property_count: 0,
                compliance_registry: Some(compliance_registry),
                owner: caller,
            }
        }

        /// Set or update the compliance registry address (owner only)
        #[ink(message)]
        pub fn set_compliance_registry(&mut self, compliance_registry: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }
            self.compliance_registry = Some(compliance_registry);
            Ok(())
        }

        /// Get the compliance registry address
        #[ink(message)]
        pub fn get_compliance_registry(&self) -> Option<AccountId> {
            self.compliance_registry
        }

        /// Check if an account is compliant (internal helper)
        fn check_compliance(&self, account: AccountId) -> Result<(), Error> {
            if let Some(compliance_addr) = self.compliance_registry {
                // Build cross-contract call to ComplianceRegistry::is_compliant
                // Using is_compliant which returns bool (simpler than require_compliance)
                let selector = ink::selector_bytes!("is_compliant");
                
                let is_compliant: bool = ink::env::call::build_call::<ink::env::DefaultEnvironment>()
                    .call(compliance_addr)
                    .exec_input(
                        ink::env::call::ExecutionInput::new(
                            ink::env::call::Selector::new(selector)
                        ).push_arg(account)
                    )
                    .returns::<bool>()
                    .invoke();

                if is_compliant {
                    Ok(())
                } else {
                    Err(Error::NotCompliant)
                }
            } else {
                // No compliance registry set, allow transfer (backward compatibility)
                Ok(())
            }
        }

        /// Registers a new property
        /// Optionally checks compliance if compliance registry is set
        #[ink(message)]
        pub fn register_property(&mut self, metadata: PropertyMetadata) -> Result<u64, Error> {
            let caller = self.env().caller();
            
            // Check compliance for property registration (optional but recommended)
            self.check_compliance(caller)?;
            
            self.property_count += 1;
            let property_id = self.property_count;

            let property_info = PropertyInfo {
                id: property_id,
                owner: caller,
                metadata,
                registered_at: self.env().block_timestamp(),
            };

            self.properties.insert(&property_id, &property_info);

            let mut owner_props = self.owner_properties.get(&caller).unwrap_or_default();
            owner_props.push(property_id);
            self.owner_properties.insert(&caller, &owner_props);

            self.env().emit_event(PropertyRegistered {
                property_id,
                owner: caller,
            });

            Ok(property_id)
        }

        /// Transfers property ownership
        /// Requires recipient to be compliant if compliance registry is set
        #[ink(message)]
        pub fn transfer_property(&mut self, property_id: u64, to: AccountId) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut property = self.properties.get(&property_id).ok_or(Error::PropertyNotFound)?;

            if property.owner != caller {
                return Err(Error::Unauthorized);
            }

            // CRITICAL: Check compliance before allowing transfer
            // This ensures only verified, compliant users can receive properties
            self.check_compliance(to)?;

            // Remove from current owner's properties
            let mut current_owner_props = self.owner_properties.get(&caller).unwrap_or_default();
            current_owner_props.retain(|&id| id != property_id);
            self.owner_properties.insert(&caller, &current_owner_props);

            // Add to new owner's properties
            let mut new_owner_props = self.owner_properties.get(&to).unwrap_or_default();
            new_owner_props.push(property_id);
            self.owner_properties.insert(&to, &new_owner_props);

            // Update property owner
            property.owner = to;
            self.properties.insert(&property_id, &property);

            self.env().emit_event(PropertyTransferred {
                property_id,
                from: caller,
                to,
            });

            Ok(())
        }

        /// Gets property information
        #[ink(message)]
        pub fn get_property(&self, property_id: u64) -> Option<PropertyInfo> {
            self.properties.get(&property_id)
        }

        /// Gets properties owned by an account
        #[ink(message)]
        pub fn get_owner_properties(&self, owner: AccountId) -> Vec<u64> {
            self.owner_properties.get(&owner).unwrap_or_default()
        }

        /// Gets total property count
        #[ink(message)]
        pub fn property_count(&self) -> u64 {
            self.property_count
        }
    }

    impl Default for PropertyRegistry {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests;
