use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table, TableComponent,
    modifiers::UTF8_ROUND_CORNERS,
    presets::{UTF8_FULL, UTF8_FULL_CONDENSED},
};

use crate::domain::{Column, Constraint, DataType};

pub fn print_select(columns: Vec<Column>, rows: Vec<Vec<String>>) -> usize {
    let mut table = Table::new();

    table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    // table.load_preset(UTF8_FULL); //full border
    table.load_preset(UTF8_FULL_CONDENSED);

    // =====================
    // HEADER
    // =====================
    let header: Vec<Cell> = columns
        .iter()
        .map(|c| {
            let mut cell = Cell::new(c.name());

            if c.has_constraint(|c| matches!(c, Constraint::PrimaryKey)) {
                cell = cell.fg(Color::Red).add_attribute(Attribute::Bold);
            } else if c.has_constraint(|c| matches!(c, Constraint::Unique)) {
                cell = cell.fg(Color::Green).add_attribute(Attribute::Bold);
            }

            cell
        })
        .collect();

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

    rows.len()
}

pub fn print_select_column(columns: &[&str], rows: Vec<Vec<String>>) -> usize {
    let mut table = Table::new();

    table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // table.load_preset(UTF8_FULL); //full border
    table.load_preset(UTF8_FULL_CONDENSED);

    // header
    let header: Vec<Cell> = columns
        .iter()
        .map(|c| Cell::new(*c).set_alignment(CellAlignment::Center))
        .collect();

    table.set_header(header);

    // rows
    for row in &rows {
        let cells: Vec<Cell> = row
            .iter()
            .map(|value| Cell::new(value).set_alignment(CellAlignment::Right))
            .collect();

        table.add_row(cells);
    }

    println!("{table}");

    rows.len()
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
    table.set_header(vec![
        Cell::new("Field").add_attribute(Attribute::Bold),
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Null").add_attribute(Attribute::Bold),
        Cell::new("Key").add_attribute(Attribute::Bold),
        Cell::new("Default").add_attribute(Attribute::Bold),
    ]);

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

        let is_nullable =
            if col.has_constraint(|c| matches!(c, Constraint::NotNull | Constraint::PrimaryKey)) {
                "NO"
            } else {
                "YES"
            };

        let key = if col.has_constraint(|c| matches!(c, Constraint::PrimaryKey)) {
            "PRI"
        } else if col.has_constraint(|c| matches!(c, Constraint::Unique)) {
            "UNI"
        } else {
            ""
        };

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
        // COLOR (optional tapi cakep)
        // =====================
        let mut field_cell = Cell::new(field);

        if col.has_constraint(|c| matches!(c, Constraint::PrimaryKey)) {
            field_cell = field_cell.fg(Color::Red).add_attribute(Attribute::Bold);
        } else if col.has_constraint(|c| matches!(c, Constraint::Unique)) {
            field_cell = field_cell.fg(Color::Green).add_attribute(Attribute::Bold);
        }

        table.add_row(vec![
            field_cell,
            Cell::new(dtype),
            Cell::new(is_nullable),
            Cell::new(key),
            Cell::new(default),
        ]);
    }

    println!("{table}");
}
