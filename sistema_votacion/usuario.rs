use ink::prelude::string::String;

/// Información personal del usuario que integra el sistema
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, Clone, PartialEq)]
pub struct Usuario {
    pub nombre: String,
    pub apellido: String,
    pub dni: String,
}

impl Usuario {
    /// Creacion de un usuario con su información personal
    pub fn new(nombre: String, apellido: String, dni: String) -> Self {
        Self {
            nombre,
            apellido,
            dni,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probar_creacion_usuario(){
        let usuario1 = Usuario::new("Carlos".to_string(), "Rodrigues".to_string(), "39_040_417".to_string());
        assert_eq!(usuario1.nombre,"Carlos".to_string());

        let usuario2 = Usuario::new("Julio".to_string(), "Diaz".to_string(), "41_457_167".to_string());
        assert_eq!(usuario2.apellido,"Diaz".to_string());

        let usuario3 = Usuario::new("Carlos".to_string(), "Rodrigues".to_string(), "39_040_417".to_string());
        assert_eq!(usuario3.dni,"39_040_417".to_string());
    }
}