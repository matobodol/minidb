use minidb::{
    application::AppManager,
    domain::{Constraint, DataType, Value},
};

fn main() {
    let mut app_manager = AppManager::new();
    app_manager.create("app_db").unwrap();
    app_manager.db_use("app_db");

    let db = app_manager.db_mut().unwrap();
    db.create_table("users").unwrap();
    db.add_columns(
        "users",
        vec![
            (
                "name",
                DataType::Str,
                &[Constraint::Unique, Constraint::NotNull],
            ),
            (
                "age",
                DataType::Int,
                &[Constraint::NotNull, Constraint::Default(Value::Int(100))],
            ),
        ],
    )
    .unwrap();

    db.insert_row(
        "users",
        &[("age", Value::Int(32)), ("name", Value::Str("joni".into()))],
    )
    .unwrap();

    db.insert_row(
        "users",
        &[("age", Value::Int(20)), ("name", Value::Str("jono".into()))],
    )
    .unwrap();

    db.insert_row(
        "users",
        &[("age", Value::Int(20)), ("name", Value::Str("jani".into()))],
    )
    .unwrap();

    // db.delete_column("users", "age").unwrap();

    println!("{:#?}", &app_manager);
}
