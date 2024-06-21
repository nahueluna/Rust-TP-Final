#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes {
    use ink::prelude::{string::String, vec::Vec};
    use sistema_votacion::enums::Error;
    use sistema_votacion::reportes::ReporteVotantes;
    use sistema_votacion::usuario::Usuario;
    use sistema_votacion::SistemaVotacionRef;

    #[ink(storage)]
    pub struct Reportes {
        contrato_votacion: SistemaVotacionRef,
    }

    impl Reportes {
        #[ink(constructor)]
        pub fn new(hash_contrato_sistema_votacion: Hash) -> Self {
            Self {
                contrato_votacion: SistemaVotacionRef::new()
                    .code_hash(hash_contrato_sistema_votacion)
                    .endowment(0)
                    .salt_bytes([])
                    .instantiate_v1()
                    .gas_limit(0)
                    .instantiate(),
            }
        }

        #[ink(message)]
        pub fn test(&self, nombre: String, apellido: String) -> Result<Usuario, Error> {
            if nombre == *"error" {
                Err(Error::VotacionNoExiste)
            } else {
                Ok(Usuario::new(nombre, apellido))
            }
        }

        #[ink(message)]
        pub fn reporte_votantes(&self, id_eleccion: u32) -> Result<Vec<ReporteVotantes>, Error> {
            self.contrato_votacion.reporte_votantes(id_eleccion)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
