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

impl Fecha {
    //Determina si un año es bisiesto o no
    fn es_bisiesto(año: u16) -> bool {
        (año % 4 == 0 && año % 100 != 0) || (año % 400 == 0)
    }

    //Determina cuantos dias contiene un mes de un año dado
    fn dias_en_mes(año: u16, mes: u8) -> u8 {
        let mut dias = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if Fecha::es_bisiesto(año) {
            dias[1] += 1;
        }
        dias[(mes - 1) as usize]
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
    pub fn new(segundo: u8, minuto: u8, hora: u8, dia: u8, mes: u8, año: u16) -> Fecha {
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
            tiempo_unix,
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
