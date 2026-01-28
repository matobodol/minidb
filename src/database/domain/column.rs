use crate::database::domain::{DataType, Value};

#[derive(Debug, Clone)]
struct Flag {
    // _unique: bool,
    // _increment: bool,
    nullable: bool,
    default: Option<Value>,
}
impl Flag {
    fn new() -> Self {
        Self {
            // _unique: false,
            // _increment: false,
            nullable: true, //default sementara valid untuk  row baru,
            default: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    data_type: DataType,

    // tunda optimasi dan validadi
    flag: Flag,
}

impl Column {
    pub(crate) fn default_value(&self) -> Option<&Value> {
        self.flag.default.as_ref()
    }

    pub(crate) fn is_nullable(&self) -> bool {
        self.flag.nullable
    }

    pub(super) fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            flag: Flag::new(),
        }
    }
    pub(super) fn name(&self) -> &str {
        &self.name
    }
    pub(super) fn data_type(&self) -> &DataType {
        &self.data_type
    }
    pub(super) fn _is_nullable(&self) -> bool {
        self.flag.nullable
    }
}
