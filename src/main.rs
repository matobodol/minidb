use minidb::{
    application::{AppManager, repl},
    storage::BincodeStorage,
    // storage::JsonStorage,
};

fn main() {
    // let storage = JsonStorage::new("data");
    let storage = BincodeStorage::new("data");
    let mut app = AppManager::new(storage);

    repl::start(&mut app);
}
