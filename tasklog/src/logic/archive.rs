use super::{toast, tr::tr};
use crate::{
    db::{
        self,
        def::{RecordEntry, ARCHIVE_TABLE as DB_TABLE},
    },
    slint_generatedAppWindow::{AppWindow, Logic, RecordEntry as UIRecordEntry, Store},
    toast_success,
};
use slint::{ComponentHandle, Model, VecModel};

#[macro_export]
macro_rules! store_current_archive_entries {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_archive_entries()
            .as_any()
            .downcast_ref::<VecModel<UIRecordEntry>>()
            .expect("We know we set a VecModel<UIRecordEntry> earlier")
    };
}

pub fn init(ui: &AppWindow) {
    archive_init(ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_add_archive(move |entry| {
        let ui = ui_handle.unwrap();
        store_current_archive_entries!(ui).push(entry.clone());
        add_db_entry(&ui, entry.into());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_recover_archive(move |index| {
        let ui = ui_handle.unwrap();
        let index = index as usize;

        let entry = store_current_archive_entries!(ui).row_data(index).unwrap();
        store_current_archive_entries!(ui).remove(index);
        delete_db_entry(&ui, entry.uuid.clone().into());

        ui.global::<Logic>().invoke_new_record(entry);
        toast_success!(ui, tr("Recover entry successfully"));
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_archive(move |index| {
        let ui = ui_handle.unwrap();
        let index = index as usize;

        let entry = store_current_archive_entries!(ui).row_data(index).unwrap();
        store_current_archive_entries!(ui).remove(index);
        delete_db_entry(&ui, entry.uuid.into());
        toast_success!(ui, tr("Remove entry successfully"));
    });
}

fn archive_init(ui: &AppWindow) {
    store_current_archive_entries!(ui).set_vec(vec![]);

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
                .map(|entry: RecordEntry| entry.into())
                .collect::<Vec<UIRecordEntry>>();

            store_current_archive_entries!(ui.unwrap()).set_vec(entries);
        });
    });
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
