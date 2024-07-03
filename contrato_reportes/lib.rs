#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes {
    use core::ops::Div;

    use ink::prelude::vec::Vec;
    use sistema_votacion::eleccion::Miembro;
    use sistema_votacion::enums::Error;
    use sistema_votacion::enums::EstadoDeEleccion;
    use sistema_votacion::usuario::*;
    use sistema_votacion::SistemaVotacionRef;

    #[ink(storage)]
    pub struct Reportes {
        contrato_votacion: SistemaVotacionRef,
    }

    impl Reportes {
        #[ink(constructor)]
        pub fn new(contrato_votacion: SistemaVotacionRef) -> Self {
            Self { contrato_votacion }
        }

        #[ink(message)]
        pub fn reporte_votantes(&self, id_eleccion: u32) -> Result<Vec<Usuario>, Error> {
            // no funciona el operador `?`, a veces anda a veces no.
            // entonces a veces tuve que usar match a veces no, idk
            // tampoco puedo implementar std::error::Error, sooo
            match self.contrato_votacion.consultar_estado(id_eleccion) {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => return Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => return Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => (),
                },
                Err(e) => return Err(e),
            }

            let votantes_aprobados = self.contrato_votacion.get_votantes_aprobados(id_eleccion)?;
            Ok(votantes_aprobados
                .iter()
                .map(|v| {
                    // Si nada nefasto está sucediendo, esto no debe fallar jamás
                    match self.contrato_votacion.get_usuarios(v.get_account_id()) {
                        Ok(opt) => opt.unwrap(),
                        Err(e) => panic!("{:?}", e),
                    }
                })
                .collect())
        }

        #[ink(message)]
        pub fn reporte_participacion(&self, id_eleccion: u32) -> Result<(u32, u32), Error> {
            match self.contrato_votacion.consultar_estado(id_eleccion) {
                Ok(estado) => match estado {
                    EstadoDeEleccion::Pendiente => return Err(Error::VotacionNoIniciada),
                    EstadoDeEleccion::EnCurso => return Err(Error::VotacionEnCurso),
                    EstadoDeEleccion::Finalizada => (),
                },
                Err(e) => return Err(e),
            }

            let votantes = self.contrato_votacion.get_votantes_aprobados(id_eleccion)?;

            let cantidad_de_votantes = votantes.len() as u32;
            let cantidad_de_votantes_que_votaron =
                votantes.iter().fold(0, |acc, v| acc + v.get_votos());

            let porcentaje = cantidad_de_votantes_que_votaron * 100 / cantidad_de_votantes;

            Ok((cantidad_de_votantes, porcentaje))
        }

        #[ink(message)]
        pub fn reporte_resultado(&self, id_eleccion: u32) {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
