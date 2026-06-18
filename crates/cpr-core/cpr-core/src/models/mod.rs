pub mod company;
pub mod employee;
pub mod personal_amount;
pub mod payroll;
pub mod vacation;
pub mod history;
pub mod registry;

pub use company::Company;
pub use employee::{Employee, Address, Province, PayType, AutofillType, EmployeeAutofill};
pub use personal_amount::PersonalAmount;
pub use payroll::{Payroll, AdditionalEarning, Deductions, AdditionalDeduction, EarningType, DeductionType, T4BoxType};
pub use vacation::{VacationAccrual, VacationTimeOff, VacationTransactionType, VacationPayType, calculate_vacation_accrual, count_weekdays};
pub use history::{PayRateHistory, EmploymentHistory};
pub use registry::{RegistryValue, RegistryEntry};
