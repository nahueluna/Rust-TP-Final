#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::reportes::{Reportes, ReportesRef};

#[ink::contract]
mod reportes {
    #[ink(storage)]
    pub struct Reportes {}

    impl Reportes {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {}
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn reporte_votantes(&mut self) {
            todo!()
        }

        #[ink(message)]
        pub fn reporte_participacion(&mut self) {
            todo!()
        }

        #[ink(message)]
        pub fn reporte_resultado(&mut self) {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
