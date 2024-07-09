use crate::enums::{Error, EstadoDeEleccion};
use crate::votante::Votante;
use crate::{candidato::Candidato, fecha::Fecha};
use ink::prelude::{string::String, vec::Vec};
use ink::primitives::AccountId;

/// Eleccion:
/// * Identificador
/// * Fechas de inicio y cierre de votación
/// * Vector de `Votante` aprobado y pendiente
/// * Vector de `Candidato` aprobado y pendiente
/// * Puesto por el que se vota en la elección
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Eleccion {
    pub(crate) id: u32,
    pub(crate) votantes_pendientes: Vec<Votante>,
    pub(crate) votantes_aprobados: Vec<Votante>,
    pub(crate) candidatos_pendientes: Vec<Candidato>,
    pub(crate) candidatos_aprobados: Vec<Candidato>,
    puesto: String,
    pub inicio: Fecha,
    pub fin: Fecha,
}

/// Roles posibles de un usuario que se registra en el sistema
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Clone)]
pub enum Rol {
    Candidato,
    Votante,
}

/// Introduce comportamiento común entre miembros de elecciones
pub trait Miembro {
    fn votar(&mut self) -> Result<(), Error>;
    fn get_account_id(&self) -> AccountId;
    fn get_votos(&self) -> u32;
}

impl Eleccion {
    /// Construcción de una elección vacía
    pub(crate) fn new(id: u32, puesto: String, inicio: Fecha, fin: Fecha) -> Self {
        Self {
            id,
            votantes_pendientes: Vec::new(),
            votantes_aprobados: Vec::new(),
            candidatos_pendientes: Vec::new(),
            candidatos_aprobados: Vec::new(),
            puesto,
            inicio,
            fin,
        }
    }

    /// Verifica estado de la eleccion
    ///
    /// * `EstadoDeEleccion::Pendiente` si aún no ha iniciado
    /// * `EstadoDeEleccion::EnCurso` si se encuentra abierta
    /// * `EstadoDeEleccion::Finalizada` si ha terminado
    pub fn consultar_estado(&self, tiempo: u64) -> EstadoDeEleccion {
        if tiempo < self.inicio.get_tiempo_unix() {
            EstadoDeEleccion::Pendiente
        } else if tiempo < self.fin.get_tiempo_unix() {
            EstadoDeEleccion::EnCurso
        } else {
            EstadoDeEleccion::Finalizada
        }
    }

    /// Agrega un usuario a la eleccion, según su `Rol`.
    /// Verifica que la eleccion no se haya iniciado aun.
    pub(crate) fn añadir_miembro(
        &mut self,
        id: AccountId,
        rol: Rol,
        tiempo: u64,
    ) -> Result<(), Error> {
        match self.consultar_estado(tiempo) {
            EstadoDeEleccion::EnCurso => Err(Error::VotacionEnCurso),
            EstadoDeEleccion::Finalizada => Err(Error::VotacionFinalizada),
            EstadoDeEleccion::Pendiente => {
                match rol {
                    Rol::Candidato => {
                        self.candidatos_pendientes.push(Candidato::new(id));
                    }
                    Rol::Votante => {
                        self.votantes_pendientes.push(Votante::new(id));
                    }
                }
                Ok(())
            }
        }
    }

    /// Retorna `Some(usize)` con la posición del usuario pendiente de aprobación o `None` si
    /// este no se encuentra.
    pub fn get_posicion_miembro_pendiente(&self, id: &AccountId, rol: &Rol) -> Option<usize> {
        match rol {
            Rol::Candidato => self
                .candidatos_pendientes
                .iter()
                .position(|c| &c.get_account_id() == id),
            Rol::Votante => self.votantes_pendientes.iter().position(|c| &c.id == id),
        }
    }

    /// Busca un votante o un candidato aprobado con un `AccountId` determinado.
    ///
    /// Retorna `Some(&mut Votante)` o `Some(&mut Candidato)`, respectivamente,
    /// si lo halla. Sino devuelve `None`.
    pub fn buscar_miembro_aprobado(
        &mut self,
        id: &AccountId,
        rol: &Rol,
    ) -> Option<&mut dyn Miembro> {
        match rol {
            Rol::Candidato => {
                if let Some(c) = self
                    .candidatos_aprobados
                    .iter_mut()
                    .find(|v| v.get_account_id() == *id)
                {
                    Some(c)
                } else {
                    None
                }
            }
            Rol::Votante => {
                if let Some(v) = self.votantes_aprobados.iter_mut().find(|v| v.id == *id) {
                    Some(v)
                } else {
                    None
                }
            }
        }
    }

    /// Retorna `true` si el usuario con `AccoundId` especificado existe en la eleccion,
    /// sea `Candidato` o `Votante`
    pub fn existe_usuario(&self, id: &AccountId) -> bool {
        self.votantes_pendientes
            .iter()
            .any(|vot| vot.get_account_id() == *id)
            || self
                .votantes_aprobados
                .iter()
                .any(|vot| vot.get_account_id() == *id)
            || self
                .candidatos_pendientes
                .iter()
                .any(|cand| cand.get_account_id() == *id)
            || self
                .candidatos_aprobados
                .iter()
                .any(|cand| cand.get_account_id() == *id)
    }

    /// Retorna `true` si el usuario con `AccountId` especificado es un miembro
    /// aprobado en la elección, sea `Candidato` o `Votante`
    pub fn existe_miembro_aprobado(&self, id: &AccountId) -> bool {
        self.votantes_aprobados
            .iter()
            .any(|vot| vot.get_account_id() == *id)
            || self
                .candidatos_aprobados
                .iter()
                .any(|cand| cand.get_account_id() == *id)
    }

    /// Dado un `AccoundId` y `Rol`, aprueba al usuario. Retorna `Ok()` si se ha realizado
    /// de forma exitosa o `Error` si el usuario no se ha hallado.
    pub fn aprobar_miembro(&mut self, id: &AccountId, rol: &Rol) -> Result<(), Error> {
        if let Some(pos) = self.get_posicion_miembro_pendiente(id, rol) {
            match rol {
                Rol::Candidato => {
                    let c = self.candidatos_pendientes.remove(pos);
                    self.candidatos_aprobados.push(c);
                    Ok(())
                }
                Rol::Votante => {
                    let v = self.votantes_pendientes.remove(pos);
                    self.votantes_aprobados.push(v);
                    Ok(())
                }
            }
        } else {
            match rol {
                Rol::Candidato => Err(Error::CandidatoNoExistente),
                Rol::Votante => Err(Error::VotanteNoExistente),
            }
        }
    }

    /// Dado un `AccoundId` y `Rol`, rechaza al usuario. Retorna `Ok()` si se ha realizado
    /// de forma exitosa o `Error` si el usuario no se ha hallado.
    pub fn rechazar_miembro(&mut self, id: &AccountId, rol: &Rol) -> Result<(), Error> {
        if let Some(pos) = self.get_posicion_miembro_pendiente(id, rol) {
            match rol {
                Rol::Candidato => {
                    self.candidatos_pendientes.remove(pos);
                    Ok(())
                }
                Rol::Votante => {
                    self.votantes_pendientes.remove(pos);
                    Ok(())
                }
            }
        } else {
            match rol {
                Rol::Candidato => Err(Error::CandidatoNoExistente),
                Rol::Votante => Err(Error::VotanteNoExistente),
            }
        }
    }

    /// Retorna un vector con `AccountId` de los usuarios no verificados según el `Rol` dado.
    pub fn get_no_verificados(&self, rol: &Rol) -> Vec<AccountId> {
        match rol {
            Rol::Votante => self
                .votantes_pendientes
                .iter()
                .map(|v| (v.get_account_id()))
                .collect(),

            Rol::Candidato => self
                .candidatos_pendientes
                .iter()
                .map(|c| c.get_account_id())
                .collect(),
        }
    }

    /// Retorna un vector de `AccountId` de los candidatos aprobados.
    pub fn get_candidatos_verificados(&self) -> Vec<AccountId> {
        self.candidatos_aprobados
            .iter()
            .map(|c| c.get_account_id())
            .collect()
    }

    /// Permite que el votante `id_votante` vote al candidato `id_cantidato`.
    /// Una vez que esto ocurre, el votante no puede volver a votar
    pub fn votar(
        &mut self,
        id_votante: AccountId,
        id_candidato: AccountId,
        tiempo: u64,
    ) -> Result<(), Error> {
        return match self.consultar_estado(tiempo) {
            EstadoDeEleccion::Pendiente => Err(Error::VotacionNoIniciada),
            EstadoDeEleccion::Finalizada => Err(Error::VotacionFinalizada),
            EstadoDeEleccion::EnCurso => {
                // El código está raro con el fin no romper las reglas de ownership
                if self
                    .buscar_miembro_aprobado(&id_candidato, &Rol::Candidato)
                    .is_none()
                {
                    Err(Error::CandidatoNoExistente)
                } else if let Some(votante) =
                    self.buscar_miembro_aprobado(&id_votante, &Rol::Votante)
                {
                    votante.votar().map(|()| {
                        self.buscar_miembro_aprobado(&id_candidato, &Rol::Candidato)
                            .unwrap()
                            .votar()
                    })?
                } else {
                    Err(Error::VotanteNoExistente)
                }
            }
        };
    }
}

mod tests {
    #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
    use crate::{
        eleccion::{Eleccion, Miembro, Rol},
        enums::{Error, EstadoDeEleccion},
        fecha::Fecha,
    };
    use ink::primitives::AccountId;

    #[test]
    fn test_estado_eleccion() {
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00

        let eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);

        assert_eq!(
            eleccion.consultar_estado(1716138000000), // 19/5/2024 17:00:00
            EstadoDeEleccion::Pendiente
        );
        assert_eq!(
            eleccion.consultar_estado(1716163199000), // 19/5/2024 23:59:59
            EstadoDeEleccion::Pendiente
        );
        assert_eq!(
            eleccion.consultar_estado(1716163200000), // 20/5/2024 00:00:00
            EstadoDeEleccion::EnCurso
        );
        assert_eq!(
            eleccion.consultar_estado(1716224400000), // 20/5/2024 17:00:00
            EstadoDeEleccion::EnCurso
        );
        assert_eq!(
            eleccion.consultar_estado(1716249599000), // 20/5/2024 23:59:59
            EstadoDeEleccion::EnCurso
        );
        assert_eq!(
            eleccion.consultar_estado(1716249600000), // 21/5/2024 00:00:00
            EstadoDeEleccion::Finalizada
        );
        assert_eq!(
            eleccion.consultar_estado(1716310800000), // 21/5/2024 17:00:00
            EstadoDeEleccion::Finalizada
        );
    }

    #[test]
    fn test_añadir_miembro_1() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        eleccion
            .añadir_miembro(AccountId::from(miembro_id), Rol::Candidato, 0)
            .unwrap();
        assert!(eleccion.existe_usuario(&AccountId::from(miembro_id)));

        let miembro_id: [u8; 32] = [255; 32];
        eleccion
            .añadir_miembro(AccountId::from(miembro_id), Rol::Votante, 0)
            .unwrap();
        assert!(eleccion.existe_usuario(&AccountId::from(miembro_id)));
    }

    #[test]
    fn test_añadir_miembro_2() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let result =
            eleccion.añadir_miembro(AccountId::from(miembro_id), Rol::Candidato, 63200000);
        match result {
            Ok(_) => (),
            Err(error) => assert_eq!(error, Error::VotacionNoIniciada),
        }

        let miembro_id: [u8; 32] = [255; 32];
        let result = eleccion.añadir_miembro(
            AccountId::from(miembro_id),
            Rol::Candidato,
            648726342763200000,
        );
        match result {
            Ok(_) => (),
            Err(error) => assert_eq!(error, Error::VotacionFinalizada),
        }
    }

    #[test]
    fn test_pos_miembro_pendiente() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        let res = eleccion.get_posicion_miembro_pendiente(&m_id, &Rol::Candidato);
        if let Some(pos) = res {
            assert_eq!(pos, 0_usize)
        }

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Votante, 0).unwrap();
        let res = eleccion.get_posicion_miembro_pendiente(&m_id, &Rol::Votante);
        match res {
            Some(pos) => assert_eq!(pos, 0_usize),
            None => (),
        }
    }

    #[test]
    fn test_aprobacion_de_miembros() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();
        assert!(eleccion
            .buscar_miembro_aprobado(&m_id, &Rol::Candidato)
            .is_some());

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Votante).unwrap();
        assert!(eleccion
            .buscar_miembro_aprobado(&m_id, &Rol::Votante)
            .is_some());
    }

    #[test]
    fn test_aprobacion_de_miembros_2() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        assert!(eleccion
            .buscar_miembro_aprobado(&m_id, &Rol::Candidato)
            .is_none());

        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();
        assert!(eleccion.existe_usuario(&m_id));

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Votante, 0).unwrap();
        assert!(eleccion
            .buscar_miembro_aprobado(&m_id, &Rol::Votante)
            .is_none());
    }

    #[test]
    fn test_aprobacion_de_miembros_3() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        assert!(eleccion.aprobar_miembro(&m_id, &Rol::Candidato).is_err());

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        assert!(eleccion.aprobar_miembro(&m_id, &Rol::Votante).is_err());
    }

    #[test]
    fn test_rechazar_miembros() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        assert!(eleccion.rechazar_miembro(&m_id, &Rol::Candidato).is_ok());

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Votante, 0).unwrap();
        assert!(eleccion.rechazar_miembro(&m_id, &Rol::Votante).is_ok());
    }

    #[test]
    fn test_rechazar_miembros_2() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        assert!(eleccion.rechazar_miembro(&m_id, &Rol::Candidato).is_err());

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        assert!(eleccion.rechazar_miembro(&m_id, &Rol::Votante).is_err());
    }

    #[test]
    fn test_obtener_no_verificados() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        let arr_can = eleccion.get_no_verificados(&Rol::Candidato);
        assert!(!arr_can.is_empty());

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Votante, 0).unwrap();
        let arr_vot = eleccion.get_no_verificados(&Rol::Votante);
        assert!(!arr_vot.is_empty());
    }

    #[test]
    fn test_obtener_miembros_aprobados() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();
        assert!(!eleccion.candidatos_aprobados.is_empty());

        let miembro_id: [u8; 32] = [255; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Votante).unwrap();
        assert!(!eleccion.votantes_aprobados.is_empty());
    }

    #[test]
    fn test_consultar_candidatos_verificados() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);

        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);

        // Antes de registrar miembro
        assert!(!eleccion.existe_miembro_aprobado(&m_id));
        assert!(eleccion.get_candidatos_verificados().first().is_none());

        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();

        // Antes de aprobar miembro
        assert!(!eleccion.existe_miembro_aprobado(&m_id));
        assert!(eleccion.get_candidatos_verificados().first().is_none());

        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();

        // Miembro registrado y aprobado
        assert!(eleccion.existe_miembro_aprobado(&m_id));
        assert_eq!(
            eleccion.get_candidatos_verificados().first().unwrap(),
            &m_id
        );
    }

    #[test]
    fn test_votar() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();

        let miembro_id2: [u8; 32] = [255; 32];
        let m_id2 = AccountId::from(miembro_id2);
        eleccion.añadir_miembro(m_id2, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id2, &Rol::Votante).unwrap();

        assert!(eleccion.votar(m_id2, m_id, 1716163200000).is_ok());
        assert_eq!(eleccion.candidatos_aprobados[0].get_votos(), 1);

        let miembro_id3: [u8; 32] = [1; 32];
        let m_id3 = AccountId::from(miembro_id3);
        eleccion.añadir_miembro(m_id3, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id3, &Rol::Votante).unwrap();

        assert!(eleccion.votar(m_id3, m_id, 1716163200000).is_ok());
        assert_eq!(eleccion.candidatos_aprobados[0].get_votos(), 2);
    }

    #[test]
    fn test_votar_2() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();

        let miembro_id2: [u8; 32] = [255; 32];
        let m_id2 = AccountId::from(miembro_id2);
        eleccion.añadir_miembro(m_id2, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id2, &Rol::Votante).unwrap();

        assert!(eleccion.votar(m_id2, m_id, 16163200000).is_err());
    }

    #[test]
    fn test_votar_3() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();

        let miembro_id2: [u8; 32] = [255; 32];
        let m_id2 = AccountId::from(miembro_id2);
        eleccion.añadir_miembro(m_id2, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id2, &Rol::Votante).unwrap();

        assert!(eleccion.votar(m_id2, m_id, 2837416163200000).is_err());
    }

    #[test]
    fn test_votar_4() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();

        let miembro_id2: [u8; 32] = [255; 32];
        let m_id2 = AccountId::from(miembro_id2);
        eleccion.añadir_miembro(m_id2, Rol::Votante, 0).unwrap();
        eleccion.aprobar_miembro(&m_id2, &Rol::Votante).unwrap();

        assert!(eleccion.votar(m_id2, m_id, 1716163200000).is_err());
    }

    #[test]
    fn test_votar_5() {
        // Creacion
        let id = 1;
        let puesto = "Presidente".to_string();
        let fecha_inicio = Fecha::new(0, 0, 0, 20, 5, 2024); // 20/05/2024 00:00:00
        let fecha_fin = Fecha::new(0, 0, 0, 21, 5, 2024); // 21/05/2024 00:00:00
        let mut eleccion = Eleccion::new(id, puesto, fecha_inicio, fecha_fin);
        // Testeo
        let miembro_id: [u8; 32] = [0; 32];
        let m_id = AccountId::from(miembro_id);
        eleccion.añadir_miembro(m_id, Rol::Candidato, 0).unwrap();
        eleccion.aprobar_miembro(&m_id, &Rol::Candidato).unwrap();

        let miembro_id2: [u8; 32] = [255; 32];
        let m_id2 = AccountId::from(miembro_id2);
        eleccion.añadir_miembro(m_id2, Rol::Votante, 0).unwrap();

        assert!(eleccion.votar(m_id2, m_id, 1716163200000).is_err());
    }
}

//cargo tarpaulin --target-dir src/coverage --skip-clean --exclude-files=target/debug/* --out html
