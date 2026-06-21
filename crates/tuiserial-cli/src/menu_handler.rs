//! Menu action handler — dispatches menu bar actions to the appropriate logic.

use rust_i18n::t;
use tuiserial_core::{AppState, MenuAction, menu_def::MENU_BAR};

use crate::handler::SerialHandler;
use crate::plugin_adapter::PluginProxy;

/// Handle menu action execution.
pub fn handle_menu_action(
    app: &mut AppState,
    handler: &mut SerialHandler,
    plugin_proxy: &mut PluginProxy,
    menu_idx: usize,
    item_idx: usize,
) -> bool {
    let action = match MENU_BAR.get_action(menu_idx, item_idx) {
        Some(a) => a,
        None => return false,
    };

    if action.is_separator() {
        return false;
    }

    match action {
        MenuAction::SaveConfig => {
            match app.save_config() {
                Ok(_) => app.add_success(t!("notify.config_saved").to_string()),
                Err(e) => app.add_error(format!("{}: {}", t!("notify.config_save_failed"), e)),
            }
            false
        }
        MenuAction::LoadConfig => {
            app.load_config();
            app.add_success(t!("notify.config_loaded").to_string());
            false
        }
        MenuAction::Exit => {
            if handler.is_connected() {
                handler.disconnect();
            }
            true
        }
        MenuAction::ToggleLanguage => {
            app.toggle_language();
            app.add_success(t!("notify.language_changed").to_string());
            false
        }
        MenuAction::ShowShortcuts => {
            app.show_shortcuts_help = !app.show_shortcuts_help;
            false
        }
        MenuAction::ShowAbout => {
            let about_text = if app.language == tuiserial_core::Language::English {
                "TuiSerial v0.2.0\nTerminal Serial Port Monitor\n\nA modern serial port debugging tool with mouse support, plugin system and internationalization."
            } else {
                "TuiSerial v0.2.0\n终端串口监控工具\n\n一个现代化的串口调试工具，支持插件系统、鼠标操作和国际化。"
            };
            app.add_info(about_text.to_string());
            false
        }
        MenuAction::NewSession
        | MenuAction::DuplicateSession
        | MenuAction::RenameSession
        | MenuAction::CloseSession => {
            app.add_info("Multi-session support coming soon!".to_string());
            false
        }
        MenuAction::ViewSingle
        | MenuAction::ViewSplitHorizontal
        | MenuAction::ViewSplitVertical
        | MenuAction::ViewGrid2x2
        | MenuAction::ViewNextPane
        | MenuAction::ViewPrevPane => {
            app.add_info("Layout management coming soon!".to_string());
            false
        }
        MenuAction::PluginsInstall => {
            plugin_proxy.open_registry_modal(app);
            false
        }
        MenuAction::PluginsCheckUpdate => {
            plugin_proxy.check_updates_action(app);
            false
        }
        MenuAction::PluginsUpdateAll => {
            plugin_proxy.update_all_action(app);
            false
        }
        MenuAction::PluginsReload => {
            plugin_proxy.reload_all_plugins(app);
            false
        }
        MenuAction::PluginsManager => {
            plugin_proxy.open_local_modal(app);
            false
        }
        MenuAction::PluginsList => {
            plugin_proxy.list_plugins_action(app);
            false
        }
        MenuAction::Separator => false,
    }
}
