use minidb::database::{
    domain::{Condition, DataType, Value},
    engine::{EngineError, MiniDBEngine},
};

fn main() -> Result<(), EngineError> {
    // build database & table. FIX
    let mut engine = MiniDBEngine::new();
    engine.create_table("users")?;
    engine.create_table("siswa")?;

    // delete table
    engine.drop_table("siswa")?;

    // add column. FIX
    engine.add_columns(
        "users",
        vec![
            ("name", DataType::Str),
            ("age", DataType::Int),
            (
                "state",
                DataType::Enum {
                    variants: vec!["Aktif".into(), "Nonaktif".into()],
                },
            ),
        ],
    )?;

    // insert row
    engine.insert_row(
        "users",
        &[
            ("name", Value::Str("jani".into())),
            ("age", Value::Int(32)),
            (
                "state",
                Value::Enum {
                    value: "Aktif".into(),
                },
            ),
        ],
    )?;
    engine.insert_row(
        "users",
        &[
            ("name", Value::Str("joni".into())),
            ("age", Value::Int(30)),
            (
                "state",
                Value::Enum {
                    value: "Nonaktif".into(),
                },
            ),
        ],
    )?;
    engine.insert_row(
        "users",
        &[
            ("name", Value::Str("jono".into())),
            ("age", Value::Int(29)),
            (
                "state",
                Value::Enum {
                    value: "Aktif".into(),
                },
            ),
        ],
    )?;

    // delete column
    engine.add_columns("users", vec![("Alamat", DataType::Str)])?;
    engine.delete_column("users", "Alamat")?;

    engine.update_row_where(
        "users",
        &[Condition::eq("name", Value::Str("joni".into()))],
        ("age".into(), Value::Int(20)),
    )?;

    // delete row
    engine.delete_row_where("users", &[Condition::eq("name", Value::Str("joni".into()))])?;

    println!("{:#?}", engine);
    Ok(())
}
