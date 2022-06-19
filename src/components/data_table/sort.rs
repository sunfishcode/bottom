use std::{fmt::Display, marker::PhantomData};

use tui::widgets::Row;

use crate::utils::gen_util::truncate_text;

use super::{
    DataTable, DataTableColumn, DataTableProps, DataTableState, DataTableStyling, ToDataRow,
};

pub trait SortType {
    /// Constructs the table header.
    fn build_header<T: Display>(&self, columns: &[DataTableColumn<T>]) -> Row<'_> {
        Row::new(columns.iter().filter_map(|c| {
            if c.calculated_width == 0 {
                None
            } else {
                Some(truncate_text(
                    c.header.to_string().into(),
                    c.calculated_width.into(),
                ))
            }
        }))
    }
}

pub struct Unsortable;
impl SortType for Unsortable {}

impl<DataType: ToDataRow, T: Display> DataTable<DataType, T, Unsortable> {
    pub fn new<C: Into<Vec<DataTableColumn<T>>>>(
        columns: C, props: DataTableProps, styling: DataTableStyling,
    ) -> Self {
        Self {
            columns: columns.into(),
            state: DataTableState::default(),
            props,
            styling,
            sort_type: Unsortable,
            _pd: PhantomData,
        }
    }
}

pub struct Sortable {}
impl SortType for Sortable {}

pub type SortDataTable<DataType, T> = DataTable<DataType, T, Sortable>;

impl<DataType: ToDataRow, T: Display> DataTable<DataType, T, Sortable> {
    pub fn new_sortable<C: Into<Vec<DataTableColumn<T>>>>(
        columns: C, props: DataTableProps, styling: DataTableStyling,
    ) -> Self {
        Self {
            columns: columns.into(),
            state: DataTableState::default(),
            props,
            styling,
            sort_type: Sortable {},
            _pd: PhantomData,
        }
    }
}
