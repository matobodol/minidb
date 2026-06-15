// main.rs
use minidb::{
    application::{AppManager, repl},
    storage::BincodeStorage, // or JsonStorage
};

fn main() {
    // let storage = BincodeStorage::with_default();
    let storage = BincodeStorage::with_default();

    let mut app = AppManager::new(storage);

    repl::start(&mut app);
}
