#[derive(Debug)]
pub struct WPAEntreprise {
    pub eap: Eap,
    pub show_password: bool,
}

#[derive(Debug)]
pub enum Eap {
    PWD,
    PEAP,
    PAP,
    TLS,
}
