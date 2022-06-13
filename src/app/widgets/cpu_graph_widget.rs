use std::time::Instant;

use concat_string::concat_string;

use tui::widgets::Row;

use crate::{
    app::{data_harvester::cpu::CpuDataType, AppConfigFields},
    components::data_table::{DataColumn, DataTable, DataTableProps, ToDataRow},
    data_conversion::CpuWidgetData,
    utils::gen_util::truncate_text,
};

impl ToDataRow for CpuWidgetData {
    fn to_data_row<'a>(&'a self, columns: &[DataColumn]) -> Row<'a> {
        match self {
            CpuWidgetData::All => Row::new(vec![truncate_text(
                "All".into(),
                columns[0].calculated_width.into(),
            )]),
            CpuWidgetData::Entry {
                data_type,
                data: _,
                last_entry,
            } => {
                let entry_text = match data_type {
                    CpuDataType::Avg => {
                        truncate_text("AVG".into(), columns[0].calculated_width.into())
                    }
                    CpuDataType::Cpu(index) => {
                        let index = index.to_string();
                        let width = columns[0].calculated_width;
                        if width < 5 {
                            truncate_text(index.into(), width.into())
                        } else {
                            truncate_text(concat_string!("CPU", index).into(), width.into())
                        }
                    }
                };

                Row::new(vec![
                    entry_text,
                    truncate_text(
                        format!("{:.0}%", last_entry.round()).into(),
                        columns[1].calculated_width.into(),
                    ),
                ])
            }
        }
    }

    fn column_widths(_data: &[Self]) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![1, 3]
    }
}

pub struct CpuWidgetState {
    pub current_display_time: u64,
    pub is_legend_hidden: bool,
    pub show_avg: bool,
    pub autohide_timer: Option<Instant>,
    pub table: DataTable<CpuWidgetData>,
}

impl CpuWidgetState {
    pub fn new(
        config: &AppConfigFields, current_display_time: u64, autohide_timer: Option<Instant>,
    ) -> Self {
        const COLUMNS: [DataColumn; 2] = [
            DataColumn::soft("CPU", Some(0.5)),
            DataColumn::soft("Use%", Some(0.5)),
        ];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: false, // TODO: Should this be possible?
            show_current_entry_when_unfocused: true,
        };

        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            show_avg: config.show_average_cpu,
            autohide_timer,
            table: DataTable::new(COLUMNS.to_vec(), props),
        }
    }
}
