use crate::{
    db::{
        self,
        def::{RecordEntry, RECORD_TABLE as DB_TABLE},
    },
    slint_generatedAppWindow::{AppWindow, Logic, PopupIndex, RecordEntry as UIRecordEntry, Store},
};
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};

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

pub fn init(ui: &AppWindow) {
    record_init(ui);

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

    ui.global::<Logic>().on_search_record(move |keyword| {
        // let values = entries
        //     .iter()
        //     .flat_map(|entry| {
        //         if entry.children.row_count() > 0 {
        //             entry
        //                 .children
        //                 .iter()
        //                 .map(|item| item.title)
        //                 .collect::<Vec<_>>()
        //         } else {
        //             vec![entry.category]
        //         }
        //     })
        //     .collect::<Vec<_>>();
        // ModelRc::new(VecModel::from_slice(&values[..]))
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
                .map(|entry: RecordEntry| entry.into())
                .rev()
                .collect::<Vec<UIRecordEntry>>();

            store_current_record_entries!(ui.unwrap()).set_vec(entries);
        });
    });
}
