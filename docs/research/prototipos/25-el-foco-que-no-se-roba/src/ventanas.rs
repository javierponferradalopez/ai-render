//! La sonda del orden Z: quién está delante de quién, leído del WindowServer y
//! no de lo que la app cree de sí misma.
//!
//! `CGWindowListCopyWindowInfo` se puede llamar desde cualquier hilo, así que la
//! contesta el hilo del servidor —el único que sabe qué hora es— sin depender
//! del que dibuja, que macOS congela en cuanto la ventana queda tapada.

use objc2::msg_send;
use objc2::runtime::AnyObject;
use objc2_core_foundation::{CFArray, CFString};
use objc2_core_graphics::{
    CGWindowListCopyWindowInfo, CGWindowListOption, kCGWindowBounds, kCGWindowLayer,
    kCGWindowOwnerName, kCGWindowOwnerPID,
};
use objc2_foundation::{NSArray, NSDictionary, NSString};

/// El nivel de las ventanas normales. Los paneles del sistema —barra de menús,
/// Dock, cursores— viven por encima y no compiten por estar delante.
const NIVEL_NORMAL: i64 = 0;

#[derive(Debug, Clone)]
pub struct Ventana {
    pub pid: i32,
    pub owner: String,
    pub bounds: (f64, f64, f64, f64),
}

/// Las ventanas de nivel normal que hay en pantalla, **de delante hacia
/// atrás**. Es el orden que el WindowServer usa para decidir qué tapa qué.
pub fn de_delante_hacia_atras() -> Vec<Ventana> {
    let Some(info) = CGWindowListCopyWindowInfo(
        CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements,
        0,
    ) else {
        return Vec::new();
    };
    // CFArray y NSArray son la misma cosa con dos nombres (toll-free bridged),
    // y las claves de esta lista son `CFString` constantes de CoreGraphics.
    let lista: &NSArray<NSDictionary<NSString, AnyObject>> =
        unsafe { &*(&*info as *const CFArray as *const NSArray<NSDictionary<NSString, AnyObject>>) };

    lista
        .iter()
        .filter(|ventana| entero(ventana, unsafe { kCGWindowLayer }) == Some(NIVEL_NORMAL))
        .filter_map(|ventana| {
            Some(Ventana {
                pid: entero(&ventana, unsafe { kCGWindowOwnerPID })? as i32,
                owner: texto(&ventana, unsafe { kCGWindowOwnerName })
                    .unwrap_or_else(|| "?".to_string()),
                bounds: rectangulo(&ventana)?,
            })
        })
        .collect()
}

type Diccionario = NSDictionary<NSString, AnyObject>;

fn valor(ventana: &Diccionario, clave: &CFString) -> Option<objc2::rc::Retained<AnyObject>> {
    let clave: &NSString = unsafe { &*(clave as *const CFString as *const NSString) };
    ventana.objectForKey(clave)
}

fn entero(ventana: &Diccionario, clave: &CFString) -> Option<i64> {
    let numero = valor(ventana, clave)?;
    Some(unsafe { msg_send![&*numero, longLongValue] })
}

fn texto(ventana: &Diccionario, clave: &CFString) -> Option<String> {
    let cadena = valor(ventana, clave)?;
    let cadena: &NSString = unsafe { &*(&*cadena as *const AnyObject as *const NSString) };
    Some(cadena.to_string())
}

/// `kCGWindowBounds` no es un número sino un `CGRect` empaquetado en otro
/// diccionario, con las claves en llano.
fn rectangulo(ventana: &Diccionario) -> Option<(f64, f64, f64, f64)> {
    let bounds = valor(ventana, unsafe { kCGWindowBounds })?;
    let bounds: &Diccionario = unsafe { &*(&*bounds as *const AnyObject as *const Diccionario) };
    let lado = |nombre: &str| -> f64 {
        match bounds.objectForKey(&NSString::from_str(nombre)) {
            Some(numero) => unsafe { msg_send![&*numero, doubleValue] },
            None => 0.0,
        }
    };
    Some((lado("X"), lado("Y"), lado("Width"), lado("Height")))
}
