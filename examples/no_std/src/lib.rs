// Disable linking to std.
#![cfg_attr(not(test), no_std)]
// Use default alloc error handler, i.e. to panic, and enable core intrinsics.
#![cfg_attr(not(test), feature(default_alloc_error_handler, core_intrinsics))]

// Abort when panicking.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

// Use WeeAlloc as our global heap allocator.
#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//============================
// Scrypto code starts here //
//============================

use scrypto::prelude::*;

blueprint! {
    struct Greeting {
        count: u32
    }

    impl Greeting {
        pub fn new() -> Address {
            Self {
                count: 0
            }
            .instantiate()
        }

        pub fn say_hello(&mut self) -> u32 {
            info!("Hello, visitor #{}.", self.count);
            self.count += 1;
            self.count
        }
    }
}