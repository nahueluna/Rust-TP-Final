#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use ink::prelude::vec::Vec;
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

        #[ink(constructor)]
        pub fn new_v2(contrato_votacion_hash: Hash) -> Self {
            use ink::env::call::{build_call, ExecutionInput, Selector};

            let votacion_account_id = build_call::<DefaultEnvironment>()
                .delegate(contrato_votacion_hash)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get_account_id"
                ))))
                .returns::<AccountId>()
                .invoke();

            Self {
                votacion_hash: contrato_votacion_hash,
                votacion_account_id,
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
        pub fn reporte_participacion(&self, id_eleccion: u32) -> Result<(u32, u32), Error> {
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

            Ok((cantidad_de_votantes, porcentaje))
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
            //self.contrato_votacion.get_candidatos(id_eleccion)
            match build_call::<DefaultEnvironment>()
                .call(self.votacion_account_id)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("get_candidatos")))
                        .push_arg(id_eleccion),
                )
                .returns::<Result<Vec<(u32, Usuario)>, Error>>()
                .invoke()
            {
                Ok(mut v) => {
                    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                    Ok(v)
                }
                Err(e) => Err(e),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
