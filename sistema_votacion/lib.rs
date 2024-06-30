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
    use crate::usuario::Usuario;
    use crate::votante::*;
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;
    use ink::storage::StorageVec;

    /// Estructura principal del sistema. Consta del administrador electoral,
    /// una coleccion de elecciones y dos estructuras de usuarios: ID's de usuarios almacenados por DNI
    /// y la información personal de todos los usuarios del sistema almacenada por ID
    #[ink(storage)]
    pub struct SistemaVotacion {
        admin: AccountId,
        elecciones: StorageVec<Eleccion>,
        id_usuarios: Mapping<String, AccountId>,
        usuarios: Mapping<AccountId, Usuario>,
    }

    impl SistemaVotacion {
        /// Creacion del sistema,
        /// toma como admin el `AccountId` de quien crea la instancia del contrato.
        #[ink(constructor)]
        pub fn new() -> Self {
            let admin = Self::env().caller();
            Self {
                admin,
                elecciones: StorageVec::new(),
                id_usuarios: Mapping::new(),
                usuarios: Mapping::new(),
            }
        }

        /// Registra un usuario en el sistema de votacion.
        /// Retorna `Error::UsuarioExistente` si el usuario ya existe.
        #[ink(message)]
        pub fn registrar_usuario(
            &mut self,
            nombre: String,
            apellido: String,
            dni: String,
        ) -> Result<(), Error> {
            let id = self.env().caller();

            match self.es_admin() {
                true => Err(Error::UsuarioNoPermitido),
                false => {
                    if self.usuarios.contains(id) || self.id_usuarios.contains(&dni) {
                        Err(Error::UsuarioExistente)
                    } else {
                        let usuario = Usuario::new(nombre, apellido, dni);
                        self.id_usuarios.insert(usuario.dni.clone(), &id);
                        self.usuarios.insert(id, &usuario);
                        Ok(())
                    }
                }
            }
        }

        /// Registra un votante o un candidato en una votacion determinada.
        ///
        /// Retorna `Error::UsuarioNoExistente` si el usuario no esta registrado.
        /// Retorna `Error::VotanteExistente` si el votante ya existe.
        /// Retorna `Error::CandidatoExistente` si el candidato ya existe.
        /// Retorna `Error::VotacionNoExiste` si la votacion no existe.
        #[ink(message)]
        pub fn registrar_en_eleccion(&mut self, id_votacion: u32, rol: Rol) -> Result<(), Error> {
            let id = self.env().caller();

            if !self.usuarios.contains(id) {
                return Err(Error::UsuarioNoExistente);
            }

            if let Some(votacion) = self.elecciones.get(id_votacion - 1).as_mut() {
                if votacion.existe_usuario(&id) {
                    match rol {
                        Rol::Candidato => Err(Error::CandidatoExistente),
                        Rol::Votante => Err(Error::VotanteExistente),
                    }
                } else {
                    let r = votacion.añadir_miembro(id, rol, self.env().block_timestamp());
                    if r.is_ok() {
                        self.elecciones.set(id_votacion - 1, votacion); // Necesario ya que no trabajamos con una referencia
                    }
                    r
                }
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Permite al administrador crear una eleccion con los datos correspondientes.
        /// Retorna `Error::PermisosInsuficientes` si un usuario intenta acceder.
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

            if inicio.get_tiempo_unix() > fin.get_tiempo_unix() {
                return Err(Error::FechaInvalida);
            }

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
        ///
        /// Solo contendrá **usuarios registrados** que no han sido verificados por el administrador para esa
        /// elección. Éste método no verifica que el usuario exista en el sistema,
        /// esto ocurre cuando el usuario se registra como votante o candidato.
        /// Si el invocante no es administrador retorna `Error:PermisosInsuficientes`
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
                Ok(votacion.get_no_verificados(&rol))
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Permite al administrador aprobar o rechazar un miembro de una eleccion, ya sea un `Votante` o `Candidato`.
        ///
        /// # Retorno
        /// * `Error::PermisosInsuficientes` si un Usuario distinto del administrador intenta acceder.
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
                return match votacion.consultar_estado(self.env().block_timestamp()) {
                    EstadoDeEleccion::Pendiente => Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::Finalizada => Err(Error::VotacionFinalizada),
                    EstadoDeEleccion::EnCurso => match estado {
                        EstadoAprobacion::Aprobado => votacion.aprobar_miembro(&id_miembro, &rol),
                        EstadoAprobacion::Rechazado => votacion.rechazar_miembro(&id_miembro, &rol),
                    },
                };
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Retorna los candidatos aprobados en la elección de id `id_votacion`.
        /// Utiliza el `AccountId` asociado a los candidatos en la elección para buscar los
        /// usuarios registrados en el sistema.
        /// # Panics
        /// Produce panic si el candidato obtenido de la elección
        /// no está registrado en el sistema
        #[ink(message)]
        pub fn get_candidatos(
            &mut self,
            id_votacion: u32,
        ) -> Result<Vec<(AccountId, Usuario)>, Error> {
            if let Some(eleccion) = self.elecciones.get(id_votacion - 1) {
                let candidatos = eleccion
                    .candidatos_aprobados
                    .iter()
                    .map(|c| {
                        let Some(u) = self.usuarios.get(c.id) else {
                            panic!("{}", Error::CandidatoNoExistente);
                        };

                        (c.id, u)
                    })
                    .collect();

                Ok(candidatos)
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Recibe el id de una votacion y retorna su estado actual.
        /// Devuelve `Error::VotacionNoExiste` si la votacion no se halla.
        #[ink(message)]
        pub fn consultar_estado(&self, id_votacion: u32) -> Result<EstadoDeEleccion, Error> {
            if let Some(eleccion) = self.elecciones.get(id_votacion - 1) {
                Ok(eleccion.consultar_estado(self.env().block_timestamp()))
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Le permite a un registrado en el sistema votar por un candidato
        /// `id_candidato` en una elección `id_votacion`, solo si el usuario
        /// invocante está aprobado en la misma.
        #[ink(message)]
        pub fn votar(&self, id_votacion: u32, id_candidato: AccountId) -> Result<(), Error> {
            if let Some(mut eleccion) = self.elecciones.get(id_votacion) {
                eleccion.votar(
                    self.env().caller(),
                    id_candidato,
                    self.env().block_timestamp(),
                )
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Método interno que retorna `true` si el invocante del contrato es un administrador;
        /// `false` en cualquier otro caso
        fn es_admin(&self) -> bool {
            self.env().caller() == self.admin
        }

        /// Retorna `Result<T, E>` con vector de ids e informacion del usuario o `Error` en caso de que la votacion
        /// no exista
        ///
        /// # Panics
        ///
        /// Produce panic si un votante de la elección
        /// no se encuentra registrado en el sistema
        #[ink(message)]
        pub fn get_info_votantes_aprobados(
            &self,
            id_eleccion: u32,
        ) -> Result<Vec<(AccountId, Usuario)>, Error> {
            if let Some(eleccion) = self.elecciones.get(id_eleccion - 1) {
                let votantes = eleccion
                    .votantes_aprobados
                    .iter()
                    .map(|v| {
                        let Some(u) = self.usuarios.get(v.id) else {
                            panic!("Error: {}", Error::UsuarioNoExistente);
                        };

                        (v.id.clone(), u.clone())
                    })
                    .collect();

                Ok(votantes)
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Retorna `Result<T, E>` con vector de `Votante` o `Error` en caso
        /// de que la votacion no exista
        #[ink(message)]
        pub fn get_votantes_aprobados(&self, id_eleccion: u32) -> Result<Vec<Votante>, Error> {
            if let Some(eleccion) = self.elecciones.get(id_eleccion - 1) {
                Ok(eleccion.votantes_aprobados)
            } else {
                Err(Error::VotacionNoExiste)
            }
        }
    }

    impl Default for SistemaVotacion {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::test::set_caller;

        use super::*;

        /* Helpers */

        struct ContractEnv {
            contract: SistemaVotacion,
            contract_id: AccountId,
            accounts: ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment>,
        }

        impl Default for ContractEnv {
            fn default() -> Self {
                Self {
                    contract: SistemaVotacion::new(),
                    contract_id: ink::env::account_id::<ink::env::DefaultEnvironment>(),
                    accounts: ink::env::test::default_accounts::<ink::env::DefaultEnvironment>(),
                }
            }
        }

        /* Tests */

        #[ink::test]
        fn probar_registro_sistema() {
            let mut env = ContractEnv::default();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);

            // Bob como invocante del contrato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);

            // Registrar a bob en el sistema
            assert!(env
                .contract
                .registrar_usuario(
                    String::from("Bob"),
                    String::from(""),
                    String::from("11111111")
                )
                .is_ok());
            // El mismo AccountId intenta registrarse de nuevo, no debe poder
            assert_eq!(
                env.contract
                    .registrar_usuario(
                        String::from("Alice"),
                        String::from(""),
                        String::from("22222222")
                    )
                    .unwrap_err()
                    .to_string(),
                Error::UsuarioExistente.to_string()
            );

            // Eve como invocante del contrato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.eve);
            // El mismo dni intenta registrase de nuevo, no debe poder
            assert_eq!(
                env.contract
                    .registrar_usuario(
                        String::from("Eve"),
                        String::from(""),
                        String::from("11111111")
                    )
                    .unwrap_err()
                    .to_string(),
                Error::UsuarioExistente.to_string()
            );
        }

        #[ink::test]
        fn probar_registro_sistema_admin() {
            let mut env = ContractEnv::default();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);

            // Alice como invocante del contrato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            // Alice es admin del sistema
            assert!(env.contract.delegar_admin(env.accounts.alice).is_ok());
            // Registrar a Alice en el sistema no debe ser posible
            assert_eq!(
                env.contract
                    .registrar_usuario(
                        String::from("Alice"),
                        String::from(""),
                        String::from("22222222")
                    )
                    .unwrap_err()
                    .to_string(),
                Error::UsuarioNoPermitido.to_string()
            );
        }

        #[ink::test]
        fn probar_crear_eleccion() {
            let mut env = ContractEnv::default();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            // Se crea una elección con exito
            assert!(env
                .contract
                .crear_eleccion(
                    String::from("Presidente"),
                    0,
                    0,
                    1,
                    1,
                    1970,
                    1,
                    1,
                    1,
                    1,
                    1970,
                )
                .is_ok());

            // La creación falla porque la fecha de finalización es posterior a la de inicio.
            assert_eq!(
                env.contract
                    .crear_eleccion(
                        String::from("Presidente"),
                        1,
                        1,
                        1,
                        1,
                        1970,
                        0,
                        0,
                        1,
                        1,
                        1970,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::FechaInvalida.to_string()
            );

            // Bob como invocante del contrato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            // Bob no debe poder crear una elección, puesto que no es admin
            assert_eq!(
                env.contract
                    .crear_eleccion(
                        String::from("Presidente"),
                        0,
                        0,
                        1,
                        1,
                        1970,
                        1,
                        1,
                        1,
                        1,
                        1970,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes.to_string()
            );
        }
    }
}
