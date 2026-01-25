use minidb::database::{
    domain::{DataType, Value},
    engine::{DbEngine, EngineError},
};

fn main() -> Result<(), EngineError> {
    let mut engine = DbEngine::new();
    engine.create_table("users")?;
    engine.create_table("siswa")?;
    engine.drop_table("siswa")?;
    engine.add_columns(
        "users",
        vec![
            ("name", DataType::Str),
            ("age", DataType::Int),
            (
                "state".into(),
                DataType::Enum {
                    variants: vec!["Aktif".into(), "Nonaktif".into()],
                },
            ),
        ],
    )?;

    engine.insert_row(
        "users",
        vec![
            Value::Str("jani".into()),
            Value::Int(34),
            Value::Enum {
                value: "Aktif".into(),
            },
        ],
    )?;
    engine.insert_row(
        "users",
        vec![
            Value::Str("joni".into()),
            Value::Int(32),
            Value::Enum {
                value: "Nonaktif".into(),
            },
        ],
    )?;

    // engine.delete_row("users", "name", &Value::Str("joni".into()))?;
    engine.add_columns("users", vec![("Alamat".into(), DataType::Str)])?;

    engine.update_where(
        "users",
        "name",
        &Value::Str("jani".into()),
        "Alamat",
        Value::Str("is modifyed".into()),
    )?;

    println!("{:#?}", engine);
    Ok(())
}
