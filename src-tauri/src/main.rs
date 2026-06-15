// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Pass Chromium flags to WebView2 BEFORE it initializes. These flags
    // fix stutter / high CPU on Windows machines where the GPU stack is
    // virtualized or driver issues cause WebView2 to thrash the renderer
    // (a common issue in VMs, on older Intel iGPUs, and on machines
    // running remote-desktop protocols). Disabling GPU compositing and
    // forcing the legacy software path is a one-line perf fix.
    //
    //   --disable-gpu               skip GPU process; render in main
    //   --disable-gpu-compositing   no GPU-accelerated compositing
    //   --disable-features=...      disable specific Chrome features that
    //                               can cause repaint storms
    //   --disable-frame-rate-limit  don't throttle to 60fps when idle
    //   --disable-renderer-backgrounding
    //                               don't de-prioritize the renderer tab
    #[cfg(target_os = "windows")]
    {
        let existing = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default();
        let extra = "--disable-gpu --disable-gpu-compositing \
                     --disable-features=CalculateNativeWinOcclusion,IntensiveWakeUpThrottling \
                     --disable-renderer-backgrounding \
                     --disable-background-timer-throttling \
                     --disable-backgrounding-occluded-windows";
        let combined = if existing.is_empty() {
            extra.to_string()
        } else {
            format!("{existing} {extra}")
        };
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", combined);
    }

    shellmate_lib::run();
}
