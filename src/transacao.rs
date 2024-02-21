use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Transacao {
    #[validate(range(min = 0))]
    pub valor: i32,
    #[validate(custom = "validate_tipo")]
    pub tipo: String,
    #[validate(length(min = 1, max = 10))]
    pub descricao: String,
}

fn validate_tipo(tipo: &str) -> Result<(), ValidationError> {
    match tipo {
        "d" => Ok(()),
        "c" => Ok(()),
        _ => Err(ValidationError::new("tipo inválido")),
    }
}
