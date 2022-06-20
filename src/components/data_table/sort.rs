use std::{fmt::Display, marker::PhantomData, ops::Range};

use itertools::Itertools;
use tui::{layout::Rect, widgets::Row};

use crate::{components::old_text_table::SortOrder, utils::gen_util::truncate_text};

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

pub struct Sortable {
    /// The "y location" of the header row. Since all headers share the same y-location we
    /// just set it once here.
    y_loc: u16,

    /// The currently selected sort index.
    pub sort_index: usize,

    /// The current sorting order.
    pub order: SortOrder,

    /// Sort column information.
    pub sort_col_info: Vec<SortColumnInfo>,
}
impl SortType for Sortable {}

pub type SortDataTable<DataType, T> = DataTable<DataType, T, Sortable>;

pub trait SortsRow<DataType> {
    /// Sorts data.
    fn sort_data(&self, data: &mut [DataType], ascending: bool);
}

#[derive(Default)]
pub struct SortColumnInfo {
    /// The "x locations" of the column.
    pub range: Range<u16>,

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
                range: Range::default(),
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
                y_loc: 0,
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

    /// Sets the new draw locations for the table headers.
    ///
    /// **Note:** The function assumes the ranges will create a *sorted* list with the length
    /// equal to the number of columns - in debug mode, the program will assert all this, but
    /// it will **not** do so in release mode!
    pub fn update_header_locations(&mut self, draw_loc: Rect, row_widths: &[u16]) {
        let mut start = draw_loc.x;

        debug_assert_eq!(
            row_widths.len(),
            self.sort_type.sort_col_info.len(),
            "row width and sort column length should be equal"
        );

        row_widths
            .iter()
            .zip(self.sort_type.sort_col_info.iter_mut())
            .for_each(|(width, column)| {
                let range_start = start;
                let range_end = start + width + 1; // +1 for the gap b/w cols.
                start = range_end;

                column.range = range_start..range_end;
            });

        debug_assert!(
            self.sort_type
                .sort_col_info
                .iter()
                .all(|a| { a.range.start <= a.range.end }),
            "all sort column ranges should have a start <= end"
        );

        debug_assert!(
            self.sort_type
                .sort_col_info
                .iter()
                .tuple_windows()
                .all(|(a, b)| { b.range.start >= a.range.end }),
            "sort column ranges should be sorted"
        );

        self.sort_type.y_loc = draw_loc.y;
    }

    /// Given some `x` and `y`, if possible, select the corresponding column or toggle the column if already selected,
    /// and otherwise do nothing.
    ///
    /// If there was some update, the corresponding column type will be returned. If nothing happens, [`None`] is
    /// returned.
    pub fn try_select_location(&mut self, x: u16, y: u16) -> Option<usize> {
        if self.sort_type.y_loc == y {
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
        match self
            .sort_type
            .sort_col_info
            .binary_search_by_key(&needle, |col| col.range.start)
        {
            Ok(index) => Some(index),
            Err(index) => index.checked_sub(1),
        }
        .and_then(|index| {
            if needle < self.sort_type.sort_col_info[index].range.end {
                Some(index)
            } else {
                None
            }
        })
    }
}
