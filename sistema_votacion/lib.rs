#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sistema_votacion {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;

    /*
     * Estructura principal del sistema. Consta del administrador electoral,
     * una coleccion de elecciones y la totalidad de usuarios del sistema (solo su info personal)
     */
    #[ink(storage)]
    pub struct SistemaVotacion {
        admin: Administrador,
        elecciones: Vec<Eleccion>,
        usuarios: Mapping<u32, Usuario>,
    }

    /*
     * Administrador electoral. Se encarga de crear las elecciones y configurar todos sus apartados.
     */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    struct Administrador {
        hash: String,
        nombre: String,
        apellido: String,
        dni: u32,
    }

    /*
     * Eleccion: identificador, fechas de inicio y cierre.
     * Votantes con id propio y del candidato votado.
     * Candidatos con id propio y cantidad de votos recibidos (preferible que sea HashMap)
     */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    struct Eleccion {
        id: u32,
        votantes: Vec<(u32, Option<u32>)>,
        candidatos: Vec<(u32, u16)>,
        puesto: String,
        inicio: Fecha,
        fin: Fecha,
    }

    /*
     * Informacion general de votantes y candidatos, almacenado en el sistema
     */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    struct Usuario {
        hash: String,
        nombre: String,
        apellido: String,
        dni: u32,
        validado: bool,
    }

    /*
     * Representa una marca de tiempo y su tiempo unix correspondiente
     * */
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Debug)]
    pub struct Fecha {
        segundo: u8,
        minuto: u8,
        hora: u8,
        dia: u8,
        mes: u8,
        año: u16,
        tiempo_unix: u64,
    }

    impl SistemaVotacion {
        // Creacion del sistema (requiere datos del administrador)
        #[ink(constructor)]
        pub fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
            let admin = Administrador::new(hash, nombre, apellido, dni);
            Self {
                admin,
                elecciones: Vec::new(),
                usuarios: Mapping::new(),
            }
        }

        // Version muy simplificada. Hay que crear los correspondientes verificadores
        #[ink(message)]
        pub fn crear_eleccion(&mut self, 
            puesto: String,
            hora_inicio: u8,
            dia_inicio: u8,
            mes_inicio: u8,
            año_inicio: u16,
            hora_fin: u8,
            dia_fin: u8,
            mes_fin: u8,
            año_fin: u16) {
            let inicio = Fecha::new(0, 0, hora_inicio, dia_inicio, mes_inicio, año_inicio);
            let fin = Fecha::new(0, 0, hora_fin, dia_fin, mes_fin, año_fin);
            let id = self.elecciones.len() + 1; // Reemplazar por un calculo mas sofisticado
            let eleccion = Eleccion::new(id as u32, puesto, inicio, fin);
            self.elecciones.push(eleccion);
        }
    }

    impl Administrador {
        // Creacion del administrador con toda la informacion requerida
        fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
            Self {
                hash,
                nombre,
                apellido,
                dni,
            }
        }
    }

    impl Eleccion {
        // Creacion de una eleccion vacia
        fn new(id: u32, puesto: String, inicio: Fecha, fin: Fecha) -> Self {
            Self {
                id,
                votantes: Vec::new(),
                candidatos: Vec::new(),
                puesto,
                inicio,
                fin,
            }
        }
    }

    impl Usuario {
        // Creacion de usuario (votante o candidato)
        fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
            Self {
                hash,
                nombre,
                apellido,
                dni,
                validado: false,
            }
        }
    }

    impl Fecha {
        //Determina si un año es bisiesto o no
        fn es_bisiesto(año: u16) -> bool {
            (año % 4 == 0 && año % 100 != 0) || (año % 400 == 0)
        }
        
        //Determina cuantos dias contiene un mes de un año dado
        fn dias_en_mes(año: u16, mes: u8) -> u8 {
            let mut dias = [31,28,31,30,31,30,31,31,30,31,30,31];
            if Fecha::es_bisiesto(año) {
                dias[1]+=1;
            }
            dias[(mes-1) as usize]
        }
        
        //Determina cuantos dias pasaron desde el 1/1/1970 hasta la fecha recibida
        fn dias_desde_epoch(año: u16, mes: u8, dia: u8) -> u64 {
            let mut dias = 0;
            for a in 1970..año {
                dias += if Fecha::es_bisiesto(a) { 366 } else { 365 };
            }
        
            for m in 1..mes {
                dias += Fecha::dias_en_mes(año, m) as u64;
            }
        
            dias += dia as u64 - 1; 
        
            dias
        }
        
        //Crea una instancia de Fecha 
        pub fn new(segundo:u8, minuto:u8,hora:u8,dia:u8,mes:u8,año: u16) -> Fecha {
            let dias = Fecha::dias_desde_epoch(año, mes, dia);
            let segundos = (hora as u64 * 3600) + (minuto as u64 * 60) + segundo as u64;
      
            let tiempo_unix: u64 = (dias * 86400 + segundos) * 1000;
    
            Fecha {
                segundo,
                minuto,
                hora,
                dia,
                mes,
                año,
                tiempo_unix
            }
        }
    
        //Devuelve el tiempo unix de la fecha
        pub fn get_tiempo_unix(&self) -> u64 {
            self.tiempo_unix
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
