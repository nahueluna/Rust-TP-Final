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
                    EstadoDeEleccion::EnCurso => {
                        let res = match estado {
                            EstadoAprobacion::Aprobado => votacion.aprobar_miembro(&id_miembro, &rol),
                            EstadoAprobacion::Rechazado => votacion.rechazar_miembro(&id_miembro, &rol),
                        };
                        self.elecciones.set(id_votacion - 1, votacion); // Necesario ya que no trabajamos con una referencia
                        res
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
        use ink::{
            env::{DefaultEnvironment, Environment},
            primitives::AccountId,
        };

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
                    // Por defecto se superpone el `AccountId` del contrato
                    // con el de alice, por eso esto es necesario.
                    accounts: ink::env::test::DefaultAccounts {
                        alice: <DefaultEnvironment as Environment>::AccountId::from([0xFF; 32]),
                        bob: <DefaultEnvironment as Environment>::AccountId::from([0xFE; 32]),
                        charlie: <DefaultEnvironment as Environment>::AccountId::from([0xFD; 32]),
                        django: <DefaultEnvironment as Environment>::AccountId::from([0xFC; 32]),
                        eve: <DefaultEnvironment as Environment>::AccountId::from([0xFB; 32]),
                        frank: <DefaultEnvironment as Environment>::AccountId::from([0xFA; 32]),
                    },
                }
            }
        }

        impl ContractEnv {
            // Retorna un `ContractEnv` con 4 usuarios registrados en el sistema:
            // Alice, Bob, Charlie y Django.
            // El administrador es de id `contract_id`, no se delegaron los privilegios.
            fn new_inicializado() -> Self {
                let mut env = ContractEnv::default();
                ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);

                // Registrar a Alice
                ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
                env.contract
                    .registrar_usuario(
                        String::from("Alice"),
                        String::from("A"),
                        String::from("11111111"),
                    )
                    .unwrap();

                // Registrar a Bob
                ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
                env.contract
                    .registrar_usuario(
                        String::from("Bob"),
                        String::from("B"),
                        String::from("22222222"),
                    )
                    .unwrap();

                // Registrar a Charlie
                ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
                env.contract
                    .registrar_usuario(
                        String::from("Charlie"),
                        String::from("C"),
                        String::from("33333333"),
                    )
                    .unwrap();

                // Registrar a Django
                ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
                env.contract
                    .registrar_usuario(
                        String::from("Django"),
                        String::from("D"),
                        String::from("44444444"),
                    )
                    .unwrap();

                env
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
                    String::from("22222222")
                )
                .is_ok());

            // El mismo AccountId intenta registrarse de nuevo, no debe poder
            assert_eq!(
                env.contract
                    .registrar_usuario(
                        String::from("Alice"),
                        String::from(""),
                        String::from("11111111")
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
                        String::from("22222222")
                    )
                    .unwrap_err()
                    .to_string(),
                Error::UsuarioExistente.to_string()
            );
        }

        #[ink::test]
        fn probar_es_admin() {
            let env = ContractEnv::default();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // La cuenta invocante es admin
            assert!(env.contract.es_admin());

            // Eve no es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.eve);
            assert!(!env.contract.es_admin());
        }

        #[ink::test]
        fn probar_delegar_admin() {
            let mut env = ContractEnv::default();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // La cuenta que crea el contrato le cede los privilegios a Alice
            assert!(env.contract.delegar_admin(env.accounts.alice).is_ok());

            // Alice prueba si es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert!(env.contract.es_admin());

            // La cuenta que crea el contrato ya no es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert!(!env.contract.es_admin());

            // Alice le cede los privilegios a Bob
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert!(env.contract.delegar_admin(env.accounts.bob).is_ok());

            // Bob prueba si es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            assert!(env.contract.delegar_admin(env.accounts.frank).is_ok());

            // Alice ya no es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert!(!env.contract.es_admin());

            // Eve no puede delegar privilegios porque no es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.eve);
            assert_eq!(
                env.contract
                    .delegar_admin(env.accounts.alice)
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes.to_string()
            );
        }

        #[ink::test]
        fn probar_registro_sistema_admin() {
            let mut env = ContractEnv::default();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            // Alice es ahora admin del sistema
            assert!(env.contract.delegar_admin(env.accounts.alice).is_ok());

            // Alice como invocante del contrato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            // Registrar a Alice en el sistema no debe ser posible
            assert_eq!(
                env.contract
                    .registrar_usuario(
                        String::from("Alice"),
                        String::from(""),
                        String::from("11111111")
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

            // Se crea una elección con exito, de id = 1
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
                    .unwrap(),
                1
            );
            // Se crea una segunda elección con exito, de id = 2
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
                    .unwrap(),
                2
            );

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
                        0,
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

        #[ink::test]
        fn probar_registro_eleccion() {
            // inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // Crear una elección
            let eleccion_id = env
                .contract
                .crear_eleccion(
                    String::from("Presidente"),
                    0,
                    0,
                    1,
                    1,
                    1970,
                    0,
                    1,
                    1,
                    1,
                    1970,
                )
                .unwrap();

            // Establecer el tiempo del bloque en uno válido para registrarse
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Alice se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .is_ok());

            // Bob se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .is_ok());

            // Charlie se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .is_ok());

            // Django se confundió el id de la elección
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(u32::MAX, Rol::Votante)
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste.to_string()
            );

            // Eve se intenta registrar en la elección, pero no existe en el sistema
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.eve);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Votante)
                    .unwrap_err()
                    .to_string(),
                Error::UsuarioNoExistente.to_string()
            );

            // Alice olvidó que ya se había registrado, e intenta volver a hacerlo
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                    .unwrap_err()
                    .to_string(),
                Error::CandidatoExistente.to_string()
            );

            // Charlie olvidó que ya se había registrado, e intenta volver a hacerlo
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Votante)
                    .unwrap_err()
                    .to_string(),
                Error::VotanteExistente.to_string()
            );
        }

        #[ink::test]
        fn probar_registro_eleccion_tiempo() {
            // inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // Crear una elección
            let eleccion_id = env
                .contract
                .crear_eleccion(
                    String::from("Presidente"),
                    // inicio: 31/1/1970 00:00hs
                    0,
                    0,
                    31,
                    1,
                    1970,
                    // fin: 1/2/1970 00:00hs
                    0,
                    0,
                    1,
                    2,
                    1970,
                )
                .unwrap();

            // Establecer el tiempo del bloque en uno previo al período válido
            // 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Alice se registra en la elección como `Rol::Candidato`, pero la elección aún no
            // inició
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoIniciada.to_string()
            );

            // Establecer el tiempo del bloque al mínimo antes del válido
            // 29/01/1970 23:59hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Nuevamente, Alice no puede registrarse
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoIniciada.to_string()
            );

            // Establecer el tiempo al último instante **válido** antes de finalizar
            // 31/01/1970 23:59hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(2678340000);

            // Ahora Alice puede registrarse
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .is_ok());

            // Establecer el tiempo del bloque al primer instante **inválido**, tras finalizar
            // 01/02/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(2678400000);

            // Bob se olvidó de registrarse, y no puede realizarlo porque la elección finalizó
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                    .unwrap_err()
                    .to_string(),
                Error::VotacionFinalizada.to_string()
            );
        }

        #[ink::test]
        fn probar_get_no_verificados() {
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // Crear una elección
            let eleccion_id = env
                .contract
                .crear_eleccion(
                    String::from("Presidente"),
                    0,
                    0,
                    1,
                    1,
                    1970,
                    0,
                    1,
                    1,
                    1,
                    1970,
                )
                .unwrap();

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Como la elección no tiene miembros, retorna un vector vacío
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Candidato)
                    .unwrap(),
                vec![]
            );
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Votante)
                    .unwrap(),
                vec![]
            );

            // Alice se registra como Candidato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Bob se registra como Votante
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // El único candidato no verificado es Alice
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Candidato)
                    .unwrap(),
                vec![env.accounts.alice]
            );

            // El único votante no verificado es Charlie
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Votante)
                    .unwrap(),
                vec![env.accounts.charlie]
            );

            // Eve no puede obtener los miembros de una elección no verificados
            // porque no es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.eve);
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Votante)
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes.to_string()
            );

            // El admin intenta obtener una elección inexistente
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .get_no_verificados(u32::MAX, Rol::Votante)
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste.to_string()
            );
        }

        #[ink::test]
        fn probar_estado_aprobacion() {
            // inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // Crear una elección
            let eleccion_id = env
                .contract
                .crear_eleccion(
                    String::from("Presidente"),
                    0,
                    1,
                    1,
                    1,
                    1970,
                    0,
                    2,
                    1,
                    1,
                    1970,
                )
                .unwrap();

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 01:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(3600000);

            // Alice se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .is_ok());

            // Bob se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .is_ok());

            // Charlie se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .is_ok());

            // Django se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .is_ok());

            // Alice y Bob están pendientes de verificación como Candidatos
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Candidato)
                    .unwrap(),
                vec![env.accounts.alice, env.accounts.bob]
            );

            // Charlie y Django están pendientes de verificación como Votantes
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Votante)
                    .unwrap(),
                vec![env.accounts.charlie, env.accounts.django]
            );

            // Admin aprueba a Alice como Candidato
            assert!(env
                .contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.alice,
                    Rol::Candidato,
                    EstadoAprobacion::Aprobado,
                )
                .is_ok());

            // Admin rechaza a Bob como Candidato
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert!(env
                .contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.bob,
                    Rol::Candidato,
                    EstadoAprobacion::Rechazado,
                )
                .is_ok());

            // Admin aprueba a Charlie como Votante
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert!(env
                .contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.charlie,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .is_ok());

            // Admin rechaza a Django como Votante
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert!(env
                .contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.django,
                    Rol::Votante,
                    EstadoAprobacion::Rechazado,
                )
                .is_ok());

            // Admin no puede aprobar a Alice como Votante, ya que no está registrada en ese rol
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .cambiar_estado_aprobacion(
                        eleccion_id,
                        env.accounts.alice,
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::VotanteNoExistente.to_string()
            );

            // Admin no puede aprobar a Frank porque no está registrado
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .cambiar_estado_aprobacion(
                        eleccion_id,
                        env.accounts.frank,
                        Rol::Candidato,
                        EstadoAprobacion::Rechazado,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::CandidatoNoExistente.to_string()
            );

            // Admin no puede aprobar a Frank porque se confundió el id de elección
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .cambiar_estado_aprobacion(
                        u32::MAX,
                        env.accounts.django,
                        Rol::Votante,
                        EstadoAprobacion::Rechazado,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste.to_string()
            );

            // Eve no puede cambiar el estado de aprobación, porque no es admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.eve);
            assert_eq!(
                env.contract
                    .cambiar_estado_aprobacion(
                        eleccion_id,
                        env.accounts.alice,
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes.to_string(),
            );

            // Establecer el tiempo en uno previo a la elección
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Admin no puede aprobar a Django porque la elección no comenzó
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .cambiar_estado_aprobacion(
                        eleccion_id,
                        env.accounts.django,
                        Rol::Votante,
                        EstadoAprobacion::Rechazado,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoIniciada.to_string()
            );

            // Establecer el tiempo en uno posterior a la elección
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(1700000000);

            // Admin no puede aprobar a Django porque la elección finalizó
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .cambiar_estado_aprobacion(
                        eleccion_id,
                        env.accounts.django,
                        Rol::Votante,
                        EstadoAprobacion::Rechazado,
                    )
                    .unwrap_err()
                    .to_string(),
                Error::VotacionFinalizada.to_string()
            );
        }

        #[ink::test]
        fn probar_get_candidatos() {
            // inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // Crear una elección
            let eleccion_id = env
                .contract
                .crear_eleccion(
                    String::from("Presidente"),
                    0,
                    1,
                    1,
                    1,
                    1970,
                    0,
                    2,
                    1,
                    1,
                    1970,
                )
                .unwrap();

            // Intento pedir los candidatos de una eleccion que no existe
            assert_eq!(env.contract.get_candidatos(u32::MAX),Err(Error::VotacionNoExiste));
            
            // Intento pedir los candidatos de una eleccion sin candidatos  
            assert!(env.contract.get_candidatos(eleccion_id).unwrap().is_empty());

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 01:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(3600000);

            // Alice se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            env.contract.registrar_en_eleccion(eleccion_id, Rol::Candidato).unwrap();

            // Bob se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            env.contract.registrar_en_eleccion(eleccion_id, Rol::Candidato).unwrap();

            // Charlie se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            env.contract.registrar_en_eleccion(eleccion_id, Rol::Candidato).unwrap();

            // Intento pedir los candidatos de una eleccion sin candidatos aprobados
            assert!(env.contract.get_candidatos(eleccion_id).unwrap().is_empty());

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            // Admin aprueba a Alice como Candidato
            env.contract.cambiar_estado_aprobacion(eleccion_id, env.accounts.alice, Rol::Candidato, EstadoAprobacion::Aprobado).unwrap();                

            // Admin rechaza a Bob como Candidato
            env.contract.cambiar_estado_aprobacion(eleccion_id, env.accounts.bob, Rol::Candidato, EstadoAprobacion::Rechazado).unwrap();  

            // Admin aprueba a Charlie como Candidato
            env.contract.cambiar_estado_aprobacion(eleccion_id, env.accounts.charlie, Rol::Candidato, EstadoAprobacion::Aprobado).unwrap();   

            // Pido los candidatos aprobados
            let candidatos = env.contract.get_candidatos(eleccion_id).unwrap();
            let alice = env.accounts.alice;
            let charlie = env.accounts.charlie;
            // Los candidatos deben ser Alice y Charlie ya que son los unicos aprobados
            let response = vec![(alice, env.contract.usuarios.get(alice).unwrap()),
                                                                (charlie, env.contract.usuarios.get(charlie).unwrap())];
            assert_eq!(candidatos,response);
        }
    }
}
