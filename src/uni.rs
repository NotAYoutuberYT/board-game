pub struct MessageUni {
    message: String,
}

impl MessageUni {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}

pub struct MurderUni;

pub enum Uni {
    MessageUni(MessageUni),
    MurderUni(MurderUni)
}
