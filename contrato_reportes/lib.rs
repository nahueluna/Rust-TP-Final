#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Este contrato provee métodos para generar reportes en las elecciones del
/// contrato `sitema_votacion`.
/// El contrato de reportes es inmutable, una vez instanciado su estado no cambia.
#[ink::contract]
mod reportes {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use ink::prelude::format;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use sistema_votacion::candidato::Candidato;
    use sistema_votacion::eleccion::Miembro;
    use sistema_votacion::enums::Error;
    use sistema_votacion::enums::EstadoDeEleccion;
    use sistema_votacion::usuario::*;
    use sistema_votacion::votante::Votante;

    #[ink(storage)]
    pub struct Reportes {
        votacion_hash: Hash,
        votacion_account_id: AccountId,
    }

    impl Reportes {
        /// Crea el contrato. Se verifica la validez del contrato de votación.
        #[ink(constructor)]
        pub fn new(contrato_votacion_acc_id: AccountId) -> Self {
            Self::new_interno(contrato_votacion_acc_id)
        }

        fn new_interno(contrato_votacion_acc_id: AccountId) -> Self {
            // constatar que `contrato_votacion_acc_id` es el id de un contrato
            if !ink::env::is_contract::<DefaultEnvironment>(&contrato_votacion_acc_id) {
                panic!(
                    "El account_id {:#?} no es de un contrato",
                    contrato_votacion_acc_id
                );
            }

            // Forma simple de constatar que se comunica con el contrato correcto.
            // Si no lo fuera la ABI difiere y falla la instanciación
            let votacion_hash = build_call::<DefaultEnvironment>()
                .call(AccountId::from(contrato_votacion_acc_id))
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get_hash"
                ))))
                .returns::<Hash>()
                .invoke();

            Self {
                votacion_hash,
                votacion_account_id: contrato_votacion_acc_id,
            }
        }

        fn get_estado_eleccion(&self, id_eleccion: u32) -> Result<(), Error> {
            // no funciona el operador `?`, a veces anda a veces no.
            // entonces a veces tuve que usar match a veces no, idk
            // tampoco puedo implementar std::error::Error, sooo
            match build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("get_estado_eleccion")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<EstadoDeEleccion, Error>>()
                .invoke()
            {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => Ok(()),
                },
                Err(e) => Err(e),
            }
        }

        /// Retorna para una elección de id `id_eleccion` una colección con la
        /// información de los votantes que están aprobados en esa elección,
        /// solo cuando la elección esté finalizada. Si algo falla retorna un `Error`.
        ///
        /// Si bien se consideran los miembros de una elección de carácter público, con
        /// fines de preservar la información personal solo se muestra el nombre y apellido.
        #[ink(message)]
        pub fn reporte_votantes(&self, id_eleccion: u32) -> Result<Vec<String>, Error> {
            self.reporte_votantes_interno(id_eleccion)
        }

        fn reporte_votantes_interno(&self, id_eleccion: u32) -> Result<Vec<String>, Error> {
            match build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("get_estado_eleccion")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<EstadoDeEleccion, Error>>()
                .invoke()
            {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => Ok(()),
                    EstadoDeEleccion::Finalizada => Ok(()),
                },
                Err(e) => Err(e),
            }?;

            let votantes_aprobados = build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "get_votantes_aprobados"
                    )))
                    .push_arg(id_eleccion),
                )
                .returns::<Result<Vec<Votante>, Error>>()
                .invoke()?;

            Ok(votantes_aprobados
                .iter()
                .map(|v| {
                    // Si nada nefasto está sucediendo, esto no debe fallar jamás
                    match build_call::<DefaultEnvironment>()
                        .call(self.votacion_account_id)
                        .exec_input(
                            ExecutionInput::new(Selector::new(ink::selector_bytes!(
                                "get_usuarios"
                            )))
                            .push_arg(v.get_account_id()),
                        )
                        .returns::<Result<Usuario, Error>>()
                        .invoke()
                    {
                        Ok(u) => format!("{} {}", u.nombre, u.apellido),
                        Err(e) => panic!("{:?}", e),
                    }
                })
                .collect())
        }

        /// El reporte de participación retorna para una elección de id `id_elección`
        /// un `Result<(u32, u8), Error>`:
        ///
        /// - El primer campo es la cantidad de votantes
        /// - El segundo campo es el porcentaje de participación, será siempre
        /// un valor entre 0 y 100
        ///
        /// Si bien los candidatos de una elección se consideran de carácter público, con
        /// fines de preservar la información personal solo se muestra el nombre y apellido.
        #[ink(message)]
        pub fn reporte_participacion(&self, id_eleccion: u32) -> Result<(u32, u8), Error> {
            self.reporte_participacion_interno(id_eleccion)
        }

        fn reporte_participacion_interno(&self, id_eleccion: u32) -> Result<(u32, u8), Error> {
            self.get_estado_eleccion(id_eleccion)?;
            let votantes = build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "get_votantes_aprobados"
                    )))
                    .push_arg(id_eleccion),
                )
                .returns::<Result<Vec<Votante>, Error>>()
                .invoke()?;

            let cantidad_de_votantes = votantes.len() as u32;
            let cantidad_de_votantes_que_votaron =
                votantes.iter().fold(0, |acc, v| acc + v.get_votos());

            // Atrapar error de división por cero
            // Si no hay votantes, es seguro asumir que no hay votos
            if cantidad_de_votantes == 0 {
                Ok((0, 0))
            } else {
                // Es seguro hacer esta operación en un `u8`. Es imposible que hayan más
                // votantes que votaron que votantes inscriptos en una elección
                let porcentaje = cantidad_de_votantes_que_votaron * 100 / cantidad_de_votantes;
                Ok((cantidad_de_votantes, porcentaje.try_into().unwrap()))
            }
        }

        /// Reporta el resultado para un elección de id `id_elección`. Retorna un
        /// `Result<Vec<(u32, Usuario)>, Error>`. Para cada elemento del arreglo:
        ///
        /// - El primer campo (`u32`) representa los votos del candidato, cuya información se
        /// encuentra en el siguiente campo
        /// - El segundo campo de tipo `Usuario` es la información del candidato.
        ///
        /// El arreglo se encuentra ordenado de manera descendente en cantidad de votos.
        #[ink(message)]
        pub fn reporte_resultado(&self, id_eleccion: u32) -> Result<Vec<(u32, String)>, Error> {
            self.reporte_resultado_interno(id_eleccion)
        }

        fn reporte_resultado_interno(&self, id_eleccion: u32) -> Result<Vec<(u32, String)>, Error> {
            self.get_estado_eleccion(id_eleccion)?;
            let candidatos = build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("get_candidatos")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<Vec<Candidato>, Error>>()
                .invoke()?;

            let mut resultados = candidatos
                .iter()
                .map(|c| {
                    // Si nada nefasto está sucediendo, esto no debe puede ser error jamás, por eso `unwrap`
                    // recupera info de cada candidato
                    let u = build_call::<DefaultEnvironment>()
                        .call(self.votacion_account_id)
                        .exec_input(
                            ExecutionInput::new(Selector::new(ink::selector_bytes!(
                                "get_usuarios"
                            )))
                            .push_arg(c.get_account_id()),
                        )
                        .returns::<Result<Usuario, Error>>()
                        .invoke()
                        .unwrap();
                    (c.get_votos(), format!("{} {}", u.nombre, u.apellido))
                })
                .collect::<Vec<(u32, String)>>();

            resultados.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

            Ok(resultados)
        }
    }

    /// Con la finalidad de reducir el tiempo que se toman en correr
    /// los tests, solo se harán pruebas de los tres reportes en un mismo método
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use chrono::{Datelike, Duration, Timelike, Utc};
        use ink_e2e::{
            alice, subxt::blocks::ExtrinsicEvents, ContractsBackend, E2EBackend, PolkadotConfig,
        };
        use sistema_votacion::{
            eleccion::Rol,
            enums::{EstadoAprobacion, EstadoDeEleccion},
            SistemaVotacion, SistemaVotacionRef,
        };
        use std::io::Write;
        use std::{thread, time};

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        pub async fn probar_new<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Falló la instanciación del contrato de votación");
            let contrato_votacion_id = contrato_votacion.account_id;

            // Deploy del contrato de reportes, recibe el AccountId del contrato de votación
            let mut constructor_reportes = ReportesRef::new(contrato_votacion_id);
            assert!(client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .is_ok());

            Ok(())
        }

        #[ink_e2e::test]
        async fn probar_id<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de votación");
            let votacion_acc_id = contrato_votacion.account_id;
            let mut votacion_call_builder = contrato_votacion.call_builder::<SistemaVotacion>();

            // Deploy y construción del contrato de reportes
            let mut constructor_reportes = ReportesRef::new(votacion_acc_id);
            let contrato_reportes = client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de reportes");
            let reportes_account_id = contrato_reportes.account_id;
            let mut call_builder = contrato_reportes.call_builder::<Reportes>();

            // Delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder
                        .establecer_contrato_reportes(contrato_reportes.account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Verificar que informa sobre una elección de id inexistente
            assert_eq!(
                client
                    .call(&ink_e2e::alice(), &call_builder.reporte_votantes(u32::MAX))
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste {}.to_string()
            );
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_participacion(u32::MAX)
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste {}.to_string()
            );
            assert_eq!(
                client
                    .call(&ink_e2e::alice(), &call_builder.reporte_resultado(u32::MAX))
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste {}.to_string()
            );

            // Crear una elección finalizada
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
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
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Verificar que reconoce la elección por su id
            assert!(client
                .call(&ink_e2e::bob(), &call_builder.reporte_votantes(eleccion_id))
                .submit()
                .await?
                .return_value()
                .is_ok());

            Ok(())
        }

        #[ink_e2e::test]
        async fn probar_privilegios<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Falló la instanciación del contrato de votación");
            let votacion_acc_id = contrato_votacion.account_id;
            let mut votacion_call_builder = contrato_votacion.call_builder::<SistemaVotacion>();

            // Deploy y construción del contrato de reportes
            let mut constructor_reportes = ReportesRef::new(votacion_acc_id);
            let contrato_reportes = client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .expect("Falló la instanciación del contrato de reportes");
            let reportes_account_id = contrato_reportes.account_id;
            let mut call_builder = contrato_reportes.call_builder::<Reportes>();

            // Crear una elección
            let inicio = Utc::now() + Duration::minutes(1);
            let fin = Utc::now() + Duration::minutes(2);
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
                        String::from("Presidente"),
                        inicio.hour().try_into().unwrap(),
                        inicio.minute().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.day().try_into().unwrap(),
                        fin.month().try_into().unwrap(),
                        fin.year().try_into().unwrap(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Verificar que no es posible acceder a información si el
            // contrato de votacion no tiene asociado el de reportes
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_votantes(eleccion_id)
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes {}.to_string()
            );
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_participacion(eleccion_id)
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes {}.to_string()
            );
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_resultado(eleccion_id)
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::PermisosInsuficientes {}.to_string()
            );

            // Delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder
                        .establecer_contrato_reportes(contrato_reportes.account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Verificar que no se pueden invocar los métodos en esta instancia temporal
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_votantes(eleccion_id),
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoIniciada {}.to_string()
            );
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_participacion(eleccion_id),
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoIniciada {}.to_string()
            );
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_resultado(eleccion_id),
                    )
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoIniciada {}.to_string()
            );

            std::io::stdout().write_all(b"Esperando a que comience la votacion...\n")?;
            loop {
                if client
                    .call(
                        &ink_e2e::alice(),
                        &votacion_call_builder.consultar_estado(eleccion_id),
                    )
                    .submit()
                    .await?
                    .return_value()
                    .unwrap()
                    != EstadoDeEleccion::Pendiente
                {
                    break;
                }
                thread::sleep(time::Duration::from_millis(100));
            }

            // Verificar que es posible invocar el reporte de votantes
            // en esta instancia de tiempo
            assert!(client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_votantes(eleccion_id)
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Asignar permisos
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.establecer_contrato_reportes(reportes_account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Verificar que no es posible invocar reportes de participación y resultados
            // a menos que finalice la eleción
            assert!(client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_participacion(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .is_err());
            assert!(client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_resultado(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .is_err());

            Ok(())
        }

        #[ink_e2e::test]
        async fn reportes<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de votación");
            let mut votacion_call_builder = contrato_votacion.call_builder::<SistemaVotacion>();
            let votacion_acc_id = contrato_votacion.account_id;

            // Registrar a bob
            assert!(client
                .call(
                    &ink_e2e::bob(),
                    &votacion_call_builder.registrar_usuario(
                        "Bob".to_string(),
                        "B".to_string(),
                        "11111111".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Charlie
            assert!(client
                .call(
                    &ink_e2e::charlie(),
                    &votacion_call_builder.registrar_usuario(
                        "Charlie".to_string(),
                        "C".to_string(),
                        "22222222".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Dave
            assert!(client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.registrar_usuario(
                        "Dave".to_string(),
                        "D".to_string(),
                        "33333333".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Ferdie
            assert!(client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.registrar_usuario(
                        "Ferdie".to_string(),
                        "F".to_string(),
                        "44444444".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Crear una elección
            let inicio = Utc::now() + Duration::minutes(1);
            let fin = Utc::now() + Duration::minutes(2);
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
                        String::from("Presidente"),
                        inicio.hour().try_into().unwrap(),
                        inicio.minute().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.day().try_into().unwrap(),
                        fin.month().try_into().unwrap(),
                        fin.year().try_into().unwrap(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Bob como candidato
            client
                .call(
                    &ink_e2e::bob(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Candidato),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Charlie como candidato
            client
                .call(
                    &ink_e2e::charlie(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Candidato),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Dave como Votante
            client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Ferdie como Votante
            client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Aprobar a todos
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Bob),
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Dave),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Deploy y construción del contrato de reportes
            let mut constructor_reportes = ReportesRef::new(votacion_acc_id);
            let contrato_reportes = client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de reportes");
            let call_builder = contrato_reportes.call_builder::<Reportes>();

            // delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder
                        .establecer_contrato_reportes(contrato_reportes.account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Esperar
            std::io::stdout().write_all(b"Esperando a que comience la votacion...\n")?;
            loop {
                if client
                    .call(
                        &ink_e2e::alice(),
                        &votacion_call_builder.consultar_estado(eleccion_id),
                    )
                    .submit()
                    .await?
                    .return_value()
                    .unwrap()
                    != EstadoDeEleccion::Pendiente
                {
                    break;
                }
                thread::sleep(time::Duration::from_millis(100));
            }

            // votar
            client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.votar(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.votar(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Esperar
            std::io::stdout().write_all(b"Esperando a que finalice la votacion...\n")?;
            loop {
                if client
                    .call(
                        &ink_e2e::alice(),
                        &votacion_call_builder.consultar_estado(eleccion_id),
                    )
                    .submit()
                    .await?
                    .return_value()
                    .unwrap()
                    != EstadoDeEleccion::EnCurso
                {
                    break;
                }
                thread::sleep(time::Duration::from_millis(100));
            }

            // Generar reportes
            let reporte_votantes = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_votantes(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            let reporte_participacion = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_participacion(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            let reporte_resultado = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_resultado(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            assert_eq!(
                reporte_votantes,
                vec!["Dave D".to_string(), "Ferdie F".to_string()]
            );

            assert_eq!(reporte_participacion, (2, 100));

            assert_eq!(
                reporte_resultado,
                vec![
                    (2, format!("{} {}", "Charlie", "C")),
                    (0, format!("{} {}", "Bob", "B"))
                ]
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn reportes2<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de votación");
            let mut votacion_call_builder = contrato_votacion.call_builder::<SistemaVotacion>();
            let votacion_acc_id = contrato_votacion.account_id;

            // Registrar a bob
            assert!(client
                .call(
                    &ink_e2e::bob(),
                    &votacion_call_builder.registrar_usuario(
                        "Bob".to_string(),
                        "B".to_string(),
                        "11111111".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Charlie
            assert!(client
                .call(
                    &ink_e2e::charlie(),
                    &votacion_call_builder.registrar_usuario(
                        "Charlie".to_string(),
                        "C".to_string(),
                        "22222222".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Eve
            assert!(client
                .call(
                    &ink_e2e::eve(),
                    &votacion_call_builder.registrar_usuario(
                        "Eve".to_string(),
                        "E".to_string(),
                        "33333333".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Dave
            assert!(client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.registrar_usuario(
                        "Dave".to_string(),
                        "D".to_string(),
                        "44444444".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Ferdie
            assert!(client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.registrar_usuario(
                        "Ferdie".to_string(),
                        "F".to_string(),
                        "55555555".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Crear una elección
            let inicio = Utc::now() + Duration::minutes(1);
            let fin = Utc::now() + Duration::minutes(2);
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
                        String::from("Presidente"),
                        inicio.hour().try_into().unwrap(),
                        inicio.minute().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.day().try_into().unwrap(),
                        fin.month().try_into().unwrap(),
                        fin.year().try_into().unwrap(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Bob como candidato
            client
                .call(
                    &ink_e2e::bob(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Candidato),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Charlie como candidato
            client
                .call(
                    &ink_e2e::charlie(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Candidato),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Eve como Votante
            client
                .call(
                    &ink_e2e::eve(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Dave como Votante
            client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Ferdie como Votante
            client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Aprobar a todos
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Bob),
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Dave),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Eve),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Deploy y construción del contrato de reportes
            let mut constructor_reportes = ReportesRef::new(votacion_acc_id);
            let contrato_reportes = client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de reportes");
            let call_builder = contrato_reportes.call_builder::<Reportes>();

            // delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder
                        .establecer_contrato_reportes(contrato_reportes.account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Esperar
            std::io::stdout().write_all(b"Esperando a que comience la votacion...\n")?;
            loop {
                if client
                    .call(
                        &ink_e2e::alice(),
                        &votacion_call_builder.consultar_estado(eleccion_id),
                    )
                    .submit()
                    .await?
                    .return_value()
                    .unwrap()
                    != EstadoDeEleccion::Pendiente
                {
                    break;
                }
                thread::sleep(time::Duration::from_millis(100));
            }

            // votar
            client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.votar(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::eve(),
                    &votacion_call_builder.votar(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Bob),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.votar(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Esperar
            std::io::stdout().write_all(b"Esperando a que finalice la votacion...\n")?;
            loop {
                if client
                    .call(
                        &ink_e2e::alice(),
                        &votacion_call_builder.consultar_estado(eleccion_id),
                    )
                    .submit()
                    .await?
                    .return_value()
                    .unwrap()
                    != EstadoDeEleccion::EnCurso
                {
                    break;
                }
                thread::sleep(time::Duration::from_millis(100));
            }

            // Generar reportes
            let reporte_votantes = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_votantes(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            let reporte_participacion = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_participacion(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            let reporte_resultado = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_resultado(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            assert_eq!(
                reporte_votantes,
                vec![
                    "Dave D".to_string(),
                    "Eve E".to_string(),
                    "Ferdie F".to_string()
                ]
            );

            assert_eq!(reporte_participacion, (3, 100));

            assert_eq!(
                reporte_resultado,
                vec![
                    (2, format!("{} {}", "Charlie", "C")),
                    (1, format!("{} {}", "Bob", "B"))
                ]
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn reportes_eleccion_vacia<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de votación");
            let mut votacion_call_builder = contrato_votacion.call_builder::<SistemaVotacion>();
            let votacion_acc_id = contrato_votacion.account_id;
            let votacion_hash = client
                .call(&ink_e2e::bob(), &votacion_call_builder.get_hash())
                .submit()
                .await?
                .return_value();

            // Deploy y construción del contrato de reportes
            let mut constructor_reportes = ReportesRef::new(votacion_acc_id);
            let contrato_reportes = client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de reportes");
            let call_builder = contrato_reportes.call_builder::<Reportes>();

            // Delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder
                        .establecer_contrato_reportes(contrato_reportes.account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Crear una elección finalizada y vacía
            let inicio = Utc::now() - Duration::hours(24);
            let fin = Utc::now() - Duration::hours(23);
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
                        String::from("Presidente"),
                        inicio.hour().try_into().unwrap(),
                        inicio.minute().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.day().try_into().unwrap(),
                        fin.month().try_into().unwrap(),
                        fin.year().try_into().unwrap(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Probar reportes en una elección vacía
            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_votantes(eleccion_id),
                    )
                    .dry_run()
                    .await?
                    .return_value(),
                Ok(vec![])
            );

            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_participacion(eleccion_id),
                    )
                    .dry_run()
                    .await?
                    .return_value(),
                Ok((0, 0))
            );

            assert_eq!(
                client
                    .call(
                        &ink_e2e::alice(),
                        &call_builder.reporte_resultado(eleccion_id),
                    )
                    .dry_run()
                    .await?
                    .return_value(),
                Ok(vec![])
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn reportes_eleccion_sin_votos<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // Deploy del contrato de votación
            let mut constructor_votacion = SistemaVotacionRef::new();
            let contrato_votacion = client
                .instantiate(
                    "sistema_votacion",
                    &ink_e2e::alice(),
                    &mut constructor_votacion,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de votación");
            let mut votacion_call_builder = contrato_votacion.call_builder::<SistemaVotacion>();
            let votacion_acc_id = contrato_votacion.account_id;
            let votacion_hash = client
                .call(&ink_e2e::bob(), &votacion_call_builder.get_hash())
                .submit()
                .await?
                .return_value();

            // Registrar a bob
            assert!(client
                .call(
                    &ink_e2e::bob(),
                    &votacion_call_builder.registrar_usuario(
                        "Bob".to_string(),
                        "B".to_string(),
                        "11111111".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Charlie
            assert!(client
                .call(
                    &ink_e2e::charlie(),
                    &votacion_call_builder.registrar_usuario(
                        "Charlie".to_string(),
                        "C".to_string(),
                        "22222222".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Dave
            assert!(client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.registrar_usuario(
                        "Dave".to_string(),
                        "D".to_string(),
                        "33333333".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Registrar a Ferdie
            assert!(client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.registrar_usuario(
                        "Ferdie".to_string(),
                        "F".to_string(),
                        "44444444".to_string(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .is_ok());

            // Crear una elección
            let inicio = Utc::now() + Duration::minutes(1);
            let fin = Utc::now() + Duration::minutes(2);
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
                        String::from("Presidente"),
                        inicio.hour().try_into().unwrap(),
                        inicio.minute().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.day().try_into().unwrap(),
                        fin.month().try_into().unwrap(),
                        fin.year().try_into().unwrap(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Bob como candidato
            client
                .call(
                    &ink_e2e::bob(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Candidato),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Charlie como candidato
            client
                .call(
                    &ink_e2e::charlie(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Candidato),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Dave como Votante
            client
                .call(
                    &ink_e2e::dave(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Registrar en la elección a Ferdie como Votante
            client
                .call(
                    &ink_e2e::ferdie(),
                    &votacion_call_builder.registrar_en_eleccion(eleccion_id, Rol::Votante),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Aprobar a todos
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Bob),
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Charlie),
                        Rol::Candidato,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Dave),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.cambiar_estado_aprobacion(
                        eleccion_id,
                        ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie),
                        Rol::Votante,
                        EstadoAprobacion::Aprobado,
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Deploy y construción del contrato de reportes
            let mut constructor_reportes = ReportesRef::new(votacion_acc_id);
            let contrato_reportes = client
                .instantiate(
                    "contrato_reportes",
                    &ink_e2e::alice(),
                    &mut constructor_reportes,
                )
                .submit()
                .await
                .expect("Fallo la instanciación del contrato de reportes");
            let call_builder = contrato_reportes.call_builder::<Reportes>();

            // Delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder
                        .establecer_contrato_reportes(contrato_reportes.account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            // Esperar
            std::io::stdout().write_all(b"\nEsperando...\n")?;
            loop {
                if client
                    .call(
                        &ink_e2e::alice(),
                        &votacion_call_builder.consultar_estado(eleccion_id),
                    )
                    .submit()
                    .await?
                    .return_value()
                    .unwrap()
                    == EstadoDeEleccion::Finalizada
                {
                    break;
                }
                thread::sleep(time::Duration::from_millis(100));
            }

            // Generar reportes de una elección sin votos
            let reporte_votantes = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_votantes(eleccion_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

            let reporte_participacion = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_participacion(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            let reporte_resultado = client
                .call(
                    &ink_e2e::alice(),
                    &call_builder.reporte_resultado(eleccion_id),
                )
                .dry_run()
                .await?
                .return_value()
                .unwrap();

            assert_eq!(reporte_votantes[0], format!("{} {}", "Dave", "D"),);
            assert_eq!(reporte_votantes[1], format!("{} {}", "Ferdie", "F"),);

            assert_eq!(reporte_participacion, (2, 0));

            assert_eq!(reporte_resultado[0], (0, format!("{} {}", "Bob", "B")),);
            assert_eq!(reporte_resultado[1], (0, format!("{} {}", "Charlie", "C")));

            Ok(())
        }
    }
}
