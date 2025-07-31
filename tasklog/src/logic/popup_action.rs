use super::tr::tr;
use crate::slint_generatedAppWindow::{AppWindow, ConfirmDialogSetting, Logic, PopupActionSetting};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<PopupActionSetting>()
        .on_action(move |action, user_data| {
            let ui = ui_handle.unwrap();

            #[allow(clippy::single_match)]
            match action.as_str() {
                "remove-all-cache" => {
                    println!("handel remove all cache");
                    ui.global::<Logic>().invoke_remove_all_cache();
                }
                "edit-record" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>()
                        .invoke_open_record_dialog(current_index);
                }
                "remove-record" => {
                    ui.global::<ConfirmDialogSetting>().invoke_set(
                        true,
                        tr("Warning").into(),
                        tr("Delete or not?").into(),
                        "remove-record".into(),
                        user_data,
                    );
                }
                "archive-record" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>().invoke_archive_record(current_index);
                }
                "plan-record" => {
                    let _current_index = user_data.parse::<i32>().unwrap();
                    // TODO
                }
                "remove-archive" => {
                    ui.global::<ConfirmDialogSetting>().invoke_set(
                        true,
                        tr("Warning").into(),
                        tr("Delete or not?").into(),
                        "remove-archive".into(),
                        user_data,
                    );
                }
                "recover-archive" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>().invoke_recover_archive(current_index);
                }
                _ => (),
            }
        });
}
