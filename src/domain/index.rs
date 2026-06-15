// domain/index.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{Row, Schema, Value};

/// Unified index structure for both UNIQUE and PRIMARY KEY columns
/// Provides O(1) lookup and uniqueness checking
#[derive(Debug, Clone, Default)]
pub struct TableIndex {
    // Column name -> (Value -> RowIndex)
    // Includes both UNIQUE and PRIMARY KEY columns
    indices: HashMap<String, HashMap<Value, usize>>,
}

// domain/index.rs - Add method to handle row validation
// domain/index.rs - Add this method
impl TableIndex {
    // ... existing code ...

    /// Rebuild indices from live rows only
    pub fn rebuild_from_rows_with_filter(&mut self, schema: &Schema, live_rows: &[&Row]) {
        self.indices.clear();

        for (col_idx, col) in schema.columns().iter().enumerate() {
            if col.is_unique() || col.is_primary_key() {
                let mut col_index = std::collections::HashMap::new();
                for (row_idx, row) in live_rows.iter().enumerate() {
                    let val = &row.values()[col_idx];
                    if !matches!(val, Value::Null) {
                        col_index.insert(val.clone(), row_idx);
                    }
                }
                self.indices.insert(col.name().to_string(), col_index);
            }
        }
    }

    pub fn get_valid_row(
        &self,
        column_name: &str,
        value: &Value,
        is_row_alive: &dyn Fn(usize) -> bool,
    ) -> Option<usize> {
        if let Some(row_idx) = self.lookup(column_name, value) {
            if is_row_alive(row_idx) {
                return Some(row_idx);
            }
        }
        None
    }

    /// Remove all entries for a column that point to dead rows
    pub fn vacuum_column(
        &mut self,
        column_name: &str,
        is_row_alive: &dyn Fn(usize) -> bool,
    ) -> usize {
        let mut removed = 0;
        if let Some(col_index) = self.indices.get_mut(column_name) {
            col_index.retain(|_, &mut row_idx| {
                let keep = is_row_alive(row_idx);
                if !keep {
                    removed += 1;
                }
                keep
            });
        }
        removed
    }
}

// Custom serialization - skip index data, rebuild on load
impl Serialize for TableIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let state = serializer.serialize_struct("TableIndex", 0)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TableIndex {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Return empty index on deserialize - will be rebuilt via rebuild_from_rows()
        Ok(TableIndex::new())
    }
}

impl TableIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    /// Ensure an index exists for the given column
    /// Creates empty HashMap if not present
    pub fn ensure_index(&mut self, column_name: String) {
        if !self.indices.contains_key(&column_name) {
            self.indices.insert(column_name, HashMap::new());
        }
    }

    /// Remove index for a column completely
    pub fn remove_index(&mut self, column_name: &str) {
        self.indices.remove(column_name);
    }

    /// Insert a value into the index for a column
    /// Maps value -> row_idx for quick lookup
    pub fn insert(&mut self, column_name: &str, value: Value, row_idx: usize) {
        if let Some(col_index) = self.indices.get_mut(column_name) {
            col_index.insert(value, row_idx);
        }
    }

    /// Remove a value from the index for a column
    /// Only removes if the row_idx matches (prevents removing wrong row)
    pub fn remove(&mut self, column_name: &str, value: &Value, row_idx: usize) {
        if let Some(col_index) = self.indices.get_mut(column_name) {
            if let Some(&idx) = col_index.get(value) {
                if idx == row_idx {
                    col_index.remove(value);
                }
            }
        }
    }

    /// Lookup a value in the index
    /// Returns Some(row_idx) if found, None otherwise
    pub fn lookup(&self, column_name: &str, value: &Value) -> Option<usize> {
        self.indices
            .get(column_name)
            .and_then(|col_index| col_index.get(value))
            .copied()
    }

    /// Check if a value is unique for a column (excluding a specific row)
    /// Returns true if value already exists (not unique)
    /// Returns false if value is unique (not found in index)
    pub fn is_unique(&self, column_name: &str, value: &Value, exclude_row: Option<usize>) -> bool {
        match self.lookup(column_name, value) {
            Some(idx) => Some(idx) != exclude_row,
            None => false,
        }
    }

    /// Get mutable reference to all indices (for batch operations)
    pub fn indices_mut(&mut self) -> &mut HashMap<String, HashMap<Value, usize>> {
        &mut self.indices
    }

    /// Get reference to all indices (for inspection)
    pub fn indices(&self) -> &HashMap<String, HashMap<Value, usize>> {
        &self.indices
    }

    /// Check if a column has an index
    pub fn has_index(&self, column_name: &str) -> bool {
        self.indices.contains_key(column_name)
    }

    /// Get number of indexed columns
    pub fn indexed_columns_count(&self) -> usize {
        self.indices.len()
    }

    /// Get total number of indexed values across all columns
    pub fn total_entries(&self) -> usize {
        self.indices.values().map(|m| m.len()).sum()
    }

    /// Rebuild all indices from scratch using schema and rows
    /// This is O(n*m) where n = rows, m = indexed columns
    /// Should be called after deserialization or major changes
    pub fn rebuild_from_rows(&mut self, schema: &Schema, rows: &[Row]) {
        self.indices.clear();

        for (col_idx, col) in schema.columns().iter().enumerate() {
            // Create index for both UNIQUE and PRIMARY KEY columns
            if col.is_unique() || col.is_primary_key() {
                let mut col_index = HashMap::new();
                for (row_idx, row) in rows.iter().enumerate() {
                    let val = &row.values()[col_idx];
                    // NULL values are not indexed (they are ignored in uniqueness checks)
                    if !matches!(val, Value::Null) {
                        col_index.insert(val.clone(), row_idx);
                    }
                }
                self.indices.insert(col.name().to_string(), col_index);
            }
        }
    }

    /// Clear all indices
    pub fn clear(&mut self) {
        self.indices.clear();
    }

    /// Get all values for a specific column (for debugging)
    pub fn get_column_values(&self, column_name: &str) -> Vec<&Value> {
        self.indices
            .get(column_name)
            .map(|m| m.keys().collect())
            .unwrap_or_default()
    }

    /// Get row index for a specific column and value (returns copy)
    pub fn get_row_idx(&self, column_name: &str, value: &Value) -> Option<usize> {
        self.lookup(column_name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Constraint, DataType};

    // Helper to create a test schema
    fn create_test_schema() -> Schema {
        let mut schema = Schema::new();
        let columns: Vec<(&str, DataType, &[Constraint])> = vec![
            ("id", DataType::Int, &[Constraint::PrimaryKey]),
            ("name", DataType::Str, &[Constraint::Unique]),
            ("age", DataType::Int, &[]),
        ];
        schema.add_column(columns).unwrap();
        schema
    }

    #[test]
    fn test_new_index_is_empty() {
        let index = TableIndex::new();
        assert_eq!(index.indexed_columns_count(), 0);
        assert_eq!(index.total_entries(), 0);
    }

    #[test]
    fn test_ensure_index() {
        let mut index = TableIndex::new();
        index.ensure_index("id".to_string());
        assert!(index.has_index("id"));
        assert!(!index.has_index("name"));
    }

    #[test]
    fn test_insert_and_lookup() {
        let mut index = TableIndex::new();
        index.ensure_index("id".to_string());

        index.insert("id", Value::Int(1), 0);
        index.insert("id", Value::Int(2), 1);

        assert_eq!(index.lookup("id", &Value::Int(1)), Some(0));
        assert_eq!(index.lookup("id", &Value::Int(2)), Some(1));
        assert_eq!(index.lookup("id", &Value::Int(3)), None);
    }

    #[test]
    fn test_remove() {
        let mut index = TableIndex::new();
        index.ensure_index("id".to_string());

        index.insert("id", Value::Int(1), 0);
        assert_eq!(index.lookup("id", &Value::Int(1)), Some(0));

        index.remove("id", &Value::Int(1), 0);
        assert_eq!(index.lookup("id", &Value::Int(1)), None);
    }

    #[test]
    fn test_remove_wrong_row_does_nothing() {
        let mut index = TableIndex::new();
        index.ensure_index("id".to_string());

        index.insert("id", Value::Int(1), 0);
        index.remove("id", &Value::Int(1), 1); // wrong row_idx

        assert_eq!(index.lookup("id", &Value::Int(1)), Some(0));
    }

    #[test]
    fn test_is_unique() {
        let mut index = TableIndex::new();
        index.ensure_index("name".to_string());

        index.insert("name", Value::Str("Alice".to_string()), 0);

        // Value exists, not excluding row 0 -> not unique
        assert!(index.is_unique("name", &Value::Str("Alice".to_string()), None));

        // Value exists, excluding row 0 -> unique (same row)
        assert!(!index.is_unique("name", &Value::Str("Alice".to_string()), Some(0)));

        // Value doesn't exist -> unique
        assert!(!index.is_unique("name", &Value::Str("Bob".to_string()), None));
    }

    #[test]
    fn test_rebuild_from_rows() {
        use crate::domain::Row;

        let schema = create_test_schema();
        let mut rows = Vec::new();

        // Create row with id=1, name="Alice"
        let row1 = Row::new(vec![
            Value::Int(1),
            Value::Str("Alice".to_string()),
            Value::Int(25),
        ]);
        // Create row with id=2, name="Bob"
        let row2 = Row::new(vec![
            Value::Int(2),
            Value::Str("Bob".to_string()),
            Value::Int(30),
        ]);
        rows.push(row1);
        rows.push(row2);

        let mut index = TableIndex::new();
        index.rebuild_from_rows(&schema, &rows);

        // Should have indices for "id" (PK) and "name" (Unique)
        assert_eq!(index.indexed_columns_count(), 2);
        assert!(index.has_index("id"));
        assert!(index.has_index("name"));
        assert!(!index.has_index("age"));

        assert_eq!(index.lookup("id", &Value::Int(1)), Some(0));
        assert_eq!(index.lookup("id", &Value::Int(2)), Some(1));
        assert_eq!(
            index.lookup("name", &Value::Str("Alice".to_string())),
            Some(0)
        );
        assert_eq!(
            index.lookup("name", &Value::Str("Bob".to_string())),
            Some(1)
        );
    }

    #[test]
    fn test_null_values_not_indexed() {
        use crate::domain::Row;

        let schema = create_test_schema();
        let rows = vec![Row::new(vec![Value::Int(1), Value::Null, Value::Int(25)])];

        let mut index = TableIndex::new();
        index.rebuild_from_rows(&schema, &rows);

        // NULL should not be indexed
        assert_eq!(index.lookup("name", &Value::Null), None);
        assert_eq!(index.get_column_values("name").len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut index = TableIndex::new();
        index.ensure_index("id".to_string());
        index.insert("id", Value::Int(1), 0);

        assert_eq!(index.total_entries(), 1);

        index.clear();
        assert_eq!(index.total_entries(), 0);
        assert_eq!(index.indexed_columns_count(), 0);
    }

    #[test]
    fn test_multiple_columns() {
        let mut index = TableIndex::new();
        index.ensure_index("email".to_string());
        index.ensure_index("username".to_string());

        index.insert("email", Value::Str("a@example.com".to_string()), 0);
        index.insert("username", Value::Str("alice".to_string()), 0);
        index.insert("email", Value::Str("b@example.com".to_string()), 1);

        assert_eq!(index.total_entries(), 3);
        assert_eq!(
            index.lookup("email", &Value::Str("a@example.com".to_string())),
            Some(0)
        );
        assert_eq!(
            index.lookup("username", &Value::Str("alice".to_string())),
            Some(0)
        );
        assert_eq!(
            index.lookup("email", &Value::Str("b@example.com".to_string())),
            Some(1)
        );
    }
}
