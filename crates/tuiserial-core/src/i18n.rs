//! Internationalization support for tuiserial
//!
//! Simple, compile-time i18n using lazy_static and HashMap.
//! No complex framework - just straightforward key-value lookups.

use crate::Language;

/// Translation function - returns translated string or key itself if not found
pub fn t(key: &'static str, lang: Language) -> &'static str {
    match lang {
        Language::English => EN.get(key).copied().unwrap_or(key),
        Language::Chinese => ZH.get(key).copied().unwrap_or(key),
    }
}

/// English translations
static EN: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // Menu bar
    "menu.file" => "File",
    "menu.session" => "Session",
    "menu.view" => "View",
    "menu.settings" => "Settings",
    "menu.help" => "Help",

    // File menu
    "menu.file.save_config" => "Save Config",
    "menu.file.load_config" => "Load Config",
    "menu.file.exit" => "Exit",

    // Session menu
    "menu.session.new" => "New Session",
    "menu.session.duplicate" => "Duplicate Session",
    "menu.session.rename" => "Rename Session",
    "menu.session.close" => "Close Session",

    // View menu
    "menu.view.single" => "Single View",
    "menu.view.split_h" => "Split Horizontal",
    "menu.view.split_v" => "Split Vertical",
    "menu.view.grid_2x2" => "Grid 2×2",
    "menu.view.next_pane" => "Next Pane",
    "menu.view.prev_pane" => "Previous Pane",

    // Settings menu
    "menu.settings.language" => "Language",
    "menu.settings.toggle_language" => "Toggle Language",

    // Plugins menu
    "menu.plugins" => "Plugins",
    "menu.plugins.install" => "Install from Registry",
    "menu.plugins.check_update" => "Check for Updates",
    "menu.plugins.update_all" => "Update All",
    "menu.plugins.reload" => "Reload Plugins",
    "menu.plugins.list" => "Plugin List",
    "menu.plugins.manager" => "Plugin Manager",

    // Help menu
    "menu.help.shortcuts" => "Keyboard Shortcuts",
    "menu.help.about" => "About",

    // UI labels
    "label.port" => "Port",
    "label.baud_rate" => "Baud Rate",
    "label.data_bits" => "Data Bits",
    "label.parity" => "Parity",
    "label.stop_bits" => "Stop Bits",
    "label.flow_control" => "Flow Control",
    "label.display_mode" => "Display Mode",
    "label.tx_mode" => "TX Mode",
    "label.append_mode" => "Append",
    "label.rx_count" => "RX",
    "label.tx_count" => "TX",
    "label.message" => "Message",
    "label.locked" => "Locked",
    "label.status" => "Status",
    "label.statistics" => "Statistics",
    "label.send" => "Send",
    "label.input_prompt" => "Input data...",

    // Status
    "status.connected" => "Connected",
    "status.disconnected" => "Disconnected",
    "status.modifiable" => "Modifiable",
    "status.locked" => "Locked",
    "status.not_connected" => "Not connected - press o to connect",

    // Parity values
    "parity.none" => "None",
    "parity.even" => "Even",
    "parity.odd" => "Odd",

    // Flow control values
    "flow.none" => "None",
    "flow.hardware" => "Hardware",
    "flow.software" => "Software",

    // Display mode
    "display.hex" => "HEX",
    "display.text" => "TEXT",

    // TX mode
    "tx.hex" => "HEX",
    "tx.ascii" => "ASCII",

    // Append mode
    "append.none" => "None",
    "append.lf" => "\\n",
    "append.cr" => "\\r",
    "append.crlf" => "\\r\\n",
    "append.lfcr" => "\\n\\r",

    // Button labels
    "button.connect" => "Connect",
    "button.disconnect" => "Disconnect",
    "button.clear" => "Clear",
    "button.send" => "Send",

    // Hints
    "hint.select" => "Select",
    "hint.refresh" => "Refresh",
    "hint.switch" => "Switch",
    "hint.clear" => "Clear",
    "hint.toggle" => "Toggle",
    "hint.scroll" => "Scroll",
    "hint.quit" => "Quit",
    "hint.exit" => "Exit",
    "hint.auto_scroll" => "Auto Scroll",

    // Notifications
    "notify.config_saved" => "Configuration saved",
    "notify.config_loaded" => "Configuration loaded",
    "notify.config_save_failed" => "Failed to save configuration",
    "notify.config_load_failed" => "Failed to load configuration",
    "notify.language_changed" => "Language changed",
    "notify.connected" => "Connected",
    "notify.disconnected" => "Disconnected",
    "notify.connection_failed" => "Connection failed",
    "notify.config_locked_warning" => "Config locked, please disconnect first",
    "notify.port_selected" => "Port selected",
    "notify.send_success" => "Sent",
    "notify.send_failed" => "Send failed",
    "notify.hex_format_error" => "HEX format error",
    "notify.not_connected" => "Not connected",
    "notify.input_empty" => "Input is empty",
    "notify.ports_refreshed" => "Ports refreshed",
    "notify.display_mode" => "Display mode",
    "notify.tx_mode" => "TX mode",
    "notify.append_mode" => "Append",
    "notify.disconnected_unlocked" => "Disconnected, config unlocked",
    "notify.please_select_port" => "Please select a port first",
    "notify.connected_locked" => "Connected: {} (config locked)",
    "notify.parity" => "Parity",
    "notify.flow_control" => "Flow control",
    "notify.log_cleared" => "Log cleared",
    "notify.toggle_display_mode" => "Display mode toggled",
    "notify.auto_scroll" => "Auto scroll",
    "notify.enabled" => "Enabled",
    "notify.disabled" => "Disabled",
    "notify.plugin_disabled" => "Plugin is Disabled. Please add plugin feature to use plugin",
    "notify.plugins_loaded" => "{} plugin(s) loaded",
    "notify.plugins_reloaded" => "Plugins reloaded",
    "notify.plugin_error" => "Plugin error",
    "notify.plugin_installed" => "Plugin '{}' installed — use Reload to activate",
    "notify.plugin_install_failed" => "Install failed: {}",
    "notify.plugin_checking" => "Checking for updates...",
    "notify.plugin_up_to_date" => "All plugins up to date",
    "notify.plugin_update_available" => "{} has update: {} → {}",
    "notify.plugin_updated" => "{} updated successfully",
    "notify.plugin_update_failed" => "Update {} failed: {}",
    "notify.plugin_all_updated" => "{} plugin(s) updated",
    "notify.plugin_registry_empty" => "No plugins available in registry",
    "notify.plugin_git_missing" => "Git is not available — install git to manage plugins",

    // Plugin modal
    "plugin.modal.title" => "Plugin Manager",
    "plugin.modal.status" => "Status",
    "plugin.modal.name" => "Plugin",
    "plugin.modal.hooks" => "Hooks",
    "plugin.modal.error" => "Error",
    "plugin.modal.no_plugins" => "No plugins installed",
    "plugin.modal.hint_close" => "Esc/Q: Close",
    "plugin.modal.hint_reload" => "R: Reload All",
    "plugin.modal.hint_navigate" => "↑↓: Navigate",
    "plugin.modal.hint_install" => "Enter: Install",
    "plugin.modal.hint_back" => "Esc: Back",
    "plugin.registry.title" => "Plugin Registry",
    "plugin.registry.search_placeholder" => "Type to search...",
    "plugin.registry.hint_search" => "Type to search",
    "plugin.registry.hint_install" => "Enter: Install",
    "plugin.registry.hint_back" => "Esc: Back",
    "plugin.registry.loading" => "Fetching registry...",
    "plugin.registry.empty" => "No matching plugins",
    "plugin.registry.installed" => "✓ Installed",
    "plugin.status.loading" => "Loading…",
    "plugin.status.loaded" => "Loaded",
    "plugin.status.error" => "Error",
    "plugin.status.disabled" => "Disabled",
    "plugin.hook.rx" => "RX",
    "plugin.hook.tx" => "TX",
    "plugin.hook.connect" => "CN",
    "plugin.hook.disconnect" => "DC",
    "plugin.hook.none" => "—",

    // Plugin status bar
    "plugin.bar.loaded" => "Plugins",
    "plugin.bar.none" => "No plugins",

    // Help text
    "help.f10" => "F10: Menu",
    "help.tab" => "Tab: Next Field",
    "help.shift_tab" => "Shift+Tab: Prev Field",
    "help.esc" => "Esc: Cancel/Close",
    "help.enter" => "Enter: Select/Send",

    // Keyboard shortcuts
    "shortcuts.title" => "Keyboard Shortcuts",
    "shortcuts.session" => "Session Management:",
    "shortcuts.new_session" => "Ctrl+T: New Session",
    "shortcuts.close_session" => "Ctrl+W: Close Session",
    "shortcuts.next_session" => "Ctrl+Tab / Ctrl+→: Next Session",
    "shortcuts.prev_session" => "Ctrl+Shift+Tab / Ctrl+←: Previous Session",
    "shortcuts.switch_1_9" => "Ctrl+1~9: Switch to Session 1~9",
    "shortcuts.layout" => "Layout Management:",
    "shortcuts.cycle_layout" => "Ctrl+L: Cycle Layout Mode",
    "shortcuts.prev_layout" => "Ctrl+Shift+L: Previous Layout",
    "shortcuts.next_pane" => "Ctrl+P: Focus Next Pane",
    "shortcuts.prev_pane_key" => "Ctrl+Shift+P: Focus Previous Pane",
    "shortcuts.cycle_pane_session" => "Ctrl+N: Cycle Session in Pane",
    "shortcuts.general" => "General:",
    "shortcuts.tab" => "Tab: Next Field",
    "shortcuts.shift_tab" => "Shift+Tab: Previous Field",
    "shortcuts.connect" => "O: Connect/Disconnect",
    "shortcuts.clear" => "C: Clear Log",
    "shortcuts.display_mode" => "X: Toggle Display Mode",
    "shortcuts.auto_scroll" => "A: Toggle Auto Scroll",
    "shortcuts.menu" => "F10: Open Menu",
    "shortcuts.quit" => "Ctrl+C / Ctrl+Q: Quit",

    // Empty state messages
    "empty.no_messages" => "No messages yet",
    "empty.connect_hint" => "Connect to start receiving data",
    "empty.shortcuts" => "x - Toggle display | c - Clear | a - Auto scroll",
};

/// Chinese translations
static ZH: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // Menu bar
    "menu.file" => "文件",
    "menu.session" => "会话",
    "menu.view" => "视图",
    "menu.settings" => "设置",
    "menu.help" => "帮助",

    // File menu
    "menu.file.save_config" => "保存配置",
    "menu.file.load_config" => "加载配置",
    "menu.file.exit" => "退出",

    // Session menu
    "menu.session.new" => "新建会话",
    "menu.session.duplicate" => "复制会话",
    "menu.session.rename" => "重命名会话",
    "menu.session.close" => "关闭会话",

    // View menu
    "menu.view.single" => "单视图",
    "menu.view.split_h" => "水平分割",
    "menu.view.split_v" => "垂直分割",
    "menu.view.grid_2x2" => "2×2 网格",
    "menu.view.next_pane" => "下一个窗格",
    "menu.view.prev_pane" => "上一个窗格",

    // Settings menu
    "menu.settings.language" => "语言",
    "menu.settings.toggle_language" => "切换语言",

    // Plugins menu
    "menu.plugins" => "插件",
    "menu.plugins.install" => "从注册表安装",
    "menu.plugins.check_update" => "检查更新",
    "menu.plugins.update_all" => "更新全部",
    "menu.plugins.reload" => "重新加载插件",
    "menu.plugins.list" => "插件列表",
    "menu.plugins.manager" => "插件管理器",

    // Help menu
    "menu.help.shortcuts" => "键盘快捷键",
    "menu.help.about" => "关于",

    // UI labels
    "label.port" => "串口",
    "label.baud_rate" => "波特率",
    "label.data_bits" => "数据位",
    "label.parity" => "校验位",
    "label.stop_bits" => "停止位",
    "label.flow_control" => "流控制",
    "label.display_mode" => "显示模式",
    "label.tx_mode" => "发送模式",
    "label.append_mode" => "追加",
    "label.rx_count" => "接收",
    "label.tx_count" => "发送",
    "label.message" => "消息",
    "label.locked" => "已锁定",
    "label.status" => "状态信息",
    "label.statistics" => "统计信息",
    "label.send" => "发送",
    "label.input_prompt" => "输入数据...",

    // Status
    "status.connected" => "已连接",
    "status.disconnected" => "未连接",
    "status.modifiable" => "可修改",
    "status.locked" => "已锁定",
    "status.not_connected" => "未连接 - 请按 o 打开串口连接",

    // Parity values
    "parity.none" => "无",
    "parity.even" => "偶",
    "parity.odd" => "奇",

    // Flow control values
    "flow.none" => "无",
    "flow.hardware" => "硬件",
    "flow.software" => "软件",

    // Display mode
    "display.hex" => "HEX",
    "display.text" => "TEXT",

    // TX mode
    "tx.hex" => "HEX",
    "tx.ascii" => "ASCII",

    // Append mode
    "append.none" => "无追加",
    "append.lf" => "\\n",
    "append.cr" => "\\r",
    "append.crlf" => "\\r\\n",
    "append.lfcr" => "\\n\\r",

    // Button labels
    "button.connect" => "连接",
    "button.disconnect" => "断开",
    "button.clear" => "清空",
    "button.send" => "发送",

    // Hints
    "hint.select" => "选择",
    "hint.refresh" => "刷新",
    "hint.switch" => "切换",
    "hint.clear" => "清空",
    "hint.toggle" => "切换",
    "hint.scroll" => "滚动浏览",
    "hint.quit" => "退出",
    "hint.exit" => "退出",
    "hint.auto_scroll" => "自动滚动",

    // Notifications
    "notify.config_saved" => "配置已保存",
    "notify.config_loaded" => "配置已加载",
    "notify.config_save_failed" => "保存配置失败",
    "notify.config_load_failed" => "加载配置失败",
    "notify.language_changed" => "语言已切换",
    "notify.connected" => "已连接",
    "notify.disconnected" => "已断开",
    "notify.connection_failed" => "连接失败",
    "notify.config_locked_warning" => "配置已锁定，请先断开连接",
    "notify.port_selected" => "选择串口",
    "notify.send_success" => "已发送",
    "notify.send_failed" => "发送失败",
    "notify.hex_format_error" => "HEX 格式错误",
    "notify.not_connected" => "未连接串口",
    "notify.input_empty" => "输入内容为空",
    "notify.ports_refreshed" => "已刷新串口列表",
    "notify.display_mode" => "显示模式",
    "notify.tx_mode" => "发送模式",
    "notify.append_mode" => "追加",
    "notify.disconnected_unlocked" => "已断开连接，配置已解锁",
    "notify.please_select_port" => "请先选择串口",
    "notify.connected_locked" => "已连接: {} (配置已锁定)",
    "notify.parity" => "校验位",
    "notify.flow_control" => "流控",
    "notify.log_cleared" => "已清空消息记录",
    "notify.toggle_display_mode" => "切换显示模式",
    "notify.auto_scroll" => "自动滚动",
    "notify.enabled" => "启用",
    "notify.disabled" => "禁用",
    "notify.plugin_disabled" => "插件被禁用。请启用 Plugin Feature 以使用插件功能",
    "notify.plugins_loaded" => "已加载 {} 个插件",
    "notify.plugins_reloaded" => "插件已重新加载",
    "notify.plugin_error" => "插件错误",
    "notify.plugin_installed" => "插件 '{}' 已安装 — 请使用重新加载激活",
    "notify.plugin_install_failed" => "安装失败: {}",
    "notify.plugin_checking" => "正在检查更新...",
    "notify.plugin_up_to_date" => "所有插件已是最新",
    "notify.plugin_update_available" => "{} 有更新: {} → {}",
    "notify.plugin_updated" => "{} 已更新",
    "notify.plugin_update_failed" => "更新 {} 失败: {}",
    "notify.plugin_all_updated" => "已更新 {} 个插件",
    "notify.plugin_registry_empty" => "注册表中没有可用插件",
    "notify.plugin_git_missing" => "Git 不可用 — 请安装 git 以管理插件",

    // Plugin modal
    "plugin.modal.title" => "插件管理器",
    "plugin.modal.status" => "状态",
    "plugin.modal.name" => "插件",
    "plugin.modal.hooks" => "钩子",
    "plugin.modal.error" => "错误",
    "plugin.modal.no_plugins" => "未安装插件",
    "plugin.modal.hint_close" => "Esc/Q: 关闭",
    "plugin.modal.hint_reload" => "R: 重新加载",
    "plugin.modal.hint_navigate" => "↑↓: 导航",
    "plugin.modal.hint_install" => "Enter: 安装",
    "plugin.modal.hint_back" => "Esc: 返回",
    "plugin.registry.title" => "插件市场",
    "plugin.registry.search_placeholder" => "输入以搜索...",
    "plugin.registry.hint_search" => "输入搜索",
    "plugin.registry.hint_install" => "Enter: 安装",
    "plugin.registry.hint_back" => "Esc: 返回",
    "plugin.registry.loading" => "正在获取插件列表...",
    "plugin.registry.empty" => "无匹配插件",
    "plugin.registry.installed" => "✓ 已安装",
    "plugin.status.loading" => "加载中…",
    "plugin.status.loaded" => "已加载",
    "plugin.status.error" => "错误",
    "plugin.status.disabled" => "已禁用",
    "plugin.hook.rx" => "RX",
    "plugin.hook.tx" => "TX",
    "plugin.hook.connect" => "CN",
    "plugin.hook.disconnect" => "DC",
    "plugin.hook.none" => "—",

    // Plugin status bar
    "plugin.bar.loaded" => "插件",
    "plugin.bar.none" => "无插件",

    // Help text
    "help.f10" => "F10: 菜单",
    "help.tab" => "Tab: 下一个字段",
    "help.shift_tab" => "Shift+Tab: 上一个字段",
    "help.esc" => "Esc: 取消/关闭",
    "help.enter" => "Enter: 选择/发送",

    // Keyboard shortcuts
    "shortcuts.title" => "键盘快捷键",
    "shortcuts.session" => "会话管理：",
    "shortcuts.new_session" => "Ctrl+T: 新建会话",
    "shortcuts.close_session" => "Ctrl+W: 关闭会话",
    "shortcuts.next_session" => "Ctrl+Tab / Ctrl+→: 下一个会话",
    "shortcuts.prev_session" => "Ctrl+Shift+Tab / Ctrl+←: 上一个会话",
    "shortcuts.switch_1_9" => "Ctrl+1~9: 切换到会话 1~9",
    "shortcuts.layout" => "布局管理：",
    "shortcuts.cycle_layout" => "Ctrl+L: 切换布局模式",
    "shortcuts.prev_layout" => "Ctrl+Shift+L: 上一个布局",
    "shortcuts.next_pane" => "Ctrl+P: 聚焦下一个窗格",
    "shortcuts.prev_pane_key" => "Ctrl+Shift+P: 聚焦上一个窗格",
    "shortcuts.cycle_pane_session" => "Ctrl+N: 切换窗格会话",
    "shortcuts.general" => "常规：",
    "shortcuts.tab" => "Tab: 下一个字段",
    "shortcuts.shift_tab" => "Shift+Tab: 上一个字段",
    "shortcuts.connect" => "O: 连接/断开",
    "shortcuts.clear" => "C: 清空日志",
    "shortcuts.display_mode" => "X: 切换显示模式",
    "shortcuts.auto_scroll" => "A: 切换自动滚动",
    "shortcuts.menu" => "F10: 打开菜单",
    "shortcuts.quit" => "Ctrl+C / Ctrl+Q: 退出",

    // Empty state messages
    "empty.no_messages" => "暂无消息",
    "empty.connect_hint" => "未连接 - 请按 o 打开串口连接",
    "empty.shortcuts" => "x - 切换 HEX/TEXT 显示模式 | c - 清空消息日志 | a - 自动滚动开关",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_exists() {
        assert_eq!(t("menu.file", Language::English), "File");
        assert_eq!(t("menu.file", Language::Chinese), "文件");
    }

    #[test]
    fn test_fallback_to_key() {
        assert_eq!(t("nonexistent.key", Language::English), "nonexistent.key");
    }
}
