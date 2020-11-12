use scrypto::prelude::*;

blueprint! {
    struct FlatAdmin {
        admin_mint_badge: Vault,
        admin_badge: ResourceDef,
    }

    impl FlatAdmin {
        pub fn new(badge_name: String) -> (Component, Bucket) {
        
            // Create a badge for internal use which will hold mint/burn authority for the admin badge we will soon create
            let admin_mint_badge = ResourceBuilder::new().new_badge_fixed(1);

            // Create the ResourceDef for a mutable supply admin badge
            let admin_badge_def = ResourceBuilder::new()
                .metadata("name", badge_name)
                .new_badge_mutable(admin_mint_badge.resource_def());

            // Using our minting authority badge, mint a single admin badge
            let first_admin_badge = admin_badge_def.mint(1, admin_mint_badge.borrow());

            // Initialize our component, placing the minting authority badge within its vault, where it will remain forever
            let component = Self {
                admin_mint_badge: Vault::with_bucket(admin_mint_badge),
                admin_badge: admin_badge_def
            }
            .instantiate();

            // Return the instantiated component and the admin badge we just minted
            (component, first_admin_badge)
        }

        // Any existing admin may create another admin token
        #[auth(admin_badge)]
        pub fn create_additional_admin(&self) -> Bucket {
            // The "authorize" method provides a convenient shortcut to make use of the mint authority badge within our vault without removing it
            self.admin_mint_badge
                .authorize(|badge| self.admin_badge.mint(1, badge))
        }

        pub fn destroy_admin_badge(&self, to_destroy: Bucket) {
            scrypto_assert!(
                to_destroy.resource_def().address() == self.admin_badge.address(),
                "Can not destroy the contents of this bucket!"
            );
            self.admin_mint_badge
                .authorize(|badge| self.admin_badge.burn(to_destroy, badge))
        }

        pub fn get_admin_badge_address(&self) -> Address {
            self.admin_badge.address()
        }
    }
}
