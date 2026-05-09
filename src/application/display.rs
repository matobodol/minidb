use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table, TableComponent,
    presets::{UTF8_FULL, UTF8_FULL_CONDENSED},
};

use crate::domain::{Column, Constraint, DataType};

pub fn print_select(columns: Vec<Column>, rows: Vec<Vec<String>>) -> usize {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // =====================
    // HEADER
    // =====================
    let header = columns
        .iter()
        .map(|c| {
            let mut cell = Cell::new(c.name());

            if c.has_constraint(|c| matches!(c, Constraint::PrimaryKey)) {
                cell = cell.fg(Color::Red);
            } else if c.has_constraint(|c| matches!(c, Constraint::Unique)) {
                cell = cell.fg(Color::Green);
            }

            cell.add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center)
        })
        .collect::<Vec<Cell>>();

    table.set_header(header);

    // =====================
    // ROWS
    // =====================
    for row in &rows {
        let cells: Vec<Cell> = row
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let align = match columns[i].data_type() {
                    DataType::Int | DataType::Float => CellAlignment::Right,
                    DataType::Str => CellAlignment::Left,
                    DataType::Enum { .. } => CellAlignment::Center,
                };

                if value == "-" {
                    Cell::new(value).set_alignment(CellAlignment::Center)
                } else {
                    Cell::new(value).set_alignment(align)
                }
            })
            .collect();

        table.add_row(cells);
    }

    println!("{table}");

    // =====================
    // FOOTER
    // =====================
    let count = rows.len();

    count
}

pub fn print_describe(columns: Vec<Column>) {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .remove_style(TableComponent::HorizontalLines);

    // =====================
    // HEADER
    // =====================
    let headers = ["Field", "Type", "Null", "Key", "Default", "Extra"]
        .iter()
        .map(|text| {
            Cell::new(text)
                .add_attribute(Attribute::Bold)
                .set_alignment(CellAlignment::Center)
        })
        .collect::<Vec<Cell>>();

    table.set_header(headers);

    // =====================
    // ROWS
    // =====================
    for col in columns {
        let field = col.name().to_string();

        let dtype = match col.data_type() {
            DataType::Int => "int".to_string(),
            DataType::Float => "float".to_string(),
            DataType::Str => "string".to_string(),

            DataType::Enum { variants } => {
                let mut v: Vec<_> = variants.iter().cloned().collect();
                v.sort();

                format!("enum({})", v.join(","))
            }
        };

        // =====================
        // NULLABLE
        // =====================
        let is_nullable =
            if col.has_constraint(|c| matches!(c, Constraint::NotNull | Constraint::PrimaryKey)) {
                "NO"
            } else {
                "YES"
            };

        // =====================
        // KEY
        // =====================
        let key = if col.has_constraint(|c| matches!(c, Constraint::PrimaryKey)) {
            "PRI"
        } else if col.has_constraint(|c| matches!(c, Constraint::Unique)) {
            "UNI"
        } else {
            ""
        };

        // =====================
        // DEFAULT
        // =====================
        let default = col
            .get_constraint(|c| {
                if let Constraint::Default(v) = c {
                    Some(v.to_display_str())
                } else {
                    None
                }
            })
            .unwrap_or("".to_string());

        // =====================
        // EXTRA
        // =====================
        let extra = if col.is_increment() {
            "auto_increment"
        } else {
            ""
        };

        // =====================
        // COLOR
        // =====================
        let mut field_cell = Cell::new(field);

        if col.has_constraint(|c| matches!(c, Constraint::PrimaryKey)) {
            field_cell = field_cell.fg(Color::Red).add_attribute(Attribute::Bold);
        } else if col.has_constraint(|c| matches!(c, Constraint::Unique)) {
            field_cell = field_cell.fg(Color::Green).add_attribute(Attribute::Bold);
        }

        // =====================
        // ADD ROW
        // =====================
        table.add_row(vec![
            field_cell,
            Cell::new(dtype),
            Cell::new(is_nullable),
            Cell::new(key),
            Cell::new(default),
            Cell::new(extra),
        ]);
    }

    println!("{table}");
}
