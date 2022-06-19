use std::fmt::Display;

use tui::widgets::Row;

use super::DataTableColumn;

pub trait ToDataRow {
    /// Builds a [`Row`] given data.
    fn to_data_row<'a, T: Display>(&self, columns: &[DataTableColumn<T>]) -> Row<'a>;

    /// Returns the desired column widths in light of having seen data.
    fn column_widths(data: &[Self]) -> Vec<u16>
    where
        Self: Sized;
}
