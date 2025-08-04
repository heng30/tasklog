use super::tr::tr;
use crate::{
    db::{
        self,
        def::{RecordEntry, ARCHIVE_TABLE, RECORD_TABLE},
    },
    slint_generatedAppWindow::{
        AppWindow, ChartBarEntry as UIChartBarEntry, Logic, RecordEntry as UIRecordEntry,
        RecordState as UIRecordState, Store,
    },
};
use slint::{Brush::SolidColor, ComponentHandle, Model, ModelRc, VecModel};

#[macro_export]
macro_rules! store_statistic_entries {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_statistic_entries()
            .as_any()
            .downcast_ref::<VecModel<UIRecordEntry>>()
            .expect("We know we set a VecModel<UIRecordEntry> for statistic earlier")
    };
}

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_statistic_init(move || {
        let ui = ui_handle.unwrap();
        statistic_init(&ui);
    });

    ui.global::<Logic>()
        .on_statistic_total_days_spent(move |entries| {
            let mut days = 0;
            let current_timestamp = cutil::time::timestamp();
            let day_seconds = 24 * 60 * 60;

            for entry in entries.iter() {
                let start_timestamp = cutil::time::date_str_to_timestamp(&entry.start_date);
                let end_timestamp = cutil::time::date_str_to_timestamp(&entry.end_date);
                if start_timestamp.is_err() || end_timestamp.is_err() {
                    continue;
                }

                let start_timestamp = start_timestamp.unwrap();
                let end_timestamp = end_timestamp.unwrap();
                if current_timestamp < start_timestamp {
                    continue;
                }

                if current_timestamp < end_timestamp {
                    days += (current_timestamp - start_timestamp) / day_seconds;
                } else {
                    days += (end_timestamp - start_timestamp) / day_seconds;
                }
            }

            return days as i32;
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_statistic_chart_tasks_count(move |entries| {
            let ui = ui_handle.unwrap();

            let mut items = vec![
                UIChartBarEntry {
                    label: tr("NotStarted").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::NotStarted),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Running").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Running),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Finished").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Finished),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Giveup").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Giveup),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Timeout").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Timeout),
                    ),
                },
            ];

            for entry in entries.iter() {
                let index = match entry.state {
                    UIRecordState::NotStarted => 0,
                    UIRecordState::Running => 1,
                    UIRecordState::Finished => 2,
                    UIRecordState::Giveup => 3,
                    UIRecordState::Timeout => 4,
                };

                items[index].value += 1;
            }

            ModelRc::new(VecModel::from_slice(&items))
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_statistic_chart_days_spent(move |entries| {
            let ui = ui_handle.unwrap();

            let mut items = vec![
                UIChartBarEntry {
                    label: tr("Running").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Running),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Finished").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Finished),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Giveup").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Giveup),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Timeout").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Timeout),
                    ),
                },
            ];

            let current_timestamp = cutil::time::timestamp();
            let day_seconds = 24 * 60 * 60;

            for entry in entries.iter() {
                if entry.state == UIRecordState::NotStarted {
                    continue;
                }

                let start_timestamp = cutil::time::date_str_to_timestamp(&entry.start_date);
                let end_timestamp = cutil::time::date_str_to_timestamp(&entry.end_date);
                if start_timestamp.is_err() || end_timestamp.is_err() {
                    continue;
                }

                let start_timestamp = start_timestamp.unwrap();
                let end_timestamp = end_timestamp.unwrap();
                if current_timestamp < start_timestamp {
                    continue;
                }

                let days = if current_timestamp < end_timestamp {
                    (current_timestamp - start_timestamp) / day_seconds
                } else {
                    (end_timestamp - start_timestamp) / day_seconds
                };

                let index = match entry.state {
                    UIRecordState::Running => 0,
                    UIRecordState::Finished => 1,
                    UIRecordState::Giveup => 2,
                    UIRecordState::Timeout => 3,
                    _ => unreachable!(),
                };

                items[index].value += days as i32;
            }

            ModelRc::new(VecModel::from_slice(&items))
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_statistic_chart_mean_days_spent(move |entries| {
            let ui = ui_handle.unwrap();

            let mut counts = vec![0; 4];
            let mut items = vec![
                UIChartBarEntry {
                    label: tr("Running").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Running),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Finished").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Finished),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Giveup").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Giveup),
                    ),
                },
                UIChartBarEntry {
                    label: tr("Timeout").into(),
                    value: 0,
                    color: SolidColor(
                        ui.global::<Logic>()
                            .invoke_state_color(UIRecordState::Timeout),
                    ),
                },
            ];

            let current_timestamp = cutil::time::timestamp();
            let day_seconds = 24 * 60 * 60;

            for entry in entries.iter() {
                if entry.state == UIRecordState::NotStarted {
                    continue;
                }

                let start_timestamp = cutil::time::date_str_to_timestamp(&entry.start_date);
                let end_timestamp = cutil::time::date_str_to_timestamp(&entry.end_date);
                if start_timestamp.is_err() || end_timestamp.is_err() {
                    continue;
                }

                let start_timestamp = start_timestamp.unwrap();
                let end_timestamp = end_timestamp.unwrap();
                if current_timestamp < start_timestamp {
                    continue;
                }

                let days = if current_timestamp < end_timestamp {
                    (current_timestamp - start_timestamp) / day_seconds
                } else {
                    (end_timestamp - start_timestamp) / day_seconds
                };

                let index = match entry.state {
                    UIRecordState::Running => 0,
                    UIRecordState::Finished => 1,
                    UIRecordState::Giveup => 2,
                    UIRecordState::Timeout => 3,
                    _ => unreachable!(),
                };

                counts[index] += 1;
                items[index].value += days as i32;
            }

            for index in 0..4 {
                if counts[index] <= 0 {
                    continue;
                }

                items[index].value = items[index].value / counts[index];
            }

            ModelRc::new(VecModel::from_slice(&items))
        });
}

fn statistic_init(ui: &AppWindow) {
    store_statistic_entries!(ui).set_vec(vec![]);

    let ui = ui.as_weak();
    tokio::spawn(async move {
        let mut entries = vec![];

        let archive_entries = match db::entry::select_all(ARCHIVE_TABLE).await {
            Ok(items) => items
                .into_iter()
                .filter_map(|item| serde_json::from_str::<RecordEntry>(&item.data).ok())
                .collect::<Vec<RecordEntry>>(),

            Err(e) => {
                log::warn!("{:?}", e);
                vec![]
            }
        };

        let record_entries = match db::entry::select_all(RECORD_TABLE).await {
            Ok(items) => items
                .into_iter()
                .filter_map(|item| serde_json::from_str::<RecordEntry>(&item.data).ok())
                .collect::<Vec<RecordEntry>>(),

            Err(e) => {
                log::warn!("{:?}", e);
                vec![]
            }
        };

        entries.extend(archive_entries.into_iter());
        entries.extend(record_entries.into_iter());

        _ = slint::invoke_from_event_loop(move || {
            let entries = entries
                .into_iter()
                .map(|entry: RecordEntry| entry.into())
                .collect::<Vec<UIRecordEntry>>();

            store_statistic_entries!(ui.unwrap()).set_vec(entries);
        });
    });
}
