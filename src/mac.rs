use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::{NSActivityOptions, NSObjectProtocol, NSProcessInfo, NSString};

const USER_INITIATED_AND_LATENCY_CRITICAL: u64 = 0x00FF_FFFF | (1 << 20) | 0xFF_0000_0000;

pub fn keep_awake_while_the_session_lasts() -> Retained<ProtocolObject<dyn NSObjectProtocol>> {
    NSProcessInfo::processInfo().beginActivityWithOptions_reason(
        NSActivityOptions(USER_INITIATED_AND_LATENCY_CRITICAL),
        &NSString::from_str("flipchart serves MCP for as long as the session lasts"),
    )
}

pub fn stay_out_of_the_dock(main_thread: MainThreadMarker) {
    NSApplication::sharedApplication(main_thread)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

/// Trae la app al frente. Sólo agarra sobre una app que ya es `Regular`; quien
/// la sube es el event loop al nacer, y eso es el primer `show`.
pub fn take_the_focus() {
    if let Some(app) = application() {
        app.activate();
    }
}

fn application() -> Option<Retained<NSApplication>> {
    Some(NSApplication::sharedApplication(MainThreadMarker::new()?))
}
