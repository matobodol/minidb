use minidb::{
    application::{AppManager, run_repl},
    storage::FileDatabaseStorage,
};

fn main() {
    let storage = FileDatabaseStorage::new("./data");
    let mut app = AppManager::new(storage);
    println!("{:#?}", app);

    run_repl(&mut app);
}

// exit ✅️
// contoh: exit | quit | /q

// create database✅️
// contoh: create database mydb

// use database✅️
// contoh: use database mydb | use mydb

// show databases✅️
// contoh: show databases

// show current database✅️
// contoh: show current

// drop database✅️
// contoh: drop database mydb

// CREATE TABLE users;✅️
// DROP TABLE users;✅️
// SHOW TABLES;✅️
// DESCRIBE users; -- alias DESC users;✅️

// ALTER TABLE users ADD COLUMN name str primary key not null, age int✅️
// ALTER TABLE users DROP COLUMN name;✅️

// insert into users (name,age) values ("jono", 30)✅️
// update users set age = 21 wherw name = "jono"✅️
// delete from users where name = "jojon"✅️
