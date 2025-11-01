pub mod entreprise;
pub mod psk;

use crate::mode::station::auth::psk::Psk;

#[derive(Debug, Default)]
pub struct Auth {
    pub psk: Psk,
}
