/// mensagens que o servidor envia para o client
#[derive(Deserialize, Serialize, Debug)]
pub enum ServerMsg {
    // versioninfo deve sempre estar primeiro nessa estrutura
    VersionInfo {}
}

/// mensagens que o client envia para o servidor
#[derive(Deserialize, Serialize, Debug)]
pub enum ClientMsg {
    // versioninfo deve sempre estar primeiro nessa estrutura
    VersionInfo {}
}

/// controlar tipo de mensagem, utilizado em [postbox](super::postbox) e [postoffice](super::postoffice) para controlar threads
pub enum ControlMsg {
    Shutdown
}
