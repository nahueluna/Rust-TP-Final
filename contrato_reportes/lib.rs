#![cfg_attr(not(feature = "std"), no_std, no_main)]

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
        #[ink(constructor)]
        pub fn new(contrato_votacion_acc_id: AccountId) -> Self {
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

        #[ink(message)]
        pub fn reporte_votantes(&self, id_eleccion: u32) -> Result<Vec<Usuario>, Error> {
            // no funciona el operador `?`, a veces anda a veces no.
            // entonces a veces tuve que usar match a veces no, idk
            // tampoco puedo implementar std::error::Error, sooo
            match build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("consultar_estado")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<EstadoDeEleccion, Error>>()
                .invoke()
            {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => return Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => return Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => (),
                },
                Err(e) => return Err(e),
            }

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
                        Ok(opt) => opt,
                        Err(e) => panic!("{:?}", e),
                    }
                })
                .collect())
        }

        #[ink(message)]
        pub fn reporte_participacion(&self, id_eleccion: u32) -> Result<(u32, String), Error> {
            match build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("consultar_estado")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<EstadoDeEleccion, Error>>()
                .invoke()
            {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => return Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => return Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => (),
                },
                Err(e) => return Err(e),
            }

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

            let porcentaje = cantidad_de_votantes_que_votaron * 100 / cantidad_de_votantes;

            Ok((cantidad_de_votantes, format!("{}%", porcentaje)))
        }

        #[ink(message)]
        pub fn reporte_resultado(&self, id_eleccion: u32) -> Result<Vec<(u32, Usuario)>, Error> {
            match build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("consultar_estado")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<EstadoDeEleccion, Error>>()
                .invoke()
            {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => return Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => return Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => (),
                },
                Err(e) => return Err(e),
            }

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
                    (
                        c.get_votos(),
                        // recupera info de cada candidato
                        build_call::<DefaultEnvironment>()
                            .call(self.votacion_account_id)
                            .exec_input(
                                ExecutionInput::new(Selector::new(ink::selector_bytes!(
                                    "get_usuarios"
                                )))
                                .push_arg(c.get_account_id()),
                            )
                            .returns::<Result<Usuario, Error>>()
                            .invoke()
                            .unwrap(),
                    )
                })
                .collect::<Vec<(u32, Usuario)>>();

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
        use ink_e2e::{alice, subxt::blocks::ExtrinsicEvents, ContractsBackend, PolkadotConfig};
        use sistema_votacion::{
            eleccion::Rol,
            enums::{EstadoAprobacion, EstadoDeEleccion},
            SistemaVotacion, SistemaVotacionRef,
        };
        use std::io::Write;
        use std::{thread, time};

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn probar_new<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
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

            // Verificar que informa sobre una elección de id inexistente
            assert_eq!(
                client
                    .call(&ink_e2e::bob(), &call_builder.reporte_votantes(u32::MAX))
                    .dry_run()
                    .await?
                    .return_value()
                    .unwrap_err()
                    .to_string(),
                Error::VotacionNoExiste {}.to_string()
            );

            // Crear una elección
            let inicio = Utc::now() + Duration::minutes(10);
            let fin = Utc::now() + Duration::minutes(20);
            let eleccion_id: u32 = client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.crear_eleccion(
                        String::from("Presidente"),
                        inicio.minute().try_into().unwrap(),
                        inicio.hour().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
                        fin.day().try_into().unwrap(),
                        fin.month().try_into().unwrap(),
                        fin.year().try_into().unwrap(),
                    ),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

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
                        0,
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

            // verificar que no es posible acceder a información si el
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

            // asignar permisos
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.delegar_contrato_reportes(reportes_account_id),
                )
                .submit()
                .await?
                .return_value()
                .unwrap();

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
                        inicio.minute().try_into().unwrap(),
                        inicio.hour().try_into().unwrap(),
                        inicio.day().try_into().unwrap(),
                        inicio.month().try_into().unwrap(),
                        inicio.year().try_into().unwrap(),
                        fin.minute().try_into().unwrap(),
                        fin.hour().try_into().unwrap(),
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
            let mut call_builder = contrato_reportes.call_builder::<Reportes>();

            // delegar el id de reportes en el contrato de votación
            client
                .call(
                    &ink_e2e::alice(),
                    &votacion_call_builder.delegar_contrato_reportes(contrato_reportes.account_id),
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
                vec![
                    Usuario::new("Dave".to_string(), "D".to_string(), "33333333".to_string()),
                    Usuario::new(
                        "Ferdie".to_string(),
                        "F".to_string(),
                        "44444444".to_string()
                    )
                ]
            );

            assert_eq!(reporte_participacion, (2, "100%".to_string()));

            assert_eq!(
                reporte_resultado,
                vec![
                    (
                        2,
                        Usuario::new(
                            "Charlie".to_string(),
                            "C".to_string(),
                            "22222222".to_string()
                        )
                    ),
                    (
                        0,
                        Usuario::new("Bob".to_string(), "B".to_string(), "11111111".to_string())
                    )
                ]
            );

            Ok(())
        }
    }
}
