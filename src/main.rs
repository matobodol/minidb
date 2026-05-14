use minidb::{
    application::{AppManager, repl},
    storage::FileStorage,
};

fn main() {
    let storage = FileStorage::new("data");
    let mut app = AppManager::new(storage);

    repl::start(&mut app);
}
