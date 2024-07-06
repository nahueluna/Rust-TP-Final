#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::sistema_votacion::SistemaVotacion;
pub use self::sistema_votacion::SistemaVotacionRef;

mod candidato;
pub mod eleccion;
pub mod enums;
mod fecha;
pub mod usuario;
pub mod votante;

#[ink::contract]
mod sistema_votacion {
    use crate::eleccion::Eleccion;
    use crate::eleccion::Miembro;
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
        contrato_reportes: Option<AccountId>,
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
                contrato_reportes: Option::None,
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
        /// Retorna `Error::MiembroExistente` si el usuario ya esta registrado en la votacion.
        /// Retorna `Error::VotacionNoExiste` si la votacion no existe.
        #[ink(message)]
        pub fn registrar_en_eleccion(&mut self, id_votacion: u32, rol: Rol) -> Result<(), Error> {
            let id = self.env().caller();

            if !self.usuarios.contains(id) {
                return Err(Error::UsuarioNoExistente);
            }

            if let Some(votacion) = self.elecciones.get(id_votacion - 1).as_mut() {
                if votacion.existe_usuario(&id) {
                    Err(Error::MiembroExistente)
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
        ) -> Result<Vec<(AccountId, Usuario)>, Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }

            if let Some(eleccion) = self.elecciones.get(id_elección - 1) {
                let id_miembros = eleccion.get_no_verificados(&rol);
                
                let miembros = id_miembros.iter().map(|id| {
                    let Some(u) = self.usuarios.get(id) else {
                        panic!("{:?}", Error::UsuarioNoExistente);
                    };

                    (id.clone(), u.clone())
                }).collect();

                Ok(miembros)
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
                    EstadoDeEleccion::EnCurso => Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => Err(Error::VotacionFinalizada),
                    EstadoDeEleccion::Pendiente => {
                        let res = match estado {
                            EstadoAprobacion::Aprobado => {
                                votacion.aprobar_miembro(&id_miembro, &rol)
                            }
                            EstadoAprobacion::Rechazado => {
                                votacion.rechazar_miembro(&id_miembro, &rol)
                            }
                        };
                        self.elecciones.set(id_votacion - 1, votacion); // Necesario ya que no trabajamos con una referencia
                        res
                    }
                };
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// REPORTES
        /// Retorna los candidatos aprobados en la elección de id `id_votacion` asociados a su voto.
        /// Utiliza el `AccountId` asociado a los candidatos en la elección para buscar los
        /// usuarios registrados en el sistema.
        /// Verifica el estado de la elección y si el invocante es el contrato de reportes
        #[ink(message)]
        pub fn get_candidatos(&self, id_votacion: u32) -> Result<Vec<(u32, Usuario)>, Error> {
            if !self.es_contrato_reportes() {
                Err(Error::PermisosInsuficientes)
            } else if let Some(eleccion) = self.elecciones.get(id_votacion - 1) {
                match eleccion.consultar_estado(self.env().block_timestamp()) {
                    EstadoDeEleccion::Pendiente => Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => Ok(()),
                }?;

                let candidatos = eleccion.candidatos_aprobados;
                let votos_candidatos = candidatos
                    .iter()
                    .map(|c| {
                        (
                            c.get_votos(),
                            self.usuarios.get(c.get_account_id()).unwrap(),
                        )
                    })
                    .collect();

                Ok(votos_candidatos)
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
        pub fn votar(&mut self, id_votacion: u32, id_candidato: AccountId) -> Result<(), Error> {
            if let Some(mut eleccion) = self.elecciones.get(id_votacion - 1) {
                eleccion.votar(
                    self.env().caller(),
                    id_candidato,
                    self.env().block_timestamp(),
                )?;
                self.elecciones.set(id_votacion - 1, &eleccion);
                Ok(())
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// Método interno que retorna `true` si el invocante del contrato es un administrador;
        /// `false` en cualquier otro caso
        fn es_admin(&self) -> bool {
            self.env().caller() == self.admin
        }

        /// Método interno que retorna `true` si el invocante del contrato es el
        /// contrato de reportes
        /// `false` en cualquier otro caso
        fn es_contrato_reportes(&self) -> bool {
            match self.contrato_reportes {
                Some(c) => self.env().caller() == c,
                None => false,
            }
        }

        /// Retorna `Result<T, E>` con vector de ids e informacion del usuario.
        /// Si la votacion no existe devuelve `Error::VotacionNoExiste`.
        /// Si el invocante no es el admin devuelve `Error::PermisosInsuficientes`.
        ///
        /// # Panics
        ///
        /// Produce panic si un votante de la elección
        /// no se encuentra registrado en el sistema.
        #[ink(message)]
        pub fn get_info_votantes_aprobados(
            &self,
            id_eleccion: u32,
        ) -> Result<Vec<(AccountId, Usuario)>, Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }
            if let Some(eleccion) = self.elecciones.get(id_eleccion - 1) {
                let votantes = eleccion
                    .votantes_aprobados
                    .iter()
                    .map(|v| {
                        let Some(u) = self.usuarios.get(v.id) else {
                            panic!("{:?}", Error::UsuarioNoExistente);
                        };

                        (v.id.clone(), u.clone())
                    })
                    .collect();

                Ok(votantes)
            } else {
                Err(Error::VotacionNoExiste)
            }
        }

        /// REPORTES
        /// Retorna `Result<T, E>` con vector de `Votante` o `Error` en caso
        /// de que la votacion no exista o que quien le invoca no sea el
        ///contrato de reportes
        #[ink(message)]
        pub fn get_votantes_aprobados(&self, id_eleccion: u32) -> Result<Vec<Votante>, Error> {
            if self.es_contrato_reportes() {
                if let Some(eleccion) = self.elecciones.get(id_eleccion - 1) {
                    Ok(eleccion.votantes_aprobados)
                } else {
                    Err(Error::VotacionNoExiste)
                }
            } else {
                Err(Error::PermisosInsuficientes)
            }
        }

        /// REPORTES
        // Obtener un usuario cuyo AccountId es `account_id`
        // Devuelve `Err(Error::PermisosInsuficientes)` si el invocante no
        // es el contrato de reportes
        // Devuelve Error::UsuarioNoExistente si no existe el usuario
        // de AccountId `account_id`
        #[ink(message)]
        pub fn get_usuarios(&self, account_id: AccountId) -> Result<Usuario, Error> {
            if self.es_contrato_reportes() {
                if let Some(id) = self.usuarios.get(account_id) {
                    Ok(id)
                } else {
                    Err(Error::UsuarioNoExistente)
                }
            } else {
                Err(Error::PermisosInsuficientes)
            }
        }

        /// Permite al administrador establecer el AccountId del contrato que podrá acceder
        /// a una serie de métodos que obtienen información de una elección
        #[ink(message)]
        pub fn delegar_contrato_reportes(&mut self, account_id: AccountId) -> Result<(), Error> {
            if !self.es_admin() {
                return Err(Error::PermisosInsuficientes);
            }
            self.contrato_reportes = Some(account_id);
            Ok(()) //exitoso
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
            // (No está registrada eve, se utiliza para testear permisos)
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
                    1,
                    2,
                    2,
                    1970,
                    0,
                    2,
                    2,
                    2,
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
                Error::MiembroExistente.to_string()
            );

            // Charlie olvidó que ya se había registrado, e intenta volver a hacerlo
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Votante)
                    .unwrap_err()
                    .to_string(),
                Error::MiembroExistente.to_string()
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

            // Establecer el tiempo del bloque en uno previo al inicio
            // 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Alice se registra en la elección como `Rol::Candidato`.
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert!(env
                .contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .is_ok());

            // Establecer el tiempo al último instante antes del inicio de la eleccion.
            // 31/01/1970 23:59hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(2678340000);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            // Ahora Charlie no puede registrarse
            assert_eq!(
                env.contract
                    .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                    .unwrap_err()
                    .to_string(),
                Error::VotacionEnCurso.to_string()
            );

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
                    2,
                    2,
                    1970,
                    0,
                    1,
                    2,
                    2,
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
                    .unwrap().first().unwrap().0,
                env.accounts.alice
            );

            // El único votante no verificado es Charlie
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Votante)
                    .unwrap().first().unwrap().0,
                env.accounts.charlie
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

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 00:00hs
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
                    .unwrap().iter().map(|m| m.0).collect::<Vec<_>>(),
                vec![env.accounts.alice, env.accounts.bob]
            );

            // Charlie y Django están pendientes de verificación como Votantes
            assert_eq!(
                env.contract
                    .get_no_verificados(eleccion_id, Rol::Votante)
                    .unwrap().iter().map(|m| m.0).collect::<Vec<_>>(),
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

            // Establecer el tiempo para que la eleccion este en curso
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(5400000);

            // Admin no puede aprobar a Django porque la elección esta en curso
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
                Error::VotacionEnCurso.to_string()
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
                    2,
                    2,
                    1970,
                    0,
                    2,
                    2,
                    2,
                    1970,
                )
                .unwrap();

            // establecer con fines de pruebas el id del contrato reportes igual al administrador
            env.contract
                .delegar_contrato_reportes(env.contract_id)
                .unwrap();

            // Intento pedir los candidatos de una eleccion que no existe
            assert_eq!(
                env.contract.get_candidatos(u32::MAX),
                Err(Error::VotacionNoExiste)
            );

            // Intento pedir los candidatos de una eleccion sin candidatos
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(99999999999);
            assert!(env.contract.get_candidatos(eleccion_id).unwrap().is_empty());

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Intento pedir los candidatos antes de que inicie la eleccion
            assert_eq!(
                env.contract.get_candidatos(eleccion_id),
                Err(Error::VotacionNoIniciada)
            );
            
            // Alice se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Bob se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Charlie se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Intento pedir los candidatos mientras la eleccion esta en curso
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(2770200000);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract.get_candidatos(eleccion_id),
                Err(Error::VotacionEnCurso)
            );

            // Intento pedir los candidatos de una eleccion sin candidatos aprobados
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(99999999999);
            assert!(env.contract.get_candidatos(eleccion_id).unwrap().is_empty());
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Admin aprueba a Alice como Candidato
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.alice,
                    Rol::Candidato,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Admin rechaza a Bob como Candidato
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.bob,
                    Rol::Candidato,
                    EstadoAprobacion::Rechazado,
                )
                .unwrap();

            // Admin aprueba a Charlie como Candidato
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.charlie,
                    Rol::Candidato,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Pido los candidatos aprobados
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(99999999999);
            let candidatos = env.contract.get_candidatos(eleccion_id).unwrap();
            let alice = env.accounts.alice;
            let charlie = env.accounts.charlie;
            // Los candidatos deben ser Alice y Charlie ya que son los unicos aprobados
            let response = vec![
                (0, env.contract.usuarios.get(alice).unwrap()),
                (0, env.contract.usuarios.get(charlie).unwrap()),
            ];
            assert_eq!(candidatos, response);
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Django se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Admin aprueba a Django como Votante
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.django,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Establecer el tiempo del bloque en uno válido para votar, 02/02/1970 00:11hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(2768460000);

            // Django vota a Alice
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            //assert!(
            env.contract.votar(eleccion_id, env.accounts.alice).unwrap();

            // Establecer el tiempo del bloque en uno en que la elección haya finalizado
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(9999999999);

            // Ahora alice tiene un voto
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            assert_eq!(
                env.contract.get_candidatos(eleccion_id).unwrap(),
                vec![
                    (1, env.contract.usuarios.get(alice).unwrap()),
                    (0, env.contract.usuarios.get(charlie).unwrap()),
                ]
            );
        }

        #[ink::test]
        fn probar_consultar_estado() {
            // inicializar sistema con usuarios registrados
            let env = ContractEnv::new_inicializado();

            // Probar consultar el estado de una eleccion que no existe
            assert_eq!(
                env.contract.consultar_estado(u32::MAX),
                Err(Error::VotacionNoExiste)
            );

            // Los otros casos de consultar_estado() ya fueron cubiertos en los tests anteriores
        }

        #[ink::test]
        fn probar_votar() {
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
                    2,
                    2,
                    1970,
                    0,
                    2,
                    2,
                    2,
                    1970,
                )
                .unwrap();

            // establecer con fines de pruebas el id del contrato reportes igual al administrador
            env.contract
                .delegar_contrato_reportes(env.contract_id)
                .unwrap();

            // Intento votar en una eleccion que no existe
            assert_eq!(
                env.contract.votar(u32::MAX,env.accounts.bob),
                Err(Error::VotacionNoExiste)
            );

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);
            
            // Alice se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Bob se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Charlie se registra en la elección como `Rol::Candidato`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Candidato)
                .unwrap();

            // Django se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            // Admin aprueba a Alice como Candidato
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.alice,
                    Rol::Candidato,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Admin rechaza a Bob como Candidato
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.bob,
                    Rol::Candidato,
                    EstadoAprobacion::Rechazado,
                )
                .unwrap();

            // Admin aprueba a Django como Votante
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.django,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Establecer el tiempo del bloque en uno válido para votar, 02/02/1970 00:11hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(2768460000);

            // Intento votar en una eleccion con un votante invalido
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.frank);
            assert_eq!(
                env.contract.votar(eleccion_id,env.accounts.alice),
                Err(Error::VotanteNoExistente)
            );

            // Intento votar a un candidato invalido en una eleccion 
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            assert_eq!(
                env.contract.votar(eleccion_id,env.accounts.frank),
                Err(Error::CandidatoNoExistente)
            );

            // Django vota a Alice
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            env.contract.votar(eleccion_id, env.accounts.alice).unwrap();

            // Django intenta volver a votar a Alice
            assert_eq!(
                env.contract.votar(eleccion_id,env.accounts.alice),
                Err(Error::VotanteYaVoto)
            );

            // Establecer el tiempo del bloque en uno en que la elección haya finalizado
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(9999999999);

            // Pido los candidatos aprobados
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            let candidatos = env.contract.get_candidatos(eleccion_id).unwrap();
            let alice = env.accounts.alice;
            // La candidata debe ser Alice ya que es la unica aprobada
            let response = vec![
                (1, env.contract.usuarios.get(alice).unwrap()),
            ];
            assert_eq!(candidatos, response);
        }

        #[ink::test]
        fn probar_es_contrato_reportes() {
            // Inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();

            // Probar el valor por defecto de contrato_reportes
            assert_eq!(env.contract.es_contrato_reportes(),false);

            // Cambio el AccountId de contrato_reportes
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract.admin);
            env.contract.delegar_contrato_reportes(env.accounts.bob).unwrap();

            // Llamo al metodo con Alice
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            assert_eq!(env.contract.es_contrato_reportes(),false);
            
            // Llamo al metodo con bob
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            assert_eq!(env.contract.es_contrato_reportes(),true);
        }

        #[ink::test]
        fn probar_get_info_votantes_aprobados() {
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
                    2,
                    2,
                    1970,
                    0,
                    2,
                    2,
                    2,
                    1970,
                )
                .unwrap();

            // establecer con fines de pruebas el id del contrato reportes igual al administrador
            env.contract
                .delegar_contrato_reportes(env.contract_id)
                .unwrap();

            // Intento llamar al metodo con una eleccion que no existe
            assert_eq!(
                env.contract.get_info_votantes_aprobados(u32::MAX),
                Err(Error::VotacionNoExiste)
            );

            // Llamo al metodo con una eleccion sin votantes
            assert!(env.contract.get_info_votantes_aprobados(eleccion_id).unwrap().is_empty());

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Alice se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Bob se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Charlie se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Django se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Django intenta llamar al metodo
            assert_eq!(
                env.contract.get_info_votantes_aprobados(u32::MAX),
                Err(Error::PermisosInsuficientes)
            );

            // Admin aprueba a Alice como Votante
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.alice,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Admin rechaza a Bob como Votante
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.bob,
                    Rol::Votante,
                    EstadoAprobacion::Rechazado,
                )
                .unwrap();

            // Admin aprueba a Charlie como Votante
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.charlie,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Llamo al metodo correctamente
            let info_votantes = env.contract.get_info_votantes_aprobados(eleccion_id).unwrap();
            let alice = env.accounts.alice;
            let charlie = env.accounts.charlie;
            // Los votantes deben ser Alice y Charlie ya que son los unicos aprobados
            let response = vec![
                (alice, env.contract.usuarios.get(alice).unwrap()),
                (charlie, env.contract.usuarios.get(charlie).unwrap()),
            ];
            assert_eq!(info_votantes, response);
        }

        #[ink::test]
        fn probar_get_votantes_aprobados() {
            // Inicializar sistema con usuarios registrados
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
                    2,
                    2,
                    1970,
                    0,
                    2,
                    2,
                    2,
                    1970,
                )
                .unwrap();

            // Establecer con fines de pruebas el id del contrato reportes igual al administrador
            env.contract
                .delegar_contrato_reportes(env.contract_id)
                .unwrap();

            // Intento llamar al metodo con una eleccion que no existe
            assert_eq!(
                env.contract.get_votantes_aprobados(u32::MAX),
                Err(Error::VotacionNoExiste)
            );

            // Llamo al metodo con una eleccion sin votantes
            assert!(env.contract.get_votantes_aprobados(eleccion_id).unwrap().is_empty());

            // Establecer el tiempo del bloque en uno válido para registrarse, 01/01/1970 00:00hs
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(0);

            // Alice se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.alice);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Bob se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.bob);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Charlie se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.charlie);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Django se registra en la elección como `Rol::Votante`
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            env.contract
                .registrar_en_eleccion(eleccion_id, Rol::Votante)
                .unwrap();

            // Django intenta llamar al metodo
            assert_eq!(
                env.contract.get_votantes_aprobados(u32::MAX),
                Err(Error::PermisosInsuficientes)
            );

            // Admin aprueba a Alice como Votante
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.alice,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Admin rechaza a Bob como Votante
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.bob,
                    Rol::Votante,
                    EstadoAprobacion::Rechazado,
                )
                .unwrap();

            // Admin aprueba a Charlie como Votante
            env.contract
                .cambiar_estado_aprobacion(
                    eleccion_id,
                    env.accounts.charlie,
                    Rol::Votante,
                    EstadoAprobacion::Aprobado,
                )
                .unwrap();

            // Llamo al metodo correctamente
            let info_votantes = env.contract.get_votantes_aprobados(eleccion_id).unwrap();
            // Los votantes deben ser Alice y Charlie ya que son los unicos aprobados
            let response = env.contract.elecciones.get(eleccion_id -1).unwrap().votantes_aprobados;
            assert_eq!(info_votantes, response);
        }

        #[ink::test]
        fn probar_get_usuarios() {
            // Inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);

            // Establecer con fines de pruebas el id del contrato reportes igual al administrador
            env.contract
                .delegar_contrato_reportes(env.contract_id)
                .unwrap();

            // Intento llamar al metodo con un usuario que no existe
            assert_eq!(
                env.contract.get_usuarios(AccountId::from([9; 32])),
                Err(Error::UsuarioNoExistente)
            );

            // Django intenta llamar al metodo
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            assert_eq!(
                env.contract.get_usuarios(env.accounts.alice),
                Err(Error::PermisosInsuficientes)
            );

            // Llamo al metodo correctamente
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            let alice_id = env.accounts.alice;
            let alice = env.contract.usuarios.get(alice_id).unwrap();
            assert_eq!(env.contract.get_usuarios(alice_id).unwrap(),alice);
            let charlie_id = env.accounts.charlie;
            let charlie = env.contract.usuarios.get(charlie_id).unwrap();
            assert_eq!(env.contract.get_usuarios(charlie_id).unwrap(),charlie);
        }

        #[ink::test]
        fn probar_delegar_contrato_reportes() {
            // Inicializar sistema con usuarios registrados
            let mut env = ContractEnv::new_inicializado();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(env.contract_id);
            
            // Django intenta llamar al metodo
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.accounts.django);
            assert_eq!(
                env.contract.delegar_contrato_reportes(env.accounts.alice),
                Err(Error::PermisosInsuficientes)
            );
            
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(env.contract_id);
            // Llamo al metodo correctamente
            env.contract
                .delegar_contrato_reportes(env.contract_id)
                .unwrap();
        }
    }
}
