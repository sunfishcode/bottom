use std::{fmt::Display, marker::PhantomData};

use itertools::Itertools;
use tui::{layout::Rect, widgets::Row};

use crate::{components::old_text_table::SortOrder, utils::gen_util::truncate_text};

use super::{
    DataTable, DataTableColumn, DataTableProps, DataTableState, DataTableStyling, ToDataRow,
};

pub trait SortType {
    /// Constructs the table header.
    fn build_header<T: Display>(&self, columns: &[DataTableColumn<T>], widths: &[u16]) -> Row<'_> {
        Row::new(columns.iter().zip(widths).filter_map(|(c, &width)| {
            if width == 0 {
                None
            } else {
                Some(truncate_text(c.header.to_string().into(), width.into()))
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

pub struct Sortable {
    /// The currently selected sort index.
    pub sort_index: usize,

    /// The current sorting order.
    pub order: SortOrder,

    /// Sort column information.
    pub sort_col_info: Vec<SortColumnInfo>,
}
impl SortType for Sortable {
    // fn build_header<T: Display>(&self, columns: &[DataTableColumn<T>], widths: &[u16]) -> Row<'_> {
    //     todo!()
    // }
}

pub type SortDataTable<DataType, T> = DataTable<DataType, T, Sortable>;

pub trait SortsRow<DataType> {
    /// Sorts data.
    fn sort_data(&self, data: &mut [DataType], ascending: bool);
}

#[derive(Default)]
pub struct SortColumnInfo {
    /// A shortcut, if set.
    pub shortcut: Option<char>,

    /// The default sort ordering.
    pub default_order: SortOrder,
}

pub struct SortDataTableColumn<T: Display> {
    inner: DataTableColumn<T>,
    shortcut: Option<char>,
    default_order: SortOrder,
}

impl<T: Display> SortDataTableColumn<T> {
    pub fn new(inner: DataTableColumn<T>) -> Self {
        Self {
            inner,
            shortcut: Default::default(),
            default_order: Default::default(),
        }
    }

    pub fn shortcut(mut self, shortcut: Option<char>) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub fn default_order(mut self, default_order: SortOrder) -> Self {
        self.default_order = default_order;
        self
    }
}

pub struct SortDataTableProps {
    pub inner: DataTableProps,
    pub sort_index: usize,
    pub order: SortOrder,
}

impl<DataType: ToDataRow, T: Display + SortsRow<DataType>> DataTable<DataType, T, Sortable> {
    pub fn new_sortable<C: Into<Vec<SortDataTableColumn<T>>>>(
        columns: C, props: SortDataTableProps, styling: DataTableStyling,
    ) -> Self {
        let given_columns: Vec<_> = columns.into();
        let mut columns = Vec::with_capacity(given_columns.len());
        let mut sort_col_info = Vec::with_capacity(given_columns.len());

        for g in given_columns {
            columns.push(g.inner);
            sort_col_info.push(SortColumnInfo {
                shortcut: g.shortcut,
                default_order: g.default_order,
            });
        }

        Self {
            columns,
            state: DataTableState::default(),
            props: props.inner,
            styling,
            sort_type: Sortable {
                sort_index: props.sort_index,
                order: props.order,
                sort_col_info,
            },
            _pd: PhantomData,
        }
    }

    /// Returns the header at `index`, if it exists.
    pub fn get_header(&self, index: usize) -> Option<&T> {
        self.columns.get(index).map(|col| &col.header)
    }

    /// Toggles the current sort order.
    pub fn toggle_order(&mut self) {
        self.sort_type.order = match self.sort_type.order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    /// Given some `x` and `y`, if possible, select the corresponding column or toggle the column if already selected,
    /// and otherwise do nothing.
    ///
    /// If there was some update, the corresponding column type will be returned. If nothing happens, [`None`] is
    /// returned.
    pub fn try_select_location(&mut self, x: u16, y: u16) -> Option<usize> {
        if self.state.inner_rect.height > 1 && self.state.inner_rect.y == y {
            if let Some(index) = self.get_range(x) {
                self.set_sort_index(index);
                Some(self.sort_type.sort_index)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Updates the sort index, and sets the sort order as appropriate.
    ///
    /// If the index is different from the previous one, it will move to the new index and set the sort order
    /// to the prescribed default sort order.
    ///
    /// If the index is the same as the previous one, it will simply toggle the current sort order.
    pub fn set_sort_index(&mut self, index: usize) {
        if self.sort_type.sort_index == index {
            self.toggle_order();
        } else if let Some(col) = self.sort_type.sort_col_info.get(index) {
            self.sort_type.sort_index = index;
            self.sort_type.order = col.default_order;
        }
    }

    /// Given a `needle` coordinate, select the corresponding index and value.
    fn get_range(&self, needle: u16) -> Option<usize> {
        let mut start = self.state.inner_rect.x;
        let range = self
            .state
            .calculated_widths
            .iter()
            .map(|width| {
                let entry_start = start;
                start += width + 1; // +1 for the gap b/w cols.

                entry_start
            })
            .collect_vec();

        match range.binary_search(&needle) {
            Ok(index) => Some(index),
            Err(index) => index.checked_sub(1),
        }
    }
}
