#[cfg(test)]
mod seller_tests {
    use crate::internal::{hash_account_id, hash_offering_id};
    use crate::FixedPriceOffering;
    use crate::FixedPriceOfferingProposal;
    use crate::FixedPriceOfferingStatus::*;
    use crate::FixedPriceOfferingStorageKey;
    use crate::ProposalId;
    use crate::*;
    use crate::{MarketplaceContract, MarketplaceStorageKey};
    use chrono::{DateTime, TimeZone, Utc};
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::collections::{LookupMap, UnorderedSet, Vector};
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, VMContext};

    const MARKETPLACE_ACCOUNT_ID: &str = "place.eneftigo.testnet";
    const NFT_CONTRACT_ID: &str = "0.nft.eneftigo.testnet";
    const NONEXISTENT_NFT_CONTRACT_ID: &str = "nonexistent.eneftigo.testnet";
    const OFFEROR_ACCOUNT_ID: &str = "v-20220601151730-24646460642804";
    const MALICIOUS_ACCOUNT_ID: &str = "malicious.eneftigo.testnet";
    const PROPOSER1_ACCOUNT_ID: &str = "proposer1.eneftigo.testnet";
    const PROPOSER2_ACCOUNT_ID: &str = "proposer2.eneftigo.testnet";

    /*
     * fpo_add_accepting_proposals assertions
     */

    #[test]
    #[should_panic(expected = r#"Price cannot be lower than 1000 yoctoNear"#)]
    fn test_add_buy_now_price_too_low() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);
        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,         // total_supply
            U128(800), // buy_now_price_yocto
            U128(50),  // min_proposal_price_yocto
            // //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Prices must be integer multiple of 10 yoctoNear"#)]
    fn test_add_buy_now_price_not_multiple_of_step() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1115), // buy_now_price_yocto
            U128(50),   // min_proposal_price_yocto
            //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Prices must be integer multiple of 10 yoctoNear"#)]
    fn test_add_min_proposal_price_not_multiple_of_step() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1200), // buy_now_price_yocto
            U128(55),   // min_proposal_price_yocto
            //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Min proposal price must be lower than buy now price"#)]
    fn test_add_buy_now_price_not_higher_than_min_proposal_price() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            U128(1100), // min_proposal_price_yocto
            //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-06-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"End date is into the past"#)]
    fn test_end_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-04-24T00:00:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Start date is into the past"#)]
    fn test_start_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                               // nft_metadata
            Some("1975-04-24T00:00:00+00:00".to_string()), // start_date
            "1975-06-24T00:00:00+00:00".to_string(),       // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Offering duration too short"#)]
    fn test_duration_too_short() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                               // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()), // start_date
            "1975-05-24T13:50:00+00:00".to_string(),       // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Offering duration too long"#)]
    fn test_duration_too_long() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                               // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()), // start_date
            "1975-06-15T13:50:00+00:00".to_string(),       // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_supply_of_zero() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            0,          // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-05-24T13:50:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_supply_too_many() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            101,        // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                         // nft_metadata
            None,                                    // start_date
            "1975-05-24T13:50:00+00:00".to_string(), // end_date
        );
    }
    #[test]
    #[should_panic(
        expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#
    )]
    fn test_wrong_end_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            50,         // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                        // nft_metadata
            None,                                   // start_date
            "1975-5-24T13:50:00+00:00".to_string(), // end_date
        );
    }

    #[test]
    #[should_panic(
        expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#
    )]
    fn test_wrong_start_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_accepting_proposals(
            50,         // total_supply
            U128(1100), // buy_now_price_yocto
            U128(500),  // min_proposal_price_yocto
            //nft_metadata(1),                        // nft_metadata
            Some("1975-05-24".to_string()),         // start_date
            "1975-5-24T13:50:00+00:00".to_string(), // end_date
        );
    }

    // #[test]
    // #[should_panic(expected = r#"Already listed"#)]
    // fn test_already_listed() {
    //     let context = test_get_context(
    //         false,
    //         Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
    //         8380000000000000000000,
    //         0,
    //     );
    //     testing_env!(context);

    //     let mut marketplace = test_marketplace();

    //     marketplace.fpo_add_accepting_proposals(
    //         50,         // total_supply
    //         U128(1100), // buy_now_price_yocto
    //         U128(500),  // min_proposal_price_yocto
    //         //nft_metadata(1),                         // nft_metadata
    //         None,                                    // start_date
    //         "1975-05-24T13:50:00+00:00".to_string(), // end_date
    //     );

    //     marketplace.fpo_add_accepting_proposals(
    //         10,         // total_supply
    //         U128(2000), // buy_now_price_yocto
    //         U128(50),   // min_proposal_price_yocto
    //         //nft_metadata(1),                         // nft_metadata
    //         None,                                    // start_date
    //         "1975-06-24T13:50:00+00:00".to_string(), // end_date
    //     );
    // }

    /*
     * fpo_add_buy_now_only assertions
     */

    #[test]
    #[should_panic(expected = r#"Price cannot be lower than 1000 yoctoNear"#)]
    fn test_buy_now_add_buy_now_price_too_low() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            2,         // total_supply
            U128(800), // buy_now_price_yocto
            //nft_metadata(1), // nft_metadata
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = r#"Price must be integer multiple of 10 yoctoNear"#)]
    fn test_buy_now_add_buy_now_price_not_multiple_of_step() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            2,          // total_supply
            U128(1115), // buy_now_price_yocto
            //nft_metadata(1), // nft_metadata
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = r#"End date is into the past"#)]
    fn test_buy_now_end_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);
        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1),                               // nft_metadata
            None,                                          // start_date
            Some("1975-04-24T00:00:00+00:00".to_string()), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Start date is into the past"#)]
    fn test_buy_now_start_date_into_the_past() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1),                               // nft_metadata
            Some("1975-04-24T00:00:00+00:00".to_string()), // start_date
            None,
        );
    }

    #[test]
    #[should_panic(expected = r#"Offering duration too short"#)]
    fn test_buy_now_duration_too_short() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9570000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            2,          // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1),                               // nft_metadata
            Some("1975-05-24T13:10:00+00:00".to_string()), // start_date
            Some("1975-05-24T13:50:00+00:00".to_string()), // end_date
        );
    }

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_buy_now_supply_of_zero() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            0,          // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1), // nft_metadata
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = r#"Max NFT supply must be between 1 and 100"#)]
    fn test_buy_now_supply_too_many() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            101,        // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1), // nft_metadata
            None,
            None,
        );
    }
    #[test]
    #[should_panic(
        expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#
    )]
    fn test_buy_now_wrong_end_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            50,         // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1),                              // nft_metadata
            None,                                         // start_date
            Some("1975-5-24T13:50:00+00:00".to_string()), // end_date
        );
    }

    #[test]
    #[should_panic(
        expected = r#"Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)"#
    )]
    fn test_buy_now_wrong_start_date_format() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            9490000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            50,         // total_supply
            U128(1100), // buy_now_price_yocto
            //nft_metadata(1),                // nft_metadata
            Some("1975-05-24".to_string()), // start_date
            None,
        );
    }
    // #[test]
    // #[should_panic(expected = r#"Already listed"#)]
    // fn test_buy_now_already_listed() {
    //     let context = test_get_context(
    //         false,
    //         Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
    //         8380000000000000000000,
    //         0,
    //     );
    //     testing_env!(context);

    //     let mut marketplace = test_marketplace();

    //     marketplace.fpo_add_buy_now_only(
    //         50,         // total_supply
    //         U128(1100), // buy_now_price_yocto
    //         //nft_metadata(1), // nft_metadata
    //         None,
    //         None,
    //     );

    //     marketplace.fpo_add_buy_now_only(
    //         10,         // total_supply
    //         U128(2000), // buy_now_price_yocto
    //         //nft_metadata(1), // nft_metadata
    //         None,
    //         None,
    //     );
    // }

    #[test]
    fn test_buy_now_success() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            10450000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();

        marketplace.fpo_add_buy_now_only(
            10,         // total_supply
            U128(1000), // buy_now_price_yocto
            //nft_metadata(1), // nft_metadata
            None,
            None,
        );
    }

    /*
     * proposal_accepting vs buy_now_only assertions
     */

    // #[test]
    // #[should_panic(expected = r#"Already listed"#)]
    // fn test_already_listed_proposal_vs_buy_now() {
    //     let context = test_get_context(
    //         false,
    //         Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
    //         8380000000000000000000,
    //         0,
    //     );
    //     testing_env!(context);

    //     let mut marketplace = test_marketplace();

    //     marketplace.fpo_add_accepting_proposals(
    //         50,         // total_supply
    //         U128(1100), // buy_now_price_yocto
    //         U128(500),  // min_proposal_price_yocto
    //         //nft_metadata(1),                         // nft_metadata
    //         None,                                    // start_date
    //         "1975-05-24T13:50:00+00:00".to_string(), // end_date
    //     );

    //     marketplace.fpo_add_buy_now_only(
    //         10,         // total_supply
    //         U128(2000), // buy_now_price_yocto
    //         //nft_metadata(1), // nft_metadata
    //         None,
    //         None,
    //     );
    // }

    // #[test]
    // #[should_panic(expected = r#"Already listed"#)]
    // fn test_already_listed_buy_now_vs_proposal() {
    //     let context = test_get_context(
    //         false,
    //         Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
    //         8380000000000000000000,
    //         0,
    //     );
    //     testing_env!(context);

    //     let mut marketplace = test_marketplace();

    //     marketplace.fpo_add_buy_now_only(
    //         10,         // total_supply
    //         U128(2000), // buy_now_price_yocto
    //         //nft_metadata(1), // nft_metadata
    //         None,
    //         None,
    //     );

    //     marketplace.fpo_add_accepting_proposals(
    //         50,         // total_supply
    //         U128(1100), // buy_now_price_yocto
    //         U128(500),  // min_proposal_price_yocto
    //         //nft_metadata(1),                         // nft_metadata
    //         None,                                    // start_date
    //         "1975-05-24T13:50:00+00:00".to_string(), // end_date
    //     );
    // }

    /*
     * fpo_accept_proposals
     */

    #[test]
    #[should_panic(expected = r#"Only the offeror can accept proposals"#)]
    fn test_accepting_proposals_by_unauthorized_user() {
        let context = test_get_context(
            true,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_accept_proposals(nft_contract_id.clone(), collection_id, 1);
    }

    #[test]
    #[should_panic(expected = r#"Could not find NFT listing"#)]
    fn test_accepting_proposals_for_nonexistent_offering() {
        let context = test_get_context(
            true,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        let collection_id: CollectionId = 0;

        marketplace.fpo_accept_proposals(
            AccountId::new_unchecked(NONEXISTENT_NFT_CONTRACT_ID.to_string()),
            collection_id,
            1,
        );
    }
    #[test]
    #[should_panic(expected = r#"There's not enough proposals (3)"#)]
    fn test_accepting_too_many_proposals() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_accept_proposals(nft_contract_id.clone(), collection_id, 4);
    }

    #[test]
    fn test_accepting_proposals_batches() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_accept_proposals(nft_contract_id.clone(), collection_id, 1);

        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        assert!(
            fpo.acceptable_proposals.len() == 2,
            "Wrong number of acceptable_proposals"
        );
        assert!(
            fpo.acceptable_proposals.to_vec() == vec![1, 3],
            "acceptable_proposals content incorrect"
        );
        assert!(
            fpo.proposals.get(&1).is_some()
                && fpo.proposals.get(&2).is_none()
                && fpo.proposals.get(&3).is_some(),
            "proposals content incorrect"
        );
        let proposals_by_proposer1 = fpo
            .proposals_by_proposer
            .get(&AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string()))
            .unwrap();
        assert!(
            proposals_by_proposer1.contains(&1),
            "proposals_by_proposer incorrect for proposer1"
        );
        let proposals_by_proposer2 = fpo
            .proposals_by_proposer
            .get(&AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string()))
            .unwrap();
        assert!(
            !proposals_by_proposer2.contains(&2) && proposals_by_proposer2.contains(&3),
            "proposals_by_proposer incorrect for proposer2"
        );
        assert!(fpo.supply_left == 2, "Wrong supply_left");

        marketplace.fpo_accept_proposals(nft_contract_id.clone(), collection_id, 2);
        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");
        assert!(
            fpo.acceptable_proposals.is_empty(),
            "Wrong number of acceptable_proposals"
        );
        assert!(
            fpo.proposals.get(&1).is_none()
                && fpo.proposals.get(&2).is_none()
                && fpo.proposals.get(&3).is_none(),
            "proposals content incorrect"
        );
        let proposals_by_proposer1 = fpo
            .proposals_by_proposer
            .get(&AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string()));
        assert!(
            proposals_by_proposer1.is_none(),
            "proposals_by_proposer exist for proposer1"
        );
        let proposals_by_proposer2 = fpo
            .proposals_by_proposer
            .get(&AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string()));
        assert!(
            proposals_by_proposer2.is_none(),
            "proposals_by_proposer exist for proposer2"
        );
        assert!(fpo.supply_left == 0, "Wrong supply_left");
    }

    #[test]
    fn test_accepting_proposals_all_at_once() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_accept_proposals(nft_contract_id.clone(), collection_id, 3);

        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };

        let fpo = marketplace
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not get updated FPO");

        assert!(
            fpo.acceptable_proposals.is_empty(),
            "Some acceptable_proposals are left"
        );
        assert!(
            fpo.proposals.get(&1).is_none()
                && fpo.proposals.get(&2).is_none()
                && fpo.proposals.get(&3).is_none(),
            "proposals content incorrect"
        );
        let proposals_by_proposer1 = fpo
            .proposals_by_proposer
            .get(&AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string()));
        assert!(
            proposals_by_proposer1.is_none(),
            "proposals_by_proposer left for proposer1"
        );
        let proposals_by_proposer2 = fpo
            .proposals_by_proposer
            .get(&AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string()));
        assert!(
            proposals_by_proposer2.is_none(),
            "proposals_by_proposer left for proposer2"
        );
        assert!(fpo.supply_left == 0, "Some supply_left");
    }

    /*
     * fpo_conclude
     */

    #[test]
    #[should_panic(expected = r#"Only the offeror can conclude"#)]
    fn test_conclude_by_unauthorized_user() {
        let context = test_get_context(
            true,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_account_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        marketplace.fpo_conclude(nft_account_id.clone(), collection_id);
    }

    #[test]
    #[should_panic(expected = r#"Could not find NFT listing"#)]
    fn test_conclude_nonexistent_offering() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(13, 10, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);

        marketplace.fpo_conclude(
            AccountId::new_unchecked(NONEXISTENT_NFT_CONTRACT_ID.to_string()),
            0,
        );
    }

    #[test]
    fn test_conclude_before_start_date() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 5, 24).and_hms(00, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        marketplace.fpo_conclude(nft_contract_id.clone(), collection_id);

        assert!(
            marketplace.fpos_by_id.is_empty(),
            "fpos_by_contract_id not empty"
        );
        assert!(
            marketplace
                .fpos_by_offeror_id
                .get(&AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()))
                .is_none(),
            "fpos_by_contract_id not empty"
        );
    }

    #[test]
    fn test_conclude_after_end_date() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 7, 24).and_hms(00, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        marketplace.fpo_conclude(nft_contract_id.clone(), collection_id);

        assert!(
            marketplace.fpos_by_id.is_empty(),
            "fpos_by_contract_id not empty"
        );
        assert!(
            marketplace
                .fpos_by_offeror_id
                .get(&AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string()))
                .is_none(),
            "fpos_by_contract_id not empty"
        );
    }

    #[test]
    #[should_panic(expected = r#"Cannot conclude an offering while it's running"#)]
    fn test_conclude_while_running_and_supply_left() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let mut fpo = test_fpo(3);
        test_place_proposals(&mut fpo);
        test_add_fpo(&mut marketplace, &fpo);
        marketplace.fpo_conclude(nft_contract_id.clone(), collection_id);
    }

    #[test]
    fn test_conclude_while_running_and_supply_zero() {
        let context = test_get_context(
            false,
            Utc.ymd(1975, 6, 1).and_hms(00, 00, 00),
            8380000000000000000000,
            0,
        );
        testing_env!(context);

        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;

        let mut marketplace = test_marketplace();
        let fpo = test_fpo(0);
        test_add_fpo(&mut marketplace, &fpo);
        marketplace.fpo_conclude(nft_contract_id.clone(), collection_id);
    }
    /* Helpers */

    fn test_get_context(
        malicious: bool,
        datetime: DateTime<Utc>,
        attached_deposit: u128,
        storage_usage: u64,
    ) -> VMContext {
        let account_id = if malicious {
            AccountId::new_unchecked(MALICIOUS_ACCOUNT_ID.to_string())
        } else {
            AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string())
        };
        VMContextBuilder::new()
            .current_account_id(AccountId::new_unchecked(MARKETPLACE_ACCOUNT_ID.to_string()))
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .block_timestamp(datetime.timestamp_nanos() as u64)
            .attached_deposit(attached_deposit)
            .storage_usage(storage_usage)
            .build()
    }

    fn test_marketplace() -> MarketplaceContract {
        MarketplaceContract::new(AccountId::new_unchecked(MARKETPLACE_ACCOUNT_ID.to_string()))
    }

    fn test_add_fpo(marketplace: &mut MarketplaceContract, fpo: &FixedPriceOffering) {
        marketplace.fpos_by_id.insert(&fpo.offering_id, fpo);
        let mut fpos_by_this_offeror = UnorderedSet::new(
            MarketplaceStorageKey::FposByOfferorIdInner {
                account_id_hash: hash_account_id(&fpo.offeror_id),
            }
            .try_to_vec()
            .unwrap(),
        );
        fpos_by_this_offeror.insert(&fpo.offering_id.clone());
        marketplace
            .fpos_by_offeror_id
            .insert(&fpo.offeror_id, &fpos_by_this_offeror);
    }

    fn test_fpo(supply: u64) -> FixedPriceOffering {
        let nft_contract_id = AccountId::new_unchecked(NFT_CONTRACT_ID.to_string());
        let collection_id: CollectionId = 0;
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        let offering_id_hash = hash_offering_id(&offering_id);
        // let nft_account_id_hash = hash_account_id(&nft_account_id);
        let offeror_account_id = AccountId::new_unchecked(OFFEROR_ACCOUNT_ID.to_string());
        let start_timestamp = DateTime::parse_from_rfc3339("1975-05-26T00:00:00+00:00")
            .unwrap()
            .timestamp_nanos();
        let end_timestamp = DateTime::parse_from_rfc3339("1975-06-10T00:00:00+00:00")
            .unwrap()
            .timestamp_nanos();
        let fpo = FixedPriceOffering {
            offering_id,
            offeror_id: offeror_account_id.clone(),
            supply_total: supply,
            buy_now_price_yocto: 1000,
            min_proposal_price_yocto: Some(500),
            start_timestamp: Some(start_timestamp),
            end_timestamp: Some(end_timestamp),
            status: Unstarted,
            //            nft_metadata: nft_metadata(1),
            supply_left: supply,
            proposals: LookupMap::new(
                FixedPriceOfferingStorageKey::Proposals { offering_id_hash }
                    .try_to_vec()
                    .unwrap(),
            ),
            proposals_by_proposer: LookupMap::new(
                FixedPriceOfferingStorageKey::ProposalsByProposer { offering_id_hash }
                    .try_to_vec()
                    .unwrap(),
            ),
            acceptable_proposals: Vector::new(
                FixedPriceOfferingStorageKey::AcceptableProposals { offering_id_hash }
                    .try_to_vec()
                    .unwrap(),
            ),
            next_proposal_id: 0,
        };
        fpo
    }

    fn test_place_proposals(fpo: &mut FixedPriceOffering) {
        let proposer1_id = AccountId::new_unchecked(PROPOSER1_ACCOUNT_ID.to_string());
        let proposer2_id = AccountId::new_unchecked(PROPOSER2_ACCOUNT_ID.to_string());
        let proposals_vec: Vec<FixedPriceOfferingProposal> = vec![
            FixedPriceOfferingProposal {
                id: 1,
                proposer_id: proposer1_id.clone(),
                price_yocto: 500,
                is_acceptable: true,
            },
            FixedPriceOfferingProposal {
                id: 2,
                proposer_id: proposer2_id.clone(),
                price_yocto: 900,
                is_acceptable: true,
            },
            FixedPriceOfferingProposal {
                id: 3,
                proposer_id: proposer2_id.clone(),
                price_yocto: 700,
                is_acceptable: true,
            },
        ];
        for proposal in proposals_vec.iter() {
            fpo.proposals.insert(&proposal.id, &proposal);
        }
        fpo.acceptable_proposals.extend(vec![1, 3, 2]);

        let proposer1_id_hash = hash_account_id(&proposer1_id);
        let offering_id_hash = hash_offering_id(&fpo.offering_id);
        let mut proposals_by_proposer1: UnorderedSet<ProposalId> = UnorderedSet::new(
            FixedPriceOfferingStorageKey::ProposalsByProposerInner {
                offering_id_hash,
                proposer_id_hash: proposer1_id_hash,
            }
            .try_to_vec()
            .unwrap(),
        );
        proposals_by_proposer1.extend(vec![1]);

        let proposer2_id_hash = hash_account_id(&proposer2_id);
        let mut proposals_by_proposer2: UnorderedSet<ProposalId> = UnorderedSet::new(
            FixedPriceOfferingStorageKey::ProposalsByProposerInner {
                offering_id_hash,
                proposer_id_hash: proposer2_id_hash,
            }
            .try_to_vec()
            .unwrap(),
        );
        proposals_by_proposer2.extend(vec![2, 3]);

        fpo.proposals_by_proposer
            .insert(&proposer1_id, &proposals_by_proposer1);
        fpo.proposals_by_proposer
            .insert(&proposer2_id, &proposals_by_proposer2);
    }

    // fn nft_metadata(index: i32) -> TokenMetadata {
    //     TokenMetadata {
    //         title: Some(format!("nft{}", index)),
    //         description: None,
    //         media: None,
    //         media_hash: None,
    //         copies: Some(1),
    //         issued_at: None,
    //         expires_at: None,
    //         starts_at: None,
    //         updated_at: None,
    //         extra: None,
    //         reference: None,
    //         reference_hash: None,
    //     }
    // }
}
