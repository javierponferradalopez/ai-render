//! El ciclo de vida: manda la sesión MCP, no la conversación. `/clear` acaba la
//! conversación y deja viva la sesión, así que la pizarra le sobrevive.
//!
//! Las dos señales de muerte —`SIGINT` primero, EOF en stdin después— se
//! atienden en el hilo del servidor, y es ese hilo el que decide cuándo sale el
//! proceso: con la ventana completamente tapada macOS no ralentiza el event
//! loop, lo **para**, y tapada es el caso normal —el usuario está en su
//! terminal—. El event loop no es un reloj.

use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use crate::viewer::Wire;

/// El adiós: lo justo para que quien estuviera mirando en el segundo monitor se
/// entere de por qué desaparece la ventana. Dejarla en pantalla convertiría lo
/// efímero en una promesa incumplida.
const FAREWELL: Duration = Duration::from_millis(2500);

pub fn the_session_is_over(viewer: &Wire) -> ! {
    if viewer.say_goodbye() {
        sleep(FAREWELL);
    }
    exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewer::wire;

    #[test]
    fn el_adios_dura_entre_dos_y_tres_segundos() {
        assert!((Duration::from_secs(2)..=Duration::from_secs(3)).contains(&FAREWELL));
    }

    #[test]
    fn una_sesion_que_nunca_abrio_ventana_no_tiene_a_quien_despedirse() {
        let (viewer, _commands) = wire();

        assert!(!viewer.say_goodbye());
    }
}
