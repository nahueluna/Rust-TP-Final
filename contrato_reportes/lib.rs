#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes {
    use sistema_votacion::SistemaVotacionRef;

    #[ink(storage)]
    pub struct Reportes {
        contrato_votacion: SistemaVotacionRef,
    }

    impl Reportes {
        #[ink(constructor)]
        pub fn new(hash_contrato_sistema_votacion: Hash) -> Self {
            Self {
                contrato_votacion: SistemaVotacionRef::new()
                    .code_hash(hash_contrato_sistema_votacion)
                    .endowment(0)
                    .salt_bytes([])
                    .instantiate_v1()
                    .gas_limit(0)
                    .instantiate(),
            }
        }

        #[ink(message)]
        pub fn reporte_votantes(&self) {
            todo!()
        }

        #[ink(message)]
        pub fn reporte_participacion(&self) {
            todo!()
        }

        #[ink(message)]
        pub fn reporte_resultado(&self) {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
