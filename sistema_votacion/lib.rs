#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod candidato;
mod eleccion;
mod enums;
mod fecha;
mod usuario;
mod votante;

#[ink::contract]
mod sistema_votacion {
    use crate::eleccion::Eleccion;
    use crate::eleccion::Rol;
    use crate::enums::Error;
    use crate::fecha::Fecha;
    use crate::usuario::Usuario;
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::{Mapping, StorageVec};

    /// Estructura principal del sistema. Consta del administrador electoral,
    /// una coleccion de elecciones y la totalidad de usuarios del sistema (solo su info personal)
    #[ink(storage)]
    pub struct SistemaVotacion {
        admin: AccountId,
        elecciones: StorageVec<Eleccion>,
        usuarios: Mapping<AccountId, Usuario>,
    }

    impl SistemaVotacion {
        // Creacion del sistema, toma como admin el AccountId de quien crea la instancia del contrato.
        #[ink(constructor)]
        pub fn new() -> Self {
            let admin = Self::env().caller();
            Self {
                admin,
                elecciones: StorageVec::new(),
                usuarios: Mapping::new(),
            }
        }

        #[ink(message)]
        /// Registra un usuario en el sistema de votacion.
        /// Retorna Error::UsuarioExistente si el usuario ya existe.
        pub fn registrar_usuario(&mut self, nombre: String, apellido: String) -> Result<(), Error> {
            let id = self.env().caller();
            if self.usuarios.get(id).is_some() {
                return Err(Error::UsuarioExistente);
            }
            let usuario = Usuario::new(nombre, apellido);
            self.usuarios.insert(id, &usuario);
            Ok(())
        }

        #[ink(message)]
        /// Registra un votante o un candidato en una votacion determinada.
        /// Retorna Error::UsuarioNoExistente si el usuario no esta registrado.
        /// Retorna Error::VotanteExistente si el votante ya existe.
        /// Retorna Error::CandidatoExistente si el candidato ya existe.
        /// Retorna Error::VotacionNoExiste si la votacion no existe.
        pub fn registrar_en_elección(&mut self, id_votacion: u32, rol: Rol) -> Result<(), Error> {
            let id = self.env().caller();

            if self.usuarios.get(id).is_none() {
                return Err(Error::UsuarioNoExistente);
            }

            if let Some(mut votacion) = self.elecciones.get(id_votacion - 1) {
                if votacion.buscar_miembro(&id, &rol).is_some() {
                    match rol {
                        Rol::Candidato => Err(Error::CandidatoExistente),
                        Rol::Votante => Err(Error::VotanteExistente),
                    }
                } else {
                    votacion.añadir_miembro(id, rol);
                    self.elecciones.set(id_votacion - 1, &votacion); //Guardo los cambios
                    Ok(())
                }
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Permite al administrador crear una eleccion con los datos correspondientes.
        /// Retorna Error::PermisosInsuficientes si un usuario intenta acceder.
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
        ) -> Result<u32, Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }
            let inicio = Fecha::new(0, 0, hora_inicio, dia_inicio, mes_inicio, año_inicio);
            let fin = Fecha::new(0, 0, hora_fin, dia_fin, mes_fin, año_fin);
            let id: u32 = self.elecciones.len() + 1;
            let eleccion = Eleccion::new(id, puesto, inicio, fin);
            self.elecciones.push(&eleccion);
            Ok(id)
        }

        /// Permite al administrador ceder sus privilegios a otro usuario cuyo `AccountId` es `id_nuevo_admin`
        /// Si el usuario que le invoca no es administrador retorna `Error::PermisosInsuficientes`
        #[ink(message)]
        pub fn delegar_admin(&mut self, id_nuevo_admin: AccountId) -> Result<(), Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }
            self.admin = id_nuevo_admin;
            Ok(())
        }

        /// Retorna un `Vec<AccountId` de votantes o candidatos, según se corresponda a `rol`, para una elección de id `id_elección`.
        /// Solo contendrá **usuarios registrados** que no han sido verificados por el administrador para esa
        /// elección. Éste método no verifica que el usuario exista en el sistema,
        /// esto ocurre cuando el usuario se registra como votante o candidato.
        /// Si el invocante no es administrador retorna un Error:PermisosInsuficientes
        #[ink(message)]
        pub fn get_no_verificados(
            &self,
            id_elección: u32,
            rol: Rol,
        ) -> Result<Vec<AccountId>, Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }

            if let Some(votacion) = self.elecciones.get(id_elección - 1) {
                Ok(votacion.get_no_verificados(rol))
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Permite al administrador aprobar a un miembro de una eleccion ya sea un Votante o Candidato
        /// Retorna Error::PermisosInsuficientes si un Usuario intenta acceder
        /// Retorna Error::CandidatoYaAprobado si el Candidato ya fue aprobado
        /// Retorna Error::VotanteYaAprobado si el Votante ya fue aprobado
        /// Retorna Error::CandidatoNoExistente si el Candidato no existe
        /// Retorna Error::VotanteNoExistente si el Votante no existe
        /// Retorna Error::VotacionNoExiste si la Eleccion no existe
        #[ink(message)]
        pub fn aprobar(&mut self, id_votacion: u32, id_miembro: AccountId, rol: Rol) -> Result<(), Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }
            
            if let Some(mut votacion) = self.elecciones.get(id_votacion - 1) {
                if let Some(_) = votacion.buscar_miembro(&id_miembro, &rol) {

                    if votacion.esta_aprobado(&id_miembro, &rol) {

                        match rol {
                            Rol::Candidato => Err(Error::CandidatoYaAprobado),
                            Rol::Votante => Err(Error::VotanteYaAprobado),
                        }
                    } else {
                        votacion.aprobar(id_miembro, &rol);
                        self.elecciones.set(votacion.id - 1, &votacion);
                        Ok(())
                    }
                } else {
                    match rol {
                        Rol::Candidato => Err(Error::CandidatoNoExistente),
                        Rol::Votante => Err(Error::VotanteNoExistente),
                    }
                }
            } else {
                Err(Error::VotacionNoExiste)
            }

        }

        /// Método interno que retorna `true` si el invocante del contrato es un administrador;
        /// `false` en cualquier otro caso
        fn es_admin(&self) -> bool {
            self.env().caller() == self.admin
        }
    }

    // Implementacion del trait Default
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
