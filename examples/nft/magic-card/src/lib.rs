use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    MythicRare,
}

#[derive(NftData)]
pub struct MagicCard {
    color: Color,
    rarity: Rarity,
    #[scrypto(mutable)]
    level: u8,
}

blueprint! {
    struct HelloNft {
        /// A vault that holds all our special cards
        special_cards: Vault,
        /// The price for each special card
        special_card_prices: HashMap<u128, Decimal>,
        /// A vault that holds the mint badge
        random_card_mint_badge: Vault,
        /// The resource definition of all random cards
        random_card_resource_def: ResourceDef,
        /// The price of each random card
        random_card_price: Decimal,
        /// A counter for ID generation
        random_card_id_counter: u128,
        /// A vault that collects all XRD payments
        collected_xrd: Vault,
    }

    impl HelloNft {
        pub fn new() -> Component {
            // Creates a fixed set of NFTs
            let special_cards_bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "Russ' Magic Card Collection")
                .initial_supply_non_fungible([
                    (
                        1,
                        MagicCard {
                            color: Color::Black,
                            rarity: Rarity::MythicRare,
                            level: 3,
                        },
                    ),
                    (
                        2,
                        MagicCard {
                            color: Color::Green,
                            rarity: Rarity::Rare,
                            level: 5,
                        },
                    ),
                    (
                        3,
                        MagicCard {
                            color: Color::Red,
                            rarity: Rarity::Uncommon,
                            level: 100,
                        },
                    ),
                ]);

            // Create an NFT resource with mutable supply
            let random_card_mint_badge = ResourceBuilder::new_fungible(18)
                .metadata("name", "Random Cards Mint Badge")
                .initial_supply_fungible(1);
            let random_card_resource_def = ResourceBuilder::new_non_fungible()
                .metadata("name", "Random Cards")
                .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(
                    random_card_mint_badge.resource_def(),
                    MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA,
                )
                .no_initial_supply();

            // Instantiate our component
            Self {
                special_cards: Vault::with_bucket(special_cards_bucket),
                special_card_prices: HashMap::from([
                    (1, 500.into()),
                    (2, 666.into()),
                    (3, 123.into()),
                ]),
                random_card_mint_badge: Vault::with_bucket(random_card_mint_badge),
                random_card_resource_def,
                random_card_price: 50.into(),
                random_card_id_counter: 0,
                collected_xrd: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
        }

        pub fn buy_special_card(&mut self, id: u128, payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            let price = self.special_card_prices.remove(&id).unwrap();
            self.collected_xrd.put(payment.take(price));

            // Take the requested NFT
            let nft_bucket = self.special_cards.take_nft(id);

            // Return the NFT and change
            (nft_bucket, payment)
        }

        pub fn buy_random_card(&mut self, payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            self.collected_xrd.put(payment.take(self.random_card_price));

            // Mint a new card
            let random_seed = 100; // TODO: obtain from oracle
            let new_card = MagicCard {
                color: Self::random_color(random_seed),
                rarity: Self::random_rarity(random_seed),
                level: random_seed as u8 % 8,
            };
            let nft_bucket = self.random_card_mint_badge.authorize(|auth| {
                self.random_card_resource_def
                    .mint_nft(self.random_card_id_counter, new_card, auth)
            });
            self.random_card_id_counter += 1;

            // Return the NFT and change
            (nft_bucket, payment)
        }

        pub fn upgrade_my_card(&mut self, nft_bucket: Bucket) -> Bucket {
            assert!(
                nft_bucket.amount() == 1.into(),
                "We can upgrade only one card each time"
            );

            let nft_id = nft_bucket.get_nft_ids()[0];

            // Get and update the mutable data
            let mut nft_data: MagicCard = nft_bucket.get_nft_data(nft_id);
            nft_data.level += 1;

            self.random_card_mint_badge
                .authorize(|auth| nft_bucket.update_nft_data(nft_id, nft_data, auth));

            nft_bucket
        }

        pub fn fuse_my_cards(&mut self, nft_bucket: Bucket) -> Bucket {
            assert!(
                nft_bucket.amount() == 2.into(),
                "You need to pass 2 NFTs for fusion"
            );
            assert!(
                nft_bucket.resource_def() == self.random_card_resource_def,
                "Only random cards can be fused"
            );

            // Get the NFT IDs
            let nft_ids = nft_bucket.get_nft_ids();

            // Retrieve the NFT data.
            let card1: MagicCard = nft_bucket.get_nft_data(nft_ids[0]);
            let card2: MagicCard = nft_bucket.get_nft_data(nft_ids[1]);
            let new_card = Self::fuse_magic_cards(card1, card2);

            // Burn the original cards
            self.random_card_mint_badge.authorize(|auth| {
                nft_bucket.burn(Some(auth));
            });

            // Mint a new one.
            let new_nft_bucket = self.random_card_mint_badge.authorize(|auth| {
                self.random_card_resource_def
                    .mint_nft(self.random_card_id_counter, new_card, auth)
            });
            self.random_card_id_counter += 1;

            new_nft_bucket
        }

        fn fuse_magic_cards(card1: MagicCard, card2: MagicCard) -> MagicCard {
            MagicCard {
                color: card1.color,
                rarity: card2.rarity,
                level: card1.level + card2.level,
            }
        }

        fn random_color(seed: u64) -> Color {
            match seed % 5 {
                0 => Color::White,
                1 => Color::Blue,
                2 => Color::Black,
                3 => Color::Red,
                4 => Color::Green,
                _ => panic!(),
            }
        }

        fn random_rarity(seed: u64) -> Rarity {
            match seed % 4 {
                0 => Rarity::Common,
                1 => Rarity::Uncommon,
                2 => Rarity::Rare,
                3 => Rarity::MythicRare,
                _ => panic!(),
            }
        }
    }
}
