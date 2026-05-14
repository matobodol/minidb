use serde::{Deserialize, Serialize};

use crate::domain::{DataType, DomainError, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    Nullable,
    NotNull,
    Unique,
    Increment,
    PrimaryKey,
    Default(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    name: String,
    data_type: DataType,
    constraint: Vec<Constraint>,
}

impl Column {
    pub(super) fn new(
        name: impl Into<String>,
        data_type: DataType,
        constraint: Vec<Constraint>,
    ) -> Self {
        Self {
            name: name.into(),
            data_type,
            constraint,
        }
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn data_type(&self) -> &DataType {
        &self.data_type
    }
}

impl Column {
    pub(crate) fn has_constraint(&self, predicate: impl Fn(&Constraint) -> bool) -> bool {
        self.constraint.iter().any(predicate)
    }

    pub(crate) fn get_constraint<T>(
        &self,
        extractor: impl Fn(&Constraint) -> Option<T>,
    ) -> Option<T> {
        self.constraint.iter().find_map(extractor)
    }

    pub(super) fn default_value(&self) -> Option<&Value> {
        self.constraint.iter().find_map(|c| {
            if let Constraint::Default(v) = c {
                Some(v)
            } else {
                None
            }
        })
    }
}

impl Column {
    pub(super) fn is_nullable(&self) -> bool {
        !self.has_constraint(|c| matches!(c, Constraint::NotNull | Constraint::PrimaryKey))
    }

    pub(super) fn is_unique(&self) -> bool {
        self.has_constraint(|c| matches!(c, Constraint::Unique | Constraint::PrimaryKey))
    }

    pub(crate) fn is_primary_key(&self) -> bool {
        self.has_constraint(|c| matches!(c, Constraint::PrimaryKey))
    }

    pub(crate) fn is_increment(&self) -> bool {
        self.has_constraint(|c| matches!(c, Constraint::Increment))
    }
}
impl Column {
    pub(super) fn enforce<'a>(
        &self,
        input: Option<Value>,
        mut existing_values: impl Iterator<Item = &'a Value>,
    ) -> Result<Value, DomainError> {
        let nullable = self.is_nullable();
        let unique = self.is_unique();
        let default = self.default_value();

        let v = match input {
            Some(v) => self.data_type.coerce_value(v)?,

            None => {
                if let Some(default) = default {
                    self.data_type.coerce_value(default.clone())?
                } else if !nullable {
                    return Err(DomainError::NotAllowedNull);
                } else {
                    Value::Null
                }
            }
        };

        if unique && !matches!(v, Value::Null) {
            if existing_values.any(|e| e == &v) {
                return Err(DomainError::NotUniqValue(self.name.clone()));
            }
        }

        Ok(v)
    }
}

// ============================================================
// ATURAN RESMI KOMBINASI CONSTRAINT KOLOM
// ============================================================
//
// MINDSET DASAR
// ------------------------------------------------------------
// - Sistem ini TIDAK PERNAH memiliki Value::Null atau Value::Absen
// - Null = ketiadaan nilai (tidak ada entry), BUKAN nilai
// - Value selalu konkret dan valid secara tipe
// - Schema adalah sumber kebenaran
// - Constraint bekerja pada MAKNA data, bukan struktur penyimpanan
//
// ------------------------------------------------------------
// DEFINISI ERROR
// ------------------------------------------------------------
// - Schema Error : kombinasi constraint tidak masuk akal,
//                  harus gagal saat definisi schema
// - Data Error   : schema valid, tapi data melanggar constraint
//
// ------------------------------------------------------------
// DAFTAR CONSTRAINT
// ------------------------------------------------------------
// - Nullable
// - NotNull
// - Unique
// - Increment
// - Default(Value)
//
// ============================================================
// ATURAN KOMBINASI (SCHEMA-TIME)
// ============================================================
//
// 1. Nullable ↔ NotNull
// ------------------------------------------------------------
// - Nullable + NotNull        -> ❌ Schema Error
// - Nullable saja             -> ✅ Valid
// - NotNull saja              -> ✅ Valid
// - Tidak ada keduanya        -> ⚠️ Dianggap Nullable (disarankan eksplisit)
//
// ------------------------------------------------------------
// 2. Default(Value)
// ------------------------------------------------------------
// - Default + Nullable        -> ✅ Valid
// - Default + NotNull         -> ✅ Valid
// - Default(null) + NotNull  -> ❌ Schema Error
//
// Catatan:
// - Default HARUS nilai konkret jika NotNull
// - Default tidak boleh merepresentasikan ketiadaan
//
// ------------------------------------------------------------
// 3. Unique
// ------------------------------------------------------------
// - Unique saja               -> ⚠️ Valid (null / ketiadaan diabaikan)
// - Unique + NotNull          -> ✅ Valid (kombinasi kuat)
// - Unique + null sebagai nilai
//                              -> ❌ Schema Error
//
// Catatan:
// - Unique hanya membandingkan nilai yang HADIR
// - Ketiadaan nilai TIDAK ikut perbandingan
//
// ------------------------------------------------------------
// 4. Default ↔ Unique
// ------------------------------------------------------------
// - Default(v) + Unique       -> ⚠️ Valid tapi berbahaya
//
// Catatan:
// - Insert berulang tanpa nilai akan menghasilkan duplikat default
// - Disarankan hanya aman jika digabung dengan Increment
//
// ------------------------------------------------------------
// 5. Increment
// ------------------------------------------------------------
// - Increment + Nullable      -> ❌ Schema Error
// - Increment + Default       -> ❌ Schema Error
// - Increment + NotNull       -> ✅ Valid
// - Increment + Unique        -> ✅ Valid
// - Increment saja            -> ⚠️ Implicit NotNull + Unique
//
// Rekomendasi desain:
// - Increment secara implisit mengaktifkan:
//   - NotNull
//   - Unique
//
// ------------------------------------------------------------
// 6. Kombinasi Ideal (Golden Path)
// ------------------------------------------------------------
// - Increment + Unique + NotNull
//   -> ✅ Kombinasi paling aman dan direkomendasikan
//
// ============================================================
// ATURAN DATA-TIME (INSERT / UPDATE)
// ============================================================
//
// - Jika nilai TIDAK HADIR:
//   - Nullable   -> ✅ OK
//   - NotNull    -> ❌ Data Error
//
// - Jika nilai HADIR:
//   - Nullable   -> ✅ OK
//   - NotNull    -> ✅ OK
//
// - Unique:
//   - Hanya nilai HADIR yang dibandingkan
//   - Ketiadaan nilai selalu diabaikan
//
// ============================================================
// KOMBINASI YANG WAJIB DITOLAK
// ============================================================
//
// ❌ Schema Error:
// - Nullable + NotNull
// - Increment + Nullable
// - Increment + Default
// - Default(null) + NotNull
// - Unique dengan null dianggap nilai
//
// ============================================================
// FILOSOFI AKHIR
// ============================================================
// - Null adalah ketiadaan, bukan nilai
// - Constraint harus bebas paradoks
// - Increment adalah constraint dominan
// - Unique tidak memikirkan ketiadaan
// - Jika Value ada -> ia selalu valid
// ============================================================
