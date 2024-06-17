#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod administrador;
mod eleccion;
mod fecha;
mod usuario;

#[ink::contract]
mod sistema_votacion {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;

    use crate::administrador::Administrador;
    use crate::eleccion::Eleccion;
    use crate::fecha::Fecha;
    use crate::usuario::Usuario;

    /*
     * Estructura principal del sistema. Consta del administrador electoral,
     * una coleccion de elecciones y la totalidad de usuarios del sistema (solo su info personal)
     */
    #[ink(storage)]
    pub struct SistemaVotacion {
        admin: Administrador,
        elecciones: Vec<Eleccion>,
        usuarios: Mapping<u32, Usuario>,
    }

    impl SistemaVotacion {
        // Creacion del sistema (requiere datos del administrador)
        #[ink(constructor)]
        pub fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
            let admin = Administrador::new(hash, nombre, apellido, dni);
            Self {
                admin,
                elecciones: Vec::new(),
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
            let eleccion = Eleccion::new(id as u32, puesto, inicio, fin);
            self.elecciones.push(eleccion);
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
