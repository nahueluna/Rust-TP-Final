#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod eleccion;
mod fecha;
mod usuario;
mod enums;
mod votante;
mod candidato;

#[ink::contract]
mod sistema_votacion {
    use ink::prelude::string::String;
    use ink::storage::{Mapping, StorageVec};
    use crate::eleccion::Eleccion;
    use crate::fecha::Fecha;
    use crate::usuario::Usuario;

    /*
     * Estructura principal del sistema. Consta del administrador electoral,
     * una coleccion de elecciones y la totalidad de usuarios del sistema (solo su info personal)
     */
    #[ink(storage)]
    pub struct SistemaVotacion {
        admin: AccountId,
        elecciones: StorageVec<Eleccion>,
        usuarios: Mapping<u32, Usuario>,
    }

    impl SistemaVotacion {
        // Creacion del sistema, toma el como admin el AccountId de quien crea la instancia del contrato.
        #[ink(constructor)]
        pub fn new() -> Self {
            let admin = Self::env().caller();
            Self {
                admin,
                elecciones: StorageVec::new(),
                usuarios: Mapping::new(),
            }
        }

        // Version muy simplificada. Hay que crear los correspondientes verificadores
        #[ink(message)]
        pub fn crear_eleccion(
            &mut self,
            puesto: String,
            hora_inicio: u8,
            dia_inicio: u8,
            mes_inicio: u8,
            año_inicio: u16,
            hora_fin: u8,
            dia_fin: u8,
            mes_fin: u8,
            año_fin: u16,
        ) {
            let inicio = Fecha::new(0, 0, hora_inicio, dia_inicio, mes_inicio, año_inicio);
            let fin = Fecha::new(0, 0, hora_fin, dia_fin, mes_fin, año_fin);
            let id = self.elecciones.len() + 1; // Reemplazar por un calculo mas sofisticado
            let eleccion = Eleccion::new(id, puesto, inicio, fin);
            self.elecciones.push(&eleccion);
        }
    }

    //Implementacion del trait Default
    impl Default for SistemaVotacion {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
