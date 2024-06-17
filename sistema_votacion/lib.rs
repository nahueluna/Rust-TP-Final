#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sistema_votacion {
    use chrono::{DateTime, TimeZone, Utc};
    use ink::prelude::{format, string::String, vec::Vec};
    use ink::storage::Mapping;

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

    /*
     * Administrador electoral. Se encarga de crear las elecciones y configurar todos sus apartados.
     */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    struct Administrador {
        hash: String,
        nombre: String,
        apellido: String,
        dni: u32,
    }

    /*
     * Eleccion: identificador, fechas de inicio y cierre.
     * Votantes con id propio y del candidato votado.
     * Candidatos con id propio y cantidad de votos recibidos (preferible que sea HashMap)
     */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    struct Eleccion {
        id: u32,
        votantes: Vec<(u32, Option<u32>)>,
        candidatos: Vec<(u32, u16)>,
        puesto: String,
        inicio: Fecha,
        fin: Fecha,
    }

    /*
     * Informacion general de votantes y candidatos, almacenado en el sistema
     */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    struct Usuario {
        hash: String,
        nombre: String,
        apellido: String,
        dni: u32,
        validado: bool,
    }

    /*
     * Estructura provisional de fecha, preferible cambiar a crate chrono
     * */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    pub struct Fecha {
        dia: u32,
        mes: u32,
        anno: u32,
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
        pub fn crear_eleccion(&mut self, puesto: String, inicio: Fecha, fin: Fecha) {
            let id = self.elecciones.len() + 1; // Reemplazar por un calculo mas sofisticado
            let eleccion = Eleccion::new(id as u32, puesto, inicio, fin);
            self.elecciones.push(eleccion);
        }
    }

    impl Administrador {
        // Creacion del administrador con toda la informacion requerida
        fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
            Self {
                hash,
                nombre,
                apellido,
                dni,
            }
        }
    }

    impl Eleccion {
        // Creacion de una eleccion vacia
        fn new(id: u32, puesto: String, inicio: Fecha, fin: Fecha) -> Self {
            Self {
                id,
                votantes: Vec::new(),
                candidatos: Vec::new(),
                puesto,
                inicio,
                fin,
            }
        }
    }

    impl Usuario {
        // Creacion de usuario (votante o candidato)
        fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
            Self {
                hash,
                nombre,
                apellido,
                dni,
                validado: false,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
