use scrypto::blueprint;
use scrypto::resource::{Bucket, ResourceBuilder, ResourceDef};
use scrypto::rust::collections::*;
use scrypto::types::{Address, Amount};

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_mutable() -> (Bucket, ResourceDef) {
            let auth = ResourceBuilder::new()
                .metadata("name", "Auth")
                .create_fixed(1);
            let resource_def = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_mutable(auth.resource_def());
            (auth, resource_def)
        }

        pub fn create_fixed() -> Bucket {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);

            bucket
        }

        pub fn query() -> (HashMap<String, String>, Option<Address>, Amount) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);
            let resource_def = bucket.resource_def();
            bucket.burn();
            (resource_def.metadata(), resource_def.mint_auth(), resource_def.supply())
        }

        pub fn burn() {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(100);
            bucket.burn();
        }
    }
}
