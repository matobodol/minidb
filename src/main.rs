use minidb::{
    application::{AppManager, repl},
    storage::FileStorage,
};

fn main() {
    let storage_path = FileStorage::new("data"); // sesuaikan
    let mut app = AppManager::new(storage_path);

    repl::start(&mut app);
}
