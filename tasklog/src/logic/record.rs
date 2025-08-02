use super::{toast, tr::tr};
use crate::{
    db::{
        self,
        def::{RecordEntry, RECORD_TABLE as DB_TABLE},
    },
    slint_generatedAppWindow::{
        AppWindow, Logic, PopupIndex, RecordEntry as UIRecordEntry,
        RecordPlanEntry as UIRecordPlanEntry, RecordState as UIRecordState, Store,
    },
    toast_success,
};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use uuid::Uuid;

#[macro_export]
macro_rules! store_current_record_entries {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_record_entries()
            .as_any()
            .downcast_ref::<VecModel<UIRecordEntry>>()
            .expect("We know we set a VecModel<UIRecordEntry> earlier")
    };
}

#[macro_export]
macro_rules! store_current_record_entries_cache {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_record_entries_cache()
            .as_any()
            .downcast_ref::<VecModel<UIRecordEntry>>()
            .expect("We know we set a VecModel<UIRecordEntry> for cache earlier")
    };
}

#[macro_export]
macro_rules! store_current_record_plan {
    ($entry:expr) => {
        $entry
            .as_any()
            .downcast_ref::<VecModel<UIRecordPlanEntry>>()
            .expect("We know we set a VecModel<UIRecordPlanEntry> earlier")
    };
}

pub fn init(ui: &AppWindow) {
    record_init(ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_new_record(move |mut entry| {
        let ui = ui_handle.unwrap();
        entry.uuid = Uuid::new_v4().to_string().into();

        if let Some(state) = calc_state(&entry.start_date, &entry.end_date, entry.state) {
            entry.state = state;
        }

        store_current_record_entries!(ui).insert(0, entry.clone());
        add_db_entry(&ui, entry.into());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_update_record(move |mut entry| {
        let ui = ui_handle.unwrap();
        if let Some(index) = store_current_record_entries!(ui)
            .iter()
            .position(|item| item.uuid == &entry.uuid)
        {
            if let Some(state) = calc_state(&entry.start_date, &entry.end_date, entry.state) {
                entry.state = state;
            }

            store_current_record_entries!(ui).set_row_data(index, entry.clone());
            update_db_entry(&ui, entry.into());
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_record(move |index| {
        let ui = ui_handle.unwrap();
        let index = index as usize;

        let entry = store_current_record_entries!(ui).row_data(index).unwrap();
        store_current_record_entries!(ui).remove(index);
        delete_db_entry(&ui, entry.uuid.into());
        toast_success!(ui, tr("Remove entry successfully"));
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_archive_record(move |index| {
        let ui = ui_handle.unwrap();
        let index = index as usize;

        let entry = store_current_record_entries!(ui).row_data(index).unwrap();
        store_current_record_entries!(ui).remove(index);
        delete_db_entry(&ui, entry.uuid.clone().into());

        ui.global::<Logic>().invoke_add_archive(entry);
        toast_success!(ui, tr("Archive entry successfully"));
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_open_record_dialog(move |index| {
        let ui = ui_handle.unwrap();

        if index < 0 {
            ui.global::<Store>()
                .set_edit_record_entry(Default::default());
        } else {
            let entry = store_current_record_entries!(ui)
                .row_data(index as usize)
                .unwrap();

            ui.global::<Store>().set_edit_record_entry(entry);
        }

        ui.global::<Logic>()
            .invoke_switch_popup(PopupIndex::RecordEdit);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_search_record(move |keyword| {
        let ui = ui_handle.unwrap();

        if keyword.is_empty() {
            let entries = store_current_record_entries_cache!(ui)
                .iter()
                .collect::<Vec<UIRecordEntry>>();

            store_current_record_entries!(ui).set_vec(entries);
            store_current_record_entries_cache!(ui).set_vec(vec![]);
            return;
        }

        if store_current_record_entries_cache!(ui).row_count() == 0 {
            let entries = store_current_record_entries!(ui)
                .iter()
                .collect::<Vec<UIRecordEntry>>();
            store_current_record_entries_cache!(ui).set_vec(entries);
        }

        let entries = store_current_record_entries_cache!(ui)
            .iter()
            .collect::<Vec<UIRecordEntry>>();

        let filter_entries = entries
            .iter()
            .filter_map(|entry| {
                if entry.title.contains(keyword.as_str()) {
                    Some(entry.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        store_current_record_entries!(ui).set_vec(filter_entries);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_refresh_records(move || {
        let ui = ui_handle.unwrap();
        record_init(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_update_record_state(move |index, state| {
            let ui = ui_handle.unwrap();
            let index = index as usize;
            let mut entry = store_current_record_entries!(ui).row_data(index).unwrap();
            let old_state = entry.state;

            log::debug!("{old_state:?} -> {state:?}");

            match state {
                UIRecordState::Running => {
                    entry.state = UIRecordState::Running;

                    match old_state {
                        UIRecordState::NotStarted => {
                            entry.start_date = cutil::time::local_now("%Y-%m-%d").into();
                        }
                        UIRecordState::Finished
                        | UIRecordState::Timeout
                        | UIRecordState::Giveup => {
                            entry.end_date = cutil::time::local_now("%Y-%m-%d").into();
                        }
                        _ => (),
                    }
                }
                UIRecordState::Finished => {
                    entry.state = UIRecordState::Finished;
                    entry.end_date = cutil::time::local_now("%Y-%m-%d").into();

                    if old_state == UIRecordState::NotStarted {
                        entry.start_date = entry.end_date.clone();
                    }
                }
                UIRecordState::Giveup => {
                    entry.state = UIRecordState::Giveup;
                    entry.end_date = cutil::time::local_now("%Y-%m-%d").into();

                    if old_state == UIRecordState::NotStarted {
                        entry.start_date = entry.end_date.clone();
                    }
                }
                _ => return,
            }

            if let Ok(diff_days) =
                cutil::time::diff_dates_to_days(&entry.start_date, &entry.end_date)
                && diff_days < 0
            {
                entry.end_date = entry.start_date.clone();
            }

            store_current_record_entries!(ui).set_row_data(index, entry.clone());
            update_db_entry(&ui, entry.into());
        });

    ui.global::<Logic>().on_record_progress(|entry| {
        let row_count = store_current_record_plan!(entry.plan).row_count();
        if row_count > 0 {
            let mut finished_counts = 0;
            for item in entry.plan.iter() {
                if item.is_finished {
                    finished_counts += 1;
                }
            }

            return finished_counts as f32 / row_count as f32;
        } else {
            let current_date = cutil::time::local_now("%Y-%m-%d");

            let diff_1 = cutil::time::diff_dates_to_days(&entry.start_date, &current_date)
                .unwrap_or_default()
                .max(0);

            let diff_2 = cutil::time::diff_dates_to_days(&entry.start_date, &entry.end_date)
                .unwrap_or_default()
                .max(1);

            if diff_1 >= diff_2 {
                return 1.0_f32;
            } else {
                diff_1 as f32 / diff_2 as f32
            }
        }
    });

    ui.global::<Logic>().on_remain_days(|start_date, end_date| {
        cutil::time::diff_dates_to_days(&start_date, &end_date)
            .unwrap_or_default()
            .max(0) as i32
    });

    ui.global::<Logic>()
        .on_remain_days_numbers(|start_date, end_date| {
            let days = cutil::time::diff_dates_to_days(&start_date, &end_date)
                .unwrap_or_default()
                .max(0) as i32;

            let days_numbers = if days < 10 {
                vec![0, days]
            } else {
                format!("{days}")
                    .chars()
                    .into_iter()
                    .map(|n| n.to_digit(10).unwrap_or_default() as i32)
                    .collect::<Vec<i32>>()
            };

            ModelRc::new(VecModel::from_slice(&days_numbers))
        });

    // ============================== record plan ========================== //

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_open_record_plan_dialog(move |index| {
            let ui = ui_handle.unwrap();

            let entry = store_current_record_entries!(ui)
                .row_data(index as usize)
                .unwrap();

            ui.global::<Store>().set_record_plan_entry(entry);

            ui.global::<Logic>()
                .invoke_switch_popup(PopupIndex::RecordPlan);
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_add_record_plan(move || {
        let ui = ui_handle.unwrap();
        let record_entry = ui.global::<Store>().get_record_plan_entry();
        store_current_record_plan!(record_entry.plan).push(UIRecordPlanEntry::default());
        ui.global::<Logic>().invoke_update_record(record_entry);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_record_plan(move |index| {
        let ui = ui_handle.unwrap();
        let record_entry = ui.global::<Store>().get_record_plan_entry();
        let plan_entries = store_current_record_plan!(record_entry.plan);

        let mut plan_entries_duplicate = plan_entries
            .iter()
            .map(|item| item.clone())
            .collect::<Vec<_>>();

        plan_entries_duplicate.remove(index as usize);
        plan_entries.set_vec(plan_entries_duplicate);
        ui.global::<Store>().set_next_record_plan_item_pos_y(0.0);
        ui.global::<Logic>().invoke_update_record(record_entry);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_record_plans(move || {
        let ui = ui_handle.unwrap();
        let record_entry = ui.global::<Store>().get_record_plan_entry();
        let plan_entries = store_current_record_plan!(record_entry.plan);

        plan_entries.set_vec(vec![]);
        ui.global::<Store>().set_next_record_plan_item_pos_y(0.0);
        ui.global::<Logic>().invoke_update_record(record_entry);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_update_record_plan(move |index, entry| {
            let ui = ui_handle.unwrap();
            let record_entry = ui.global::<Store>().get_record_plan_entry();
            let plan_entries = store_current_record_plan!(record_entry.plan);

            plan_entries.set_row_data(index as usize, entry);
            ui.global::<Logic>().invoke_update_record(record_entry);
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_move_record_plan(move |start_index, y, item_height| {
            let ui = ui_handle.unwrap();
            let start_index = start_index as usize;
            let record_entry = ui.global::<Store>().get_record_plan_entry();
            let plan_entries = store_current_record_plan!(record_entry.plan);
            let row_count = plan_entries.row_count();
            let end_index = (y / item_height).clamp(0.0, (row_count - 1) as f32) as usize;

            log::debug!("{start_index} => {end_index}");

            let mut plan_entries_duplicate = plan_entries
                .iter()
                .map(|item| item.clone())
                .collect::<Vec<_>>();
            let moving_plan_entry = plan_entries_duplicate[start_index].clone();

            if start_index != end_index {
                plan_entries_duplicate.remove(start_index);
                plan_entries_duplicate.insert(end_index, moving_plan_entry);
            }

            plan_entries.set_vec(plan_entries_duplicate);
            ui.global::<Store>().set_next_record_plan_item_pos_y(0.0);
            ui.global::<Logic>().invoke_update_record(record_entry);
        });

    ui.global::<Logic>().on_calc_record_plan_steps(|counts| {
        ModelRc::new(VecModel::from_slice(
            (0..counts)
                .map(|index| slint::format!("{}", index + 1))
                .collect::<Vec<_>>()
                .as_slice(),
        ))
    });

    ui.global::<Logic>().on_current_record_plan_step(|entries| {
        let mut index = 0;
        for entry in entries.iter() {
            if !entry.is_finished {
                return index;
            }

            index += 1;
        }

        index
    });
}

fn record_init(ui: &AppWindow) {
    store_current_record_entries!(ui).set_vec(vec![]);
    store_current_record_entries_cache!(ui).set_vec(vec![]);

    let ui = ui.as_weak();
    tokio::spawn(async move {
        let entries = match db::entry::select_all(DB_TABLE).await {
            Ok(items) => items
                .into_iter()
                .filter_map(|item| serde_json::from_str::<RecordEntry>(&item.data).ok())
                .map(|item| item.into())
                .collect(),

            Err(e) => {
                log::warn!("{:?}", e);
                vec![]
            }
        };

        _ = slint::invoke_from_event_loop(move || {
            let entries = entries
                .into_iter()
                .map(|mut entry: RecordEntry| {
                    if let Some(state) = calc_state(
                        entry.start_date.as_str(),
                        entry.end_date.as_str(),
                        entry.state,
                    ) {
                        entry.state = state;
                    }
                    entry
                })
                .map(|entry: RecordEntry| entry.into())
                .rev()
                .collect::<Vec<UIRecordEntry>>();

            store_current_record_entries!(ui.unwrap()).set_vec(entries);
        });
    });
}

fn calc_state(
    start_date: &str,
    end_date: &str,
    current_state: UIRecordState,
) -> Option<UIRecordState> {
    match current_state {
        UIRecordState::Giveup | UIRecordState::Finished => None,
        UIRecordState::NotStarted => match cutil::time::date_str_to_timestamp(end_date) {
            Ok(end_timestamp) => {
                let current_timestamp = cutil::time::timestamp();
                if current_timestamp > end_timestamp {
                    Some(UIRecordState::Timeout)
                } else {
                    match cutil::time::date_str_to_timestamp(start_date) {
                        Ok(start_timestamp) => {
                            if current_timestamp >= start_timestamp {
                                Some(UIRecordState::Running)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
            }
            _ => None,
        },
        UIRecordState::Running => match cutil::time::date_str_to_timestamp(start_date) {
            Ok(start_timestamp) => {
                let current_timestamp = cutil::time::timestamp();
                if current_timestamp < start_timestamp {
                    Some(UIRecordState::NotStarted)
                } else {
                    match cutil::time::date_str_to_timestamp(end_date) {
                        Ok(end_timestamp) => {
                            if current_timestamp > end_timestamp {
                                Some(UIRecordState::Timeout)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
            }
            _ => None,
        },
        UIRecordState::Timeout => match cutil::time::date_str_to_timestamp(start_date) {
            Ok(start_timestamp) => {
                let current_timestamp = cutil::time::timestamp();
                if current_timestamp < start_timestamp {
                    Some(UIRecordState::NotStarted)
                } else {
                    match cutil::time::date_str_to_timestamp(end_date) {
                        Ok(end_timestamp) => {
                            if current_timestamp <= end_timestamp {
                                Some(UIRecordState::Running)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
            }
            _ => None,
        },
    }
}

fn add_db_entry(ui: &AppWindow, entry: RecordEntry) {
    let ui = ui.as_weak();
    tokio::spawn(async move {
        let data = serde_json::to_string(&entry).unwrap();
        match db::entry::insert(DB_TABLE, &entry.uuid, &data).await {
            Err(e) => toast::async_toast_warn(
                ui,
                format!("{}. {}: {e:?}", tr("Add entry failed"), tr("Reason")),
            ),
            _ => (),
        }
    });
}

fn update_db_entry(ui: &AppWindow, entry: RecordEntry) {
    let ui = ui.as_weak();
    tokio::spawn(async move {
        let data = serde_json::to_string(&entry).unwrap();
        match db::entry::update(DB_TABLE, &entry.uuid, &data).await {
            Err(e) => toast::async_toast_warn(
                ui,
                format!("{}. {}: {e:?}", tr("Update entry failed"), tr("Reason")),
            ),
            _ => (),
        }
    });
}

pub fn delete_db_entry(ui: &AppWindow, uuid: String) {
    let ui = ui.as_weak();
    tokio::spawn(async move {
        match db::entry::delete(DB_TABLE, uuid.as_str()).await {
            Err(e) => toast::async_toast_warn(
                ui,
                format!("{}. {}: {e:?}", tr("Remove entry failed"), tr("Reason")),
            ),
            _ => (),
        }
    });
}
