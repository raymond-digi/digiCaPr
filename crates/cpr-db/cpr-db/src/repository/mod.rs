pub mod company;
pub mod config;
pub mod employee;
pub mod employee_log;
pub mod history;
pub mod payroll;
pub mod registry;
pub mod remittance;
pub mod t4;
pub mod vacation;
pub mod ytd;

pub use company::CompanyRepository;
pub use config::ConfigRepository;
pub use employee::EmployeeRepository;
pub use employee_log::EmployeeHistoryRepository;
pub use history::PayrollHistoryRepository;
pub use payroll::PayrollRepository;
pub use registry::RegistryRepository;
pub use remittance::RemittanceRepository;
pub use t4::T4Repository;
pub use vacation::VacationRepository;
pub use ytd::YtdRepository;

pub mod personal_amount;
pub use personal_amount::PersonalAmountRepository;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Main database handle that provides access to all repositories
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Arc::new(Mutex::new(conn)) }
    }

    pub fn employees(&self) -> EmployeeRepository {
        EmployeeRepository::new(Arc::clone(&self.conn))
    }

    pub fn payroll(&self) -> PayrollRepository {
        PayrollRepository::new(Arc::clone(&self.conn))
    }

    pub fn payroll_history(&self) -> PayrollHistoryRepository {
        PayrollHistoryRepository::new(Arc::clone(&self.conn))
    }

    pub fn company(&self) -> CompanyRepository {
        CompanyRepository::new(Arc::clone(&self.conn))
    }

    pub fn history(&self) -> EmployeeHistoryRepository {
        EmployeeHistoryRepository::new(Arc::clone(&self.conn))
    }

    pub fn ytd(&self) -> YtdRepository {
        YtdRepository::new(Arc::clone(&self.conn))
    }

    pub fn t4(&self) -> T4Repository {
        T4Repository::new(Arc::clone(&self.conn))
    }

    pub fn personal_amount(&self) -> PersonalAmountRepository {
        PersonalAmountRepository::new(Arc::clone(&self.conn))
    }

    pub fn vacation(&self) -> VacationRepository {
        VacationRepository::new(Arc::clone(&self.conn))
    }

    pub fn remittance(&self) -> RemittanceRepository {
        RemittanceRepository::new(Arc::clone(&self.conn))
    }

    pub fn config(&self) -> ConfigRepository {
        ConfigRepository::new(Arc::clone(&self.conn))
    }

    pub fn registry(&self) -> RegistryRepository {
        RegistryRepository::new(Arc::clone(&self.conn))
    }
}
