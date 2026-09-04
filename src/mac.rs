use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSWindow};
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

/// Pone la ventana delante **sin activar la app**, que es lo que deja el
/// teclado donde estaba: donde el usuario lo tenía. *Dock* y *foco* venían en
/// el mismo paquete —subir a `Regular` y `activate()`—, pero son dos llamadas
/// distintas, y ésta es la que sólo mueve la pantalla.
///
/// Sólo agarra sobre una ventana que el sistema ya tiene montada: llamada antes
/// del primer frame, la ventana se queda **detrás** del terminal. Quién espera
/// a ese frame es el Visor.
pub fn bring_the_window_forward() {
    if let Some(window) = the_window() {
        window.orderFrontRegardless();
    }
}

/// La única que hay: el Visor enseña una hoja cada vez.
fn the_window() -> Option<Retained<NSWindow>> {
    application()?.windows().iter().next()
}

fn application() -> Option<Retained<NSApplication>> {
    Some(NSApplication::sharedApplication(MainThreadMarker::new()?))
}
