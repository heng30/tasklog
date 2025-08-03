use crate::slint_generatedAppWindow::{AppWindow, Logic, Util};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Util>()
        .on_handle_confirm_dialog(move |handle_type, user_data| {
            let ui = ui_handle.unwrap();

            #[allow(clippy::single_match)]
            match handle_type.as_str() {
                "remove-all-cache" => {
                    ui.global::<Logic>().invoke_remove_all_cache();
                }
                "close-window" => {
                    ui.global::<Util>().invoke_close_window();
                }
                "remove-record" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>().invoke_remove_record(current_index);
                }
                "remove-archive" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>().invoke_remove_archive(current_index);
                }
                "ai-generate-record-plans" => {
                    ui.global::<Logic>().invoke_ai_generate_record_plans();
                }
                "remove-record-plan" => {
                    let current_index = user_data.parse::<i32>().unwrap();
                    ui.global::<Logic>()
                        .invoke_remove_record_plan(current_index);
                }
                "remove-all-record-plans" => {
                    ui.global::<Logic>().invoke_remove_all_record_plans();
                }
                _ => (),
            }
        });
}
