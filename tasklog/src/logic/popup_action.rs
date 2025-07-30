use crate::slint_generatedAppWindow::{AppWindow, Logic, PopupActionSetting};
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
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>().invoke_delete_record(current_index);
                }
                "archive-record" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    // TODO
                }
                "plan-record" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    // TODO
                }
                _ => (),
            }
        });
}
