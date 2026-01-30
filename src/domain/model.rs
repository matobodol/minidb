use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Int,
    Str,
    Float,
    // Date,
    Enum { variants: Vec<String> },
}
impl DataType {
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (DataType::Int, Value::Int(_)) => true,
            (DataType::Str, Value::Str(_)) => true,
            (DataType::Float, Value::Float(_)) => true,
            (DataType::Enum { variants: allowed }, Value::Enum { value: val }) => {
                allowed.contains(val)
            }
            (_, Value::Absen) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Str(String),
    Float(f64),
    Enum { value: String },

    // Date(chrono::NaiveDate),
    //saya ragu menggunakan crate luar pada angine.
    //bukan karena crate ini jelek justru sebaliknya powerfull.
    //alasannya hanya menghindari engine menjadi kebergantungan dengan ekosistem eksternal
    //
    // goal: logika Value::Date("YYYY-MM-HH") harus lahir dari dalam engine.
    // Value::Date akan release ketika otak saya memandang String manipulation sudah tidak rumit.

    // ini bukan data asli.
    // terpaksa di hadirkan
    // sebagai pengisi panjang kolom dan baris tetap relasi
    // terpaksa karena belum nemu alternatifnya.
    // kemungkinan solusinya akan ditemukan pada saat membangun constraint.
    Absen, // reperesentasi absen input. yah ini memang tidak jujur.
           // INFO: saat ini Value::Absen hanya lahir dari add column.
}
impl Value {
    pub fn compare(&self, op: &Cmp, to_cmp: &Value) -> bool {
        match (op, self, to_cmp) {
            (Cmp::Eq, Value::Absen, _) => false, //menghasilkan true masih dalam pertimbangan
            (Cmp::Eq, Value::Int(a), Value::Int(b)) => a == b,
            (Cmp::Eq, Value::Str(a), Value::Str(b)) => a == b,
            (Cmp::Eq, Value::Float(a), Value::Float(b)) => a == b,
            (Cmp::Eq, Value::Enum { value: a }, Value::Enum { value: b }) => a == b,
            (Cmp::Gt, Value::Int(a), Value::Int(b)) => a > b,
            (Cmp::Lt, Value::Int(a), Value::Int(b)) => a < b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cmp {
    Eq,
    Lt,
    Gt,
}
