#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::sistema_votacion::SistemaVotacion;
pub use self::sistema_votacion::SistemaVotacionRef;

mod candidato;
mod eleccion;
pub mod enums;
mod fecha;
pub mod reportes;
pub mod usuario;
mod votante;

#[ink::contract]
mod sistema_votacion {
    use crate::eleccion::Eleccion;
    use crate::eleccion::Rol;
    use crate::enums::*;
    use crate::fecha::Fecha;
    use crate::reportes::ReporteVotantes;
    use crate::usuario::Usuario;
    use ink::prelude::{borrow::ToOwned, string::String, vec::Vec};
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
        /// Retorna `Error::UsuarioNoExistente` si el usuario no esta registrado.
        /// Retorna `Error::VotanteExistente` si el votante ya existe.
        /// Retorna `Error::CandidatoExistente` si el candidato ya existe.
        /// Retorna `Error::VotacionNoExiste` si la votacion no existe.
        pub fn registrar_en_eleccion(&mut self, id_votacion: u32, rol: Rol) -> Result<(), Error> {
            let id = self.env().caller();

            if self.usuarios.get(id).is_none() {
                return Err(Error::UsuarioNoExistente);
            }

            if let Some(votacion) = self.elecciones.get(id_votacion - 1).as_mut() {
                if votacion.existe_usuario(&id) {
                    match rol {
                        Rol::Candidato => Err(Error::CandidatoExistente),
                        Rol::Votante => Err(Error::VotanteExistente),
                    }
                } else {
                    votacion.añadir_miembro(id, rol);
                    self.elecciones.set(id_votacion - 1, votacion); // Necesario ya que no trabajamos con una referencia
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
            minuto_inicio: u8,
            hora_inicio: u8,
            dia_inicio: u8,
            mes_inicio: u8,
            año_inicio: u16,
            minuto_fin: u8,
            hora_fin: u8,
            dia_fin: u8,
            mes_fin: u8,
            año_fin: u16,
        ) -> Result<u32, Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }
            let inicio = Fecha::new(
                0,
                minuto_inicio,
                hora_inicio,
                dia_inicio,
                mes_inicio,
                año_inicio,
            );
            let fin = Fecha::new(0, minuto_fin, hora_fin, dia_fin, mes_fin, año_fin);
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

        /// Permite al administrador aprobar o rechazar un miembro de una eleccion, ya sea un `Votante` o `Candidato`.
        ///
        /// # Retorno
        /// * `Error::PermisosInsuficientes` si un Usuario distinto del administrador intenta acceder.
        /// * `Error::CandidatoYaAprobado` si el Candidato ya fue aprobado.
        /// * `Error::VotanteYaAprobado` si el Votante ya fue aprobado.
        /// * `Error::CandidatoYaRechazado` si el Candidato ya fue rechazado.
        /// * `Error::VotanteYaRechazado` si el Votante ya fue rechazado.
        /// * `Error::CandidatoNoExistente` si el Candidato no existe.
        /// * `Error::VotanteNoExistente` si el Votante no existe.
        /// * `Error::VotacionNoExiste` si la Eleccion no existe.
        #[ink(message)]
        pub fn cambiar_estado_aprobacion(
            &mut self,
            id_votacion: u32,
            id_miembro: AccountId,
            rol: Rol,
            estado: EstadoAprobacion,
        ) -> Result<(), Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }

            if let Some(votacion) = self.elecciones.get(id_votacion - 1).as_mut() {
                if let Some(u) = votacion.buscar_miembro(&id_miembro, &rol) {
                    if u.esta_aprobado() && estado == EstadoAprobacion::Aprobado {
                        match rol {
                            Rol::Candidato => Err(Error::CandidatoYaAprobado),
                            Rol::Votante => Err(Error::VotanteYaAprobado),
                        }
                    } else if u.esta_rechazado() && estado == EstadoAprobacion::Rechazado {
                        match rol {
                            Rol::Candidato => Err(Error::CandidatoYaRechazado),
                            Rol::Votante => Err(Error::VotanteYaRechazado),
                        }
                    } else {
                        u.cambiar_estado_aprobacion(estado);
                        self.elecciones.set(id_votacion - 1, votacion); // Necesario ya que no trabajamos con una referencia
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

        /// Retorna los candidatos aprobados en la elección de id `id_votacion`
        /// Utiliza el `AccountId` asociado a los candidatos en la elección para buscar los
        /// usuarios registrados en el sistema.
        #[ink(message)]
        pub fn get_candidatos(
            &mut self,
            id_votacion: u32,
        ) -> Result<Vec<(AccountId, Usuario)>, Error> {
            if let Some(eleccion) = self.elecciones.get(id_votacion - 1) {
                // Utiliza `unwrap()` ya que si el método `get_candidatos` de una elección
                // retorna un id inválido, algo MUY MALO HA PASADO, y debería finalizar la
                // ejecución.
                Ok(eleccion
                    .get_miembros(&Rol::Candidato)
                    .iter()
                    .map(|id| (*id, self.usuarios.get(id).unwrap()))
                    .collect())
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        #[ink(message)]
        /// Recibe el id de una votacion y retorna su estado actual.
        /// Devuelve un error si la votacion no existe.
        pub fn consultar_estado(&self, id_votacion: u32) -> Result<EstadoDeEleccion, Error> {
            let tiempo = self.env().block_timestamp();
            if let Some(eleccion) = self.elecciones.get(id_votacion - 1) {
                if tiempo < eleccion.inicio.get_tiempo_unix() {
                    return Ok(EstadoDeEleccion::Pendiente);
                } else if tiempo < eleccion.fin.get_tiempo_unix() {
                    return Ok(EstadoDeEleccion::EnCurso);
                }
                Ok(EstadoDeEleccion::Finalizada)
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        // Le permite a un registrado en el sistema votar por un candidato
        // `id_candidato` en una elección `id_votacion`, solo si el usuario
        // invocante está aprobado en la misma.
        #[ink(message)]
        pub fn votar(&self, id_votacion: u32, id_candidato: AccountId) -> Result<(), Error> {
            if let Some(mut eleccion) = self.elecciones.get(id_votacion) {
                eleccion.votar(self.env().caller(), id_candidato)
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Método interno que retorna `true` si el invocante del contrato es un administrador;
        /// `false` en cualquier otro caso
        fn es_admin(&self) -> bool {
            self.env().caller() == self.admin
        }

        // devuelve los votantes registrados y aprobados en una elección de id `id_eleccion`
        #[ink(message)]
        pub fn reporte_votantes(&self, id_eleccion: u32) -> Result<Vec<ReporteVotantes>, Error> {
            if let Some(eleccion) = self.elecciones.get(id_eleccion - 1) {
                Ok(eleccion
                    .get_miembros(&Rol::Votante)
                    .iter()
                    .map(|id| {
                        let u = self.usuarios.get(id).unwrap();
                        ReporteVotantes::new(id.to_owned(), u.nombre, u.apellido)
                    })
                    .collect())
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        #[ink(message)]
        pub fn get_hash_contrato(&self) -> Hash {
            self.env().own_code_hash().unwrap()
        }
    }

    impl Default for SistemaVotacion {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::hash::{Blake2x256, CryptoHash};

        #[ink::test]
        fn probar() {
            let dato = b"probando";
            let mut output = [0u8; 32];
            Blake2x256::hash(dato, &mut output);
            let hash = Hash::from(output);
            let contrato = SistemaVotacion::new();
        }
    }
}
