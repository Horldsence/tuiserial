//! Data pipeline — routes serial RX/TX data through the plugin chain.
//!
//! Each plugin with the corresponding hook (`onRx` / `onTx`) can inspect,
//! modify, or suppress the data flowing through the pipeline.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

use tuiserial_core::{NotificationLevel, SerialConfig};

use crate::types::PluginResult;

use super::{PluginManager, extract_panic_message};

impl PluginManager {
    /// Process received data through all plugins with onRx hooks.
    ///
    /// Each plugin's onRx is called in order. If a plugin returns
    /// `Modified`, the modified data is passed to the next plugin.
    /// If a plugin returns `Suppressed`, processing stops and the
    /// data is dropped.
    ///
    /// Returns `(final_data, suppressed)`.
    pub fn process_rx(&mut self, data: Vec<u8>, config: &SerialConfig) -> (Vec<u8>, bool) {
        self.process_pipeline("onRx", data, config)
    }

    /// Process outgoing data through all plugins with onTx hooks.
    ///
    /// Same pipeline semantics as `process_rx`.
    pub fn process_tx(&mut self, data: Vec<u8>, config: &SerialConfig) -> (Vec<u8>, bool) {
        self.process_pipeline("onTx", data, config)
    }

    /// Internal pipeline runner with graded error degradation.
    ///
    /// # Error behaviour
    ///
    /// | Consecutive errors | Action |
    /// |---|---|
    /// | 1–2 | 5 s backoff (`disabled_until`), then auto re-enable |
    /// | ≥ 3 | Permanent disable (`has_error = true`) |
    /// | Panic | Immediate permanent disable |
    ///
    /// A **successful** call resets `consecutive_errors` to 0.
    fn process_pipeline(
        &mut self,
        hook_name: &str,
        mut data: Vec<u8>,
        config: &SerialConfig,
    ) -> (Vec<u8>, bool) {
        let now = Instant::now();

        for plugin in &mut self.plugins {
            let has_hook = match hook_name {
                "onRx" => plugin.hooks.on_rx,
                "onTx" => plugin.hooks.on_tx,
                _ => false,
            };

            // Permanently disabled or doesn't have this hook → skip.
            if plugin.has_error || !has_hook {
                continue;
            }

            // Temporarily backed off?
            if let Some(until) = plugin.disabled_until {
                if now < until {
                    continue;
                }
                // Backoff expired — re-enable.
                plugin.disabled_until = None;
            }

            plugin.update_config(config);

            match catch_unwind(AssertUnwindSafe(|| plugin.call_data_hook(hook_name, &data))) {
                Ok(PluginResult::PassThrough) => {
                    // Success resets the consecutive-error counter.
                    plugin.consecutive_errors = 0;
                }
                Ok(PluginResult::Modified(new_data)) => {
                    plugin.consecutive_errors = 0;
                    data = new_data;
                }
                Ok(PluginResult::Suppressed) => {
                    plugin.consecutive_errors = 0;
                    return (data, true);
                }
                Ok(PluginResult::Error(msg)) => {
                    plugin.error_count += 1;
                    plugin.consecutive_errors += 1;
                    plugin.record_error(msg.clone());

                    let max = crate::runtime::MAX_CONSECUTIVE_ERRORS;
                    let backoff =
                        std::time::Duration::from_secs(crate::runtime::ERROR_BACKOFF_SECS);

                    if plugin.consecutive_errors >= max {
                        plugin.has_error = true;
                        plugin.error_message = Some(msg.clone());
                        plugin.append_log(
                            NotificationLevel::Error,
                            format!(
                                "[plugin: {}] permanently disabled after {max} errors: {msg}",
                                plugin.name,
                            ),
                        );
                    } else {
                        plugin.disabled_until = Some(now + backoff);
                        plugin.append_log(
                            NotificationLevel::Warning,
                            format!(
                                "[plugin: {}] error ({} of {max}), backing off {}s: {msg}",
                                plugin.name,
                                plugin.consecutive_errors,
                                backoff.as_secs(),
                            ),
                        );
                    }
                }
                Err(panic_info) => {
                    let msg = extract_panic_message(&panic_info);
                    plugin.error_count += 1;
                    plugin.consecutive_errors += 1;
                    plugin.has_error = true;
                    plugin.error_message = Some(format!("panic in {hook_name}: {msg}"));
                    plugin.record_error(format!("panic in {hook_name}: {msg}"));
                    plugin.append_log(
                        NotificationLevel::Error,
                        format!("[plugin: {}] panic in {}: {}", plugin.name, hook_name, msg,),
                    );
                }
            }
        }

        (data, false)
    }
}
