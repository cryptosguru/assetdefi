use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[derive(TypeId, Encode, Decode)]
pub enum Class {
    Land,
    Creature,
    Artifact,
    Enchantment,
    Planeswalker,
    Sorcery,
    Instant,
}

#[derive(TypeId, Encode, Decode)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    MythicRare,
}

#[derive(TypeId, Encode, Decode)]
pub struct MagicCard {
    color: Color,
    class: Class,
    rarity: Rarity,
}

// FIXME Bucket::take_all() is broken because of NFT

// TODO How to change an NFT? Who is authorized to do so?

// TODO Display NFTs when `resim show`

// TODO Support passing NFT to a component

blueprint! {
    struct HelloNft {
        /// A vault that holds all our special cards
        special_cards: Vault,
        /// The price for each special card
        special_card_prices: HashMap<u64, Decimal>,
        /// A vault that holds the minter badge
        random_card_minter: Vault,
        /// The resource definition of all random cards
        random_card_resource_def: ResourceDef,
        /// The price of each random card
        random_card_price: Decimal,
        /// A vault that collects all XRD payments
        collected_xrd: Vault,
    }

    impl HelloNft {
        pub fn new() -> Component {
            // Creates a fixed set of NFTs
            let special_cards_bucket = ResourceBuilder::new()
                .metadata("name", "Russ' Magic Card Collection")
                .new_nft_fixed(BTreeMap::from([
                    (
                        1,
                        MagicCard {
                            color: Color::Black,
                            class: Class::Sorcery,
                            rarity: Rarity::MythicRare,
                        },
                    ),
                    (
                        2,
                        MagicCard {
                            color: Color::Green,
                            class: Class::Planeswalker,
                            rarity: Rarity::Rare,
                        },
                    ),
                    (
                        3,
                        MagicCard {
                            color: Color::Red,
                            class: Class::Creature,
                            rarity: Rarity::Uncommon,
                        },
                    ),
                ]));

            // Create an NFT resource with mutable supply
            let random_card_minter_badge = ResourceBuilder::new()
                .metadata("name", "Random Cards Minter Badge")
                .new_badge_fixed(1);
            let random_card_resource_def = ResourceBuilder::new()
                .metadata("name", "Random Cards")
                .new_nft_mutable(random_card_minter_badge.resource_def());

            // Instantiate our component
            Self {
                special_cards: Vault::with_bucket(special_cards_bucket),
                special_card_prices: HashMap::from([
                    (1, 500.into()),
                    (2, 666.into()),
                    (3, 123.into()),
                ]),
                random_card_minter: Vault::with_bucket(random_card_minter_badge),
                random_card_resource_def,
                random_card_price: 50.into(),
                collected_xrd: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
        }

        pub fn buy_special_card(&mut self, id: u64, payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            let price = self.special_card_prices.remove(&id).unwrap();
            self.collected_xrd.put(payment.take(price));

            // Take the requested NFT
            let nft = self.special_cards.take_nft(id);

            // Return the NFT and change
            (nft, payment)
        }

        pub fn buy_random_card(&mut self, payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            self.collected_xrd.put(payment.take(self.random_card_price));

            // Mint a new card
            let random_seed = 100; // TODO: obtain from oracle
            let new_card = MagicCard {
                color: Self::random_color(random_seed),
                class: Self::random_class(random_seed),
                rarity: Self::random_rarity(random_seed),
            };
            let nft_id: u64 = self.random_card_resource_def.supply().to_string().parse().unwrap();
            let nft = self.random_card_minter.authorize(|auth| {
                self.random_card_resource_def
                    .mint_nft(nft_id, new_card, auth)
            });

            // Return the NFT and change
            (nft, payment)
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

        fn random_class(seed: u64) -> Class {
            match seed % 7 {
                0 => Class::Land,
                1 => Class::Creature,
                2 => Class::Artifact,
                3 => Class::Enchantment,
                4 => Class::Planeswalker,
                5 => Class::Sorcery,
                6 => Class::Instant,
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
