use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::rust::vec::Vec;
use scrypto::*;

blueprint! {
    struct MoveTest {
        bucket: Vec<Bucket>
    }

    impl MoveTest {

        pub fn receive_bucket(&mut self, t: Bucket) {
            info!("Received bucket: address = {}, amount = {}", t.resource(), t.amount());
            self.bucket.push(t);
        }

        pub fn receive_reference(&self, t: BucketRef) {
            info!("Received reference: address = {}, amount = {}", t.resource(), t.amount());
            t.drop();
        }

        pub fn move_bucket() {
            let resource =  create_mutable("m1", Context::package_address());
            let bucket =  mint_resource(resource, 100);
            let component: Component = MoveTest {
                bucket: Vec::new()
            }.instantiate().into();

            component.call::<()>("receive_bucket", args!(bucket));
        }

        pub fn move_reference() -> Bucket {
            let resource =  create_mutable("m2", Context::package_address());
            let bucket =  mint_resource(resource, 100);
            let component: Component = MoveTest {
                bucket: Vec::new()
            }.instantiate().into();

            component.call::<()>("receive_reference", args!(bucket.borrow()));

            // The package still owns the bucket
            bucket
        }
    }
}
