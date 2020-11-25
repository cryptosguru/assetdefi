# Hello, NFT!

From Wikipedia,

> A non-fungible token (NFT) is a unique and non-interchangeable unit of data stored on a digital ledger (blockchain). NFTs can be associated with easily-reproducible items such as photos, videos, audio, and other types of digital files as unique items

In this example, we will show you how to create, mint, transfer and update NFTs in Scrypto.

## How to Create NFT?

NFT is just another type of resource in Scrypto, and the way to define NFTs is through `ResourceBuilder`.

To create immutable NFTs, we will need to define the structure first and then provide the initial, fixed set of NFTs, like
```rust
#[derive(TypeId, Encode, Decode, Describe)]
pub struct MagicCard {
    color: Color,
    class: Class,
    rarity: Rarity,
}

let special_cards_bucket = ResourceBuilder::new()
    .metadata("name", "Russ' Magic Card Collection")
    .new_nft_fixed(BTreeMap::from([
        (
            1, // The ID of the first NFT, you can also use `Uuid::generate()` to create a random ID
            MagicCard {
                color: Color::Black,
                class: Class::Sorcery,
                rarity: Rarity::MythicRare,
            },
        ),
        (
            2, // The ID of the second NFT
            MagicCard {
                color: Color::Green,
                class: Class::Planeswalker,
                rarity: Rarity::Rare,
            },
        )
    ]));
```

To create mutable NFTs, no initial supply is required but resource authorization configuration is required.

```rust
let random_card_mint_badge = ResourceBuilder::new()
    .metadata("name", "Random Cards Mint Badge")
    .new_badge_fixed(1);
let random_card_resource_def = ResourceBuilder::new()
    .metadata("name", "Random Cards")
    .new_nft_mutable(
        ResourceAuthConfigs::new(random_card_mint_badge.resource_address())
    );
```

Here, we're using the mint badge for both minting, burning and updating. If you want, you can also specify different badge for each permission, like,

```rust
ResourceAuthConfigs::new(address_a).with_update_badge_address(address_b)
```

To further mint NFTs, we can use the `mint_nft` method:
```rust
let nft = self.random_card_mint_badge.authorize(|auth| {
    self.random_card_resource_def.mint_nft(
        // The NFT id
        self.random_card_id_counter,
        // The NFT data
        MagicCard { 
            color: Self::random_color(random_seed),
            class: Self::random_class(random_seed),
            rarity: Self::random_rarity(random_seed),
        },
        // authorization to mint
        auth
    )
});
```

## Transfer to Another Account/Component

Since NFT is just another type if resource, it must be stored in either a bucket and vault. To transfer one NFT to another account, we will need to withdraw it from the sender's account and deposit into the recipient's account.

To pick a specific NFT when calling a function or method, we can use the following syntax:

```
#nft_id_1,#nft_id2,resource_address
```

## Update an Existing NFT


To update, one needs to call the `update_nft_data` method on resource definition.

```rust
let nft = self.random_card_mint_badge.authorize(|auth| {
    self.random_card_resource_def.update_nft_data(
        // The NFT id
        self.random_card_id_counter,
        // The NFT data
        MagicCard { 
            color: Self::random_color(random_seed),
            class: Self::random_class(random_seed),
            rarity: Self::random_rarity(random_seed),
        },
        // authorization to update
        auth
    )
});
```

Only the NFT update badge owners can update an NFT. 

## How to Play?

1. Create a new account, and save the account address
```
resim new-account
```
2. Publish the package, and save the package address
```
resim publish .
```
3. Call the `new` function to instantiate a component, and save the component address
```
resim call-function <PACKAGE_ADDRESS> HelloNft new
```
4. Call the `buy_random_card` method of the component we just instantiated
```
resim call-method <COMPONENT_ADDRESS> buy_random_card "1000,030000000000000000000000000000000000000000000000000004"
```
4. Call the `buy_random_card` method again
```
resim call-method <COMPONENT_ADDRESS> buy_random_card "1000,030000000000000000000000000000000000000000000000000004"
```
5. Check out our balance
```
resim show <ACCOUNT_ADDRESS>
```
6. Fuse our random cards
```
resim call-method <COMPONENT_ADDRESS> fuse_my_cards "#0,#1,03d8541671ab09116ae450d468f91e5488a9b22c705d70dcfe9e09"
```
7. Check out our balance again and we should see a upgraded card
```
resim show <ACCOUNT_ADDRESS>
```