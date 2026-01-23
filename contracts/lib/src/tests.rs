#[cfg(test)]
mod tests {
    use crate::propchain_contracts::PropertyRegistry;
    use crate::propchain_contracts::Error;
    use ink::primitives::AccountId;
    use propchain_traits::*;

    /// Helper function to get default test accounts
    fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
    }

    /// Helper function to set the caller for the next contract call
    fn set_caller(sender: AccountId) {
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
    }

    /// Helper function to create a sample property metadata
    fn create_sample_metadata() -> PropertyMetadata {
        PropertyMetadata {
            location: "123 Main St, City, State 12345".to_string(),
            size: 1000,
            legal_description: "Test property legal description".to_string(),
            valuation: 1000000,
            documents_url: "https://example.com/docs".to_string(),
        }
    }

    /// Helper function to create metadata with custom values
    fn create_custom_metadata(
        location: &str,
        size: u64,
        legal_description: &str,
        valuation: u128,
        documents_url: &str,
    ) -> PropertyMetadata {
        PropertyMetadata {
            location: location.to_string(),
            size,
            legal_description: legal_description.to_string(),
            valuation,
            documents_url: documents_url.to_string(),
        }
    }

    // ============================================================================
    // CORE FUNCTIONALITY TESTS
    // ============================================================================

    #[ink::test]
    fn test_constructor_initializes_correctly() {
        let contract = PropertyRegistry::new();
        assert_eq!(contract.property_count(), 0);
    }

    #[ink::test]
    fn test_register_property_success() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        // Set a block timestamp
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(1000);

        let mut contract = PropertyRegistry::new();
        let metadata = create_sample_metadata();

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property");

        assert_eq!(property_id, 1);
        assert_eq!(contract.property_count(), 1);

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.id, property_id);
        assert_eq!(property.owner, accounts.alice);
        assert_eq!(property.metadata, metadata);
        assert_eq!(property.registered_at, 1000);
    }

    #[ink::test]
    fn test_register_property_increments_counter() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        let property_id_1 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 1");
        assert_eq!(property_id_1, 1);
        assert_eq!(contract.property_count(), 1);

        let property_id_2 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 2");
        assert_eq!(property_id_2, 2);
        assert_eq!(contract.property_count(), 2);
    }

    #[ink::test]
    fn test_register_property_emits_event() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_sample_metadata();

        let property_id = contract
            .register_property(metadata)
            .expect("Failed to register property");

        // Verify that an event was emitted
        let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        assert_eq!(emitted_events.len(), 1, "PropertyRegistered event should be emitted");
    }

    #[ink::test]
    fn test_transfer_property_success() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Transfer to bob
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.owner, accounts.bob);
        assert_eq!(property.id, property_id);
    }

    #[ink::test]
    fn test_transfer_property_updates_owner_lists() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id_1 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 1");
        let property_id_2 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 2");

        // Verify alice owns both properties
        let alice_properties = contract.get_owner_properties(accounts.alice);
        assert_eq!(alice_properties.len(), 2);
        assert!(alice_properties.contains(&property_id_1));
        assert!(alice_properties.contains(&property_id_2));

        // Transfer property 1 to bob
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id_1, accounts.bob)
            .is_ok());

        // Verify alice now only owns property 2
        let alice_properties = contract.get_owner_properties(accounts.alice);
        assert_eq!(alice_properties.len(), 1);
        assert_eq!(alice_properties[0], property_id_2);

        // Verify bob now owns property 1
        let bob_properties = contract.get_owner_properties(accounts.bob);
        assert_eq!(bob_properties.len(), 1);
        assert_eq!(bob_properties[0], property_id_1);
    }

    #[ink::test]
    fn test_transfer_property_emits_event() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());

        // Verify that a transfer event was emitted
        let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        assert!(emitted_events.len() >= 1, "PropertyTransferred event should be emitted");
    }

    #[ink::test]
    fn test_get_property_returns_correct_info() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_custom_metadata(
            "456 Oak Ave",
            2000,
            "Custom legal description",
            2000000,
            "https://ipfs.io/custom",
        );

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.id, property_id);
        assert_eq!(property.owner, accounts.alice);
        assert_eq!(property.metadata.location, "456 Oak Ave");
        assert_eq!(property.metadata.size, 2000);
        assert_eq!(property.metadata.legal_description, "Custom legal description");
        assert_eq!(property.metadata.valuation, 2000000);
        assert_eq!(property.metadata.documents_url, "https://ipfs.io/custom");
    }

    #[ink::test]
    fn test_get_owner_properties_returns_correct_list() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Register multiple properties
        let property_id_1 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 1");
        let property_id_2 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 2");
        let property_id_3 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property 3");

        let alice_properties = contract.get_owner_properties(accounts.alice);
        assert_eq!(alice_properties.len(), 3);
        assert!(alice_properties.contains(&property_id_1));
        assert!(alice_properties.contains(&property_id_2));
        assert!(alice_properties.contains(&property_id_3));
    }

    #[ink::test]
    fn test_get_owner_properties_empty_for_new_owner() {
        let accounts = default_accounts();
        let contract = PropertyRegistry::new();

        let bob_properties = contract.get_owner_properties(accounts.bob);
        assert_eq!(bob_properties.len(), 0);
    }

    #[ink::test]
    fn test_property_count_returns_correct_value() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        assert_eq!(contract.property_count(), 0);

        contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");
        assert_eq!(contract.property_count(), 1);

        contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");
        assert_eq!(contract.property_count(), 2);
    }

    #[ink::test]
    fn test_ownership_verification_after_multiple_transfers() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Transfer alice -> bob
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());
        assert_eq!(
            contract.get_property(property_id).unwrap().owner,
            accounts.bob
        );

        // Transfer bob -> charlie
        set_caller(accounts.bob);
        assert!(contract
            .transfer_property(property_id, accounts.charlie)
            .is_ok());
        assert_eq!(
            contract.get_property(property_id).unwrap().owner,
            accounts.charlie
        );
    }

    #[ink::test]
    fn test_metadata_preserved_after_transfer() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let original_metadata = create_custom_metadata(
            "789 Pine St",
            3000,
            "Original legal description",
            3000000,
            "https://ipfs.io/original",
        );

        let property_id = contract
            .register_property(original_metadata.clone())
            .expect("Failed to register property");

        // Transfer to bob
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());

        // Verify metadata is unchanged
        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata, original_metadata);
    }

    // ============================================================================
    // EDGE CASES
    // ============================================================================

    #[ink::test]
    fn test_register_property_with_max_size() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_custom_metadata(
            "Max size property",
            u64::MAX,
            "Maximum size property",
            u128::MAX,
            "https://ipfs.io/max",
        );

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property with max size");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata.size, u64::MAX);
        assert_eq!(property.metadata.valuation, u128::MAX);
    }

    #[ink::test]
    fn test_register_property_with_zero_values() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_custom_metadata(
            "Zero value property",
            0,
            "Zero size property",
            0,
            "https://ipfs.io/zero",
        );

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property with zero values");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata.size, 0);
        assert_eq!(property.metadata.valuation, 0);
    }

    #[ink::test]
    fn test_register_property_with_empty_strings() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_custom_metadata("", 1000, "", 1000000, "");

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property with empty strings");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata.location, "");
        assert_eq!(property.metadata.legal_description, "");
        assert_eq!(property.metadata.documents_url, "");
    }

    #[ink::test]
    fn test_get_nonexistent_property_returns_none() {
        let contract = PropertyRegistry::new();
        assert_eq!(contract.get_property(0), None);
        assert_eq!(contract.get_property(1), None);
        assert_eq!(contract.get_property(999), None);
        assert_eq!(contract.get_property(u64::MAX), None);
    }

    #[ink::test]
    fn test_transfer_nonexistent_property_fails() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        assert_eq!(
            contract.transfer_property(999, accounts.bob),
            Err(Error::PropertyNotFound)
        );
    }

    #[ink::test]
    fn test_transfer_property_to_self() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Transfer to self
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.alice)
            .is_ok());

        // Property should still be owned by alice
        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.owner, accounts.alice);

        // Alice should still have the property in her list
        let alice_properties = contract.get_owner_properties(accounts.alice);
        assert!(alice_properties.contains(&property_id));
    }

    #[ink::test]
    fn test_property_id_sequence() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Register properties and verify sequential IDs
        for i in 1..=10 {
            let property_id = contract
                .register_property(create_sample_metadata())
                .expect("Failed to register property");
            assert_eq!(property_id, i);
            assert_eq!(contract.property_count(), i);
        }
    }

    // ============================================================================
    // ERROR HANDLING
    // ============================================================================

    #[ink::test]
    fn test_transfer_property_unauthorized_fails() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Try to transfer as charlie (not owner)
        set_caller(accounts.charlie);
        assert_eq!(
            contract.transfer_property(property_id, accounts.bob),
            Err(Error::Unauthorized)
        );

        // Verify ownership unchanged
        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.owner, accounts.alice);
    }

    #[ink::test]
    fn test_transfer_property_after_already_transferred() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Transfer to bob
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());

        // Try to transfer again as alice (no longer owner)
        set_caller(accounts.alice);
        assert_eq!(
            contract.transfer_property(property_id, accounts.charlie),
            Err(Error::Unauthorized)
        );

        // Verify bob still owns it
        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.owner, accounts.bob);
    }

    #[ink::test]
    fn test_transfer_property_invalid_id() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Try to transfer non-existent property
        assert_eq!(
            contract.transfer_property(0, accounts.bob),
            Err(Error::PropertyNotFound)
        );
        assert_eq!(
            contract.transfer_property(1, accounts.bob),
            Err(Error::PropertyNotFound)
        );
        assert_eq!(
            contract.transfer_property(u64::MAX, accounts.bob),
            Err(Error::PropertyNotFound)
        );
    }

    #[ink::test]
    fn test_register_property_with_special_characters() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_custom_metadata(
            "123 Main St, Apt #4-B, City, ST 12345-6789",
            1000,
            "Legal description with \"quotes\" and 'apostrophes'",
            1000000,
            "https://example.com/docs?param=value&other=test",
        );

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property with special characters");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata.location, "123 Main St, Apt #4-B, City, ST 12345-6789");
        assert_eq!(
            property.metadata.legal_description,
            "Legal description with \"quotes\" and 'apostrophes'"
        );
        assert_eq!(
            property.metadata.documents_url,
            "https://example.com/docs?param=value&other=test"
        );
    }

    #[ink::test]
    fn test_register_property_with_unicode_characters() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let metadata = create_custom_metadata(
            "123 Main St, 城市, 州 12345",
            1000,
            "Legal description with émojis 🏠 and unicode 中文",
            1000000,
            "https://example.com/docs",
        );

        let property_id = contract
            .register_property(metadata.clone())
            .expect("Failed to register property with unicode");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata.location, "123 Main St, 城市, 州 12345");
        assert_eq!(
            property.metadata.legal_description,
            "Legal description with émojis 🏠 and unicode 中文"
        );
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[ink::test]
    fn test_bulk_property_registration() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let count = 50;

        // Register multiple properties in bulk
        for i in 1..=count {
            let property_id = contract
                .register_property(create_sample_metadata())
                .expect("Failed to register property");
            assert_eq!(property_id, i);
        }

        assert_eq!(contract.property_count(), count);

        // Verify all properties are accessible
        for i in 1..=count {
            let property = contract.get_property(i);
            assert!(property.is_some());
            let prop = property.unwrap();
            assert_eq!(prop.id, i);
            assert_eq!(prop.owner, accounts.alice);
        }
    }

    #[ink::test]
    fn test_bulk_property_transfer() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let count = 20;

        // Register properties
        let mut property_ids = Vec::new();
        for _ in 0..count {
            let property_id = contract
                .register_property(create_sample_metadata())
                .expect("Failed to register property");
            property_ids.push(property_id);
        }

        // Transfer all to bob
        set_caller(accounts.alice);
        for property_id in &property_ids {
            assert!(contract
                .transfer_property(*property_id, accounts.bob)
                .is_ok());
        }

        // Verify all transferred
        let bob_properties = contract.get_owner_properties(accounts.bob);
        assert_eq!(bob_properties.len(), count);

        for property_id in &property_ids {
            let property = contract.get_property(*property_id).unwrap();
            assert_eq!(property.owner, accounts.bob);
        }
    }

    #[ink::test]
    fn test_get_owner_properties_large_list() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let count = 50;

        // Register many properties for alice
        for _ in 0..count {
            contract
                .register_property(create_sample_metadata())
                .expect("Failed to register property");
        }

        // Get all properties
        let alice_properties = contract.get_owner_properties(accounts.alice);
        assert_eq!(alice_properties.len(), count);

        // Verify all property IDs are unique
        let mut seen = std::collections::HashSet::new();
        for property_id in &alice_properties {
            assert!(!seen.contains(property_id));
            seen.insert(*property_id);
        }
    }

    #[ink::test]
    fn test_property_count_accuracy_under_load() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let count = 100;

        // Register many properties
        for i in 1..=count {
            contract
                .register_property(create_sample_metadata())
                .expect("Failed to register property");
            assert_eq!(contract.property_count(), i);
        }

        assert_eq!(contract.property_count(), count);
    }

    // ============================================================================
    // ADDITIONAL EDGE CASES
    // ============================================================================

    #[ink::test]
    fn test_property_registered_at_timestamp() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Set a known block timestamp
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(1000);

        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.registered_at, 1000);
    }

    #[ink::test]
    fn test_multiple_transfers_same_property() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let property_id = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Transfer multiple times
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());

        set_caller(accounts.bob);
        assert!(contract
            .transfer_property(property_id, accounts.charlie)
            .is_ok());

        set_caller(accounts.charlie);
        assert!(contract
            .transfer_property(property_id, accounts.alice)
            .is_ok());

        // Should be back with alice
        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.owner, accounts.alice);
    }

    #[ink::test]
    fn test_owner_properties_after_transfer_out() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Register multiple properties
        let property_id_1 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");
        let property_id_2 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");
        let property_id_3 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        // Transfer one property out
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id_2, accounts.bob)
            .is_ok());

        // Alice should only have properties 1 and 3
        let alice_properties = contract.get_owner_properties(accounts.alice);
        assert_eq!(alice_properties.len(), 2);
        assert!(alice_properties.contains(&property_id_1));
        assert!(!alice_properties.contains(&property_id_2));
        assert!(alice_properties.contains(&property_id_3));

        // Bob should have property 2
        let bob_properties = contract.get_owner_properties(accounts.bob);
        assert_eq!(bob_properties.len(), 1);
        assert_eq!(bob_properties[0], property_id_2);
    }

    #[ink::test]
    fn test_property_metadata_immutability() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();
        let original_metadata = create_custom_metadata(
            "Original Location",
            1000,
            "Original Description",
            1000000,
            "https://original.com",
        );

        let property_id = contract
            .register_property(original_metadata.clone())
            .expect("Failed to register property");

        // Transfer property
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id, accounts.bob)
            .is_ok());

        // Metadata should remain unchanged
        let property = contract.get_property(property_id).unwrap();
        assert_eq!(property.metadata.location, "Original Location");
        assert_eq!(property.metadata.size, 1000);
        assert_eq!(property.metadata.legal_description, "Original Description");
        assert_eq!(property.metadata.valuation, 1000000);
        assert_eq!(property.metadata.documents_url, "https://original.com");
    }

    #[ink::test]
    fn test_default_implementation() {
        let contract = PropertyRegistry::default();
        assert_eq!(contract.property_count(), 0);
    }

    #[ink::test]
    fn test_property_count_consistency_after_transfers() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Register multiple properties
        let property_id_1 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");
        let property_id_2 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");
        let property_id_3 = contract
            .register_property(create_sample_metadata())
            .expect("Failed to register property");

        assert_eq!(contract.property_count(), 3);

        // Transfer all properties
        set_caller(accounts.alice);
        assert!(contract
            .transfer_property(property_id_1, accounts.bob)
            .is_ok());
        assert!(contract
            .transfer_property(property_id_2, accounts.bob)
            .is_ok());
        assert!(contract
            .transfer_property(property_id_3, accounts.charlie)
            .is_ok());

        // Property count should remain the same
        assert_eq!(contract.property_count(), 3);
    }

    #[ink::test]
    fn test_property_id_uniqueness() {
        let accounts = default_accounts();
        set_caller(accounts.alice);

        let mut contract = PropertyRegistry::new();

        // Register many properties
        let mut property_ids = std::collections::HashSet::new();
        for _ in 0..50 {
            let property_id = contract
                .register_property(create_sample_metadata())
                .expect("Failed to register property");
            assert!(
                property_ids.insert(property_id),
                "Property ID should be unique: {}",
                property_id
            );
        }

        assert_eq!(property_ids.len(), 50);
        assert_eq!(contract.property_count(), 50);
    }
}
