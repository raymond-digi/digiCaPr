use super::{EmployeeProvider, PayrollDeductions, TaxYearConfig, YtdProvider};
use crate::models::payroll::{AdditionalDeduction, DeductionType};
use crate::models::Province;
use chrono::{Datelike, NaiveDate};
/// T4127 Payroll Deductions Formula Implementation
///
/// This module implements the exact CRA T4127 formulas using a struct that holds
/// all variables as defined in Table 3.1 Glossary. Variables are named exactly
/// as they appear in the T4127 document.
///
/// # Usage
///
/// ```rust,ignore
/// use cpr_core::tax::T4127Context;
/// use cpr_core::models::Province;
/// use rust_decimal_macros::dec;
///
/// let mut context = T4127Context::new(2026)?;
///
/// // Set input variables (public fields)
/// context.P = 12; // Monthly pay
/// context.I = dec!(6000); // Periodic income
/// context.B = dec!(0); // No bonus
/// context.PI = dec!(6000); // Pensionable income
/// context.IE = dec!(6000); // Insurable income
/// context.TC = dec!(16452); // Federal personal amount
/// context.TCP = dec!(12989); // Provincial personal amount (ON)
/// context.employee_age = 35;
/// context.province = Province::ON;
///
/// // Calculate all deductions
/// let deductions = context.calc();
/// ```
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// T4127 Tax Calculation Context
///
/// This struct holds all variables from the T4127 Table 3.1 Glossary.
/// Public fields represent inputs that must be set before calling calc().
/// Private fields are computed during calculation.
///
/// Note: Variable names intentionally match T4127 documentation (non-snake case)
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct T4127Context {
    // ===== PUBLIC INPUT FIELDS (Set by caller before calc()) =====
    /// P - Number of pay periods in the year
    pub P: i32,

    /// Whether the employee is exempt from CPP contributions
    pub cpp_exempt: bool,

    /// Whether the employee is exempt from EI premiums
    pub ei_exempt: bool,

    /// I - Gross remuneration for the pay period (periodic income)
    /// Includes overtime, pension, qualified pension, taxable benefits
    /// Does NOT include bonuses or retroactive pay (those go in B)
    pub I: Decimal,

    /// B - Gross bonus, retroactive pay increase, vacation pay when vacation not taken,
    /// accumulated overtime payment or other non-periodic payment
    pub B: Decimal,

    /// B1 - Year-to-date non-periodic payments (before the current pay period)
    pub B1: Decimal,

    /// PI - Pensionable earnings for the pay period
    /// Gross income plus taxable benefits, including bonuses and retroactive pay
    pub PI: Decimal,

    /// IE - Insurable earnings for the pay period
    /// Including insurable taxable benefits, bonuses, and retroactive pay
    pub IE: Decimal,

    /// TC - Total claim amount (federal Form TD1)
    /// If not filed, use BPAF. For non-residents, TC = 0
    pub TC: Decimal,

    /// TCP - Total claim amount (provincial/territorial Form TD1)
    /// If not filed, use provincial basic personal amount
    pub TCP: Decimal,

    /// F - Payroll deductions for registered pension plan (RPP), RRSP, PRPP, or RCA
    pub F: Decimal,

    /// F1 - Annual deductions (childcare, support payments) authorized by tax office
    pub F1: Decimal,

    /// F2 - Alimony/maintenance payments (pre-May 1997) authorized by tax office
    pub F2: Decimal,

    /// F4 - Year-to-date RPP/RRSP contributions from non-periodic payments
    pub F4: Decimal,

    /// U1 - Union dues for the pay period
    pub U1: Decimal,

    /// HD - Annual deduction for living in prescribed zone (Form TD1)
    pub HD: Decimal,

    /// L - Additional tax deductions requested by employee (Form TD1)
    pub L: Decimal,

    /// D - Employee's year-to-date CPP contribution (before pay period)
    pub D: Decimal,

    /// DQ - Employee's year-to-date QPP contribution (before pay period, Quebec only)
    pub DQ: Decimal,

    /// D1 - Employee's year-to-date EI premium (before pay period)
    pub D1: Decimal,

    /// D2 - Employee's year-to-date second additional CPP contribution (before pay period)
    pub D2: Decimal,

    /// D2Q - Employee's year-to-date second additional QPP contribution (before pay period)
    pub D2Q: Decimal,

    /// PM - Total number of months during which CPP/QPP contributions are required
    /// Used in proration of maximum contribution
    pub PM: i32,

    /// Employee age (for CPP eligibility: 18-70)
    pub employee_age: i32,

    /// Province/territory of employment
    pub province: Province,

    /// E - Commission expenses (Form TD1X, for commission employees)
    pub E: Decimal,

    /// I1 - Total annual remuneration for commission employees (Form TD1X)
    pub I1: Decimal,

    /// LCF - Federal labour-sponsored funds tax credit
    pub LCF: Decimal,

    /// LCP - Provincial/territorial labour-sponsored funds tax credit
    pub LCP: Decimal,

    /// K3 - Other federal non-refundable tax credits (medical, charitable, authorized)
    pub K3: Decimal,

    /// K3P - Other provincial non-refundable tax credits
    pub K3P: Decimal,

    /// F3 - RPP/RRSP contributions from current non-periodic payment (bonus)
    pub F3: Decimal,

    /// Y - Total dependants amount from provincial TD1 (e.g., TD1ON)
    /// For Ontario: $554 × disabled dependants + $554 × dependants under 19
    /// Used in S (provincial tax reduction) calculation
    pub Y: Decimal,

    /// I_ytd - Year-to-date periodic income (before current pay period)
    pub I_ytd: Decimal,

    /// PR - Pay periods remaining in the year (including current period)
    pub PR: i32,

    /// F5B_ytd - Year-to-date F5B (CPP additional from bonuses, before current period)
    pub F5B_ytd: Decimal,

    // ===== PRIVATE COMPUTED FIELDS (Calculated during calc()) =====
    /// Tax year configuration (rates, constants, brackets)
    config: TaxYearConfig,

    /// A - Annual taxable income
    A: Decimal,

    /// C - CPP contributions for the pay period
    C: Decimal,

    /// C2 - Second additional CPP contributions for the pay period
    C2: Decimal,

    /// EI - Employment insurance premiums for the pay period
    EI: Decimal,

    /// F5 - Deductions for CPP additional contributions
    F5: Decimal,

    /// F5A - CPP additional contributions deducted from periodic income
    F5A: Decimal,

    /// F5B - CPP additional contributions deducted from non-periodic payment
    F5B: Decimal,

    /// T1 - Annual federal tax deduction
    T1: Decimal,

    /// T2 - Annual provincial tax deduction
    T2: Decimal,

    /// T3 - Annual basic federal tax
    T3: Decimal,

    /// T4 - Annual basic provincial tax
    T4: Decimal,
}

#[allow(non_snake_case)]
impl T4127Context {
    /// Create a new T4127 calculation context by loading config for a year
    pub fn new(year: i32) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let config = super::config::load_tax_config(year)?;
        Ok(Self::from_config(config))
    }

    /// Create a new T4127 calculation context with the given configuration
    pub fn from_config(config: TaxYearConfig) -> Self {
        Self {
            // Public inputs - defaults to zero/empty
            P: 12,
            cpp_exempt: false,
            ei_exempt: false,
            I: dec!(0),
            B: dec!(0),
            B1: dec!(0),
            PI: dec!(0),
            IE: dec!(0),
            TC: dec!(0),
            TCP: dec!(0),
            F: dec!(0),
            F1: dec!(0),
            F2: dec!(0),
            F4: dec!(0),
            U1: dec!(0),
            HD: dec!(0),
            L: dec!(0),
            D: dec!(0),
            DQ: dec!(0),
            D1: dec!(0),
            D2: dec!(0),
            D2Q: dec!(0),
            PM: 12,
            employee_age: 30,
            province: Province::ON,
            E: dec!(0),
            I1: dec!(0),
            LCF: dec!(0),
            LCP: dec!(0),
            K3: dec!(0),
            K3P: dec!(0),
            F3: dec!(0),
            Y: dec!(0),
            I_ytd: dec!(0),
            PR: 0,
            F5B_ytd: dec!(0),

            // Private computed fields
            config,
            A: dec!(0),
            C: dec!(0),
            C2: dec!(0),
            EI: dec!(0),
            F5: dec!(0),
            F5A: dec!(0),
            F5B: dec!(0),
            T1: dec!(0),
            T2: dec!(0),
            T3: dec!(0),
            T4: dec!(0),
        }
    }

    /// Calculate CPP contribution (C) and second additional CPP (C2)
    /// Returns total CPP for the pay period
    pub fn cpp(&mut self) -> Decimal {
        // Check if employee is exempt from CPP
        if self.cpp_exempt {
            self.C = dec!(0);
            self.C2 = dec!(0);
            self.F5 = dec!(0);
            self.F5A = dec!(0);
            self.F5B = dec!(0);
            return dec!(0);
        }

        // Quebec province: T4127 does not handle QC tax calculations
        // QC has separate provincial tax system (not implemented)
        if self.province == Province::QC {
            // Note: QPP calculations would be different from CPP
            // For now, return 0 as QC is not supported
            self.C = dec!(0);
            self.C2 = dec!(0);
            return dec!(0);
        }

        // Check age eligibility (18-70)
        if self.employee_age < 18 || self.employee_age > 70 {
            self.C = dec!(0);
            self.C2 = dec!(0);
            return dec!(0);
        }

        let cpp_config = &self.config.cpp;
        let cpp2_config = &self.config.cpp2;

        // Base CPP (C)
        let cpp_min_earn = cpp_config.basic_exemption;
        let cpp_max_earn = cpp_config.max_pensionable_earnings;
        let cpp_rate = cpp_config.employee_rate;
        let cpp_max_cpp = cpp_config.max_contribution;

        // Calculate exemption per period
        let exemption_per_period = cpp_min_earn / Decimal::from(self.P);

        // Pensionable earnings for this period
        let pensionable = (self.PI - exemption_per_period).max(dec!(0));

        // CPP this period
        let cpp_this_period = pensionable * cpp_rate;

        // Check YTD and max
        let ytd = if self.province == Province::QC { self.DQ } else { self.D };
        let remaining = cpp_max_cpp - ytd;
        self.C = cpp_this_period.min(remaining).max(dec!(0)).round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);

        // Second additional CPP (C2) - on earnings above YMPE
        let annual_gross = self.PI * Decimal::from(self.P);

        if annual_gross > cpp_max_earn && (remaining <= dec!(0) || remaining <= self.C) {
            let second_tier_start = cpp_max_earn;
            let second_tier_room = cpp2_config.max_earnings - second_tier_start;
            let second_tier_annual = (annual_gross - second_tier_start).min(second_tier_room);
            let second_tier_period = second_tier_annual * cpp2_config.rate / Decimal::from(self.P);

            let ytd2 = if self.province == Province::QC { self.D2Q } else { self.D2 };
            let remaining2 = cpp2_config.max_contribution - ytd2;
            self.C2 = second_tier_period.min(remaining2).max(dec!(0)).round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        } else {
            self.C2 = dec!(0);
        }

        // Calculate F5, F5A, F5B (additional CPP deductions for tax calc)
        self.calculate_f5();

        self.C + self.C2
    }

    /// Calculate F5, F5A, F5B - CPP additional contributions for tax deduction
    fn calculate_f5(&mut self) {
        let cpp_config = &self.config.cpp;

        // Per T4127 formula: F5 = C × (cpp_first_additional_rate / cpp_employee_rate) + C2
        self.F5 = self.C * (cpp_config.first_additional_rate / cpp_config.employee_rate) + self.C2;

        // F5A = F5 × ((PI – B)/PI)
        // F5B = F5 × (B/PI)
        if self.PI > dec!(0) {
            self.F5A = self.F5 * ((self.PI - self.B) / self.PI);
            self.F5B = self.F5 * (self.B / self.PI);
        } else {
            self.F5A = dec!(0);
            self.F5B = dec!(0);
        }
    }

    /// Calculate EI premium
    pub fn ei(&mut self) -> Decimal {
        // Check if employee is exempt from EI
        if self.ei_exempt {
            self.EI = dec!(0);
            return dec!(0);
        }

        // Quebec province: T4127 does not handle QC tax calculations
        if self.province == Province::QC {
            self.EI = dec!(0);
            return dec!(0);
        }

        let ei_config = &self.config.ei;
        let rate = ei_config.employee_rate;
        let max_contribution = ei_config.max_contribution;

        // Check if already at maximum
        if self.D1 >= max_contribution {
            self.EI = dec!(0);
            return dec!(0);
        }

        // Calculate EI for this period
        let ei_this_period = self.IE * rate;

        // Check if would exceed maximum
        let remaining = max_contribution - self.D1;

        self.EI = ei_this_period.min(remaining).round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);

        self.EI
    }

    /// Calculate Federal Tax (T1)
    fn fed_tax(&mut self) -> Decimal {
        // Quebec province: T4127 does not handle QC tax calculations
        if self.province == Province::QC {
            self.T1 = dec!(0);
            self.T3 = dec!(0);
            self.A = dec!(0);
            return dec!(0);
        }

        let p = Decimal::from(self.P);
        let fed_config = &self.config.federal;
        let cpp_config = &self.config.cpp;
        let ei_config = &self.config.ei;
        let pm = Decimal::from(self.PM);

        // Calculate annual taxable income A (T4127 Step 1 formula)
        // A = max(0, (I_a – F_a – F2_a – F5A_a – U1_a – HD – F1 – E)) + max(0, (B – F3 – F5B)) + max(0, B1 – F4 – F5B_ytd)
        // Where:
        //   I_a = I_ytd + PR × I (annualized periodic income)
        //   F_a = P × F
        //   F2_a = P × F2
        //   F5A_a = P × F5A
        //   U1_a = P × U1

        let pr = if self.PR > 0 { Decimal::from(self.PR) } else { p };
        let I_a = self.I_ytd + (pr * self.I);
        let F_a = p * self.F;
        let F2_a = p * self.F2;
        let F5A_a = p * self.F5A;
        let U1_a = p * self.U1;

        // Component 1: Periodic income after deductions
        let periodic_component = (I_a - F_a - F2_a - F5A_a - U1_a - self.HD - self.F1 - self.E).max(dec!(0));

        // Component 2: Current bonus after deductions
        let bonus_component = (self.B - self.F3 - self.F5B).max(dec!(0));

        // Component 3: YTD bonus after deductions
        let ytd_bonus_component = (self.B1 - self.F4 - self.F5B_ytd).max(dec!(0));

        self.A = periodic_component + bonus_component + ytd_bonus_component;

        if self.A <= dec!(0) {
            self.T1 = dec!(0);
            self.T3 = dec!(0);
            return self.L; // Only additional voluntary deduction
        }

        // Find tax bracket (R and K)
        let (R, K) = self.find_federal_bracket(self.A);

        // Lowest federal rate
        let lowest_fed_rate = fed_config.brackets.first().map(|b| b.rate).unwrap_or(dec!(0.15));

        // K1 = lowest_fed_rate × TC
        let K1 = lowest_fed_rate * self.TC;

        // K2 - CPP and EI credits (Quebec handled above, should never reach here for QC)
        // Per T4127: K2 = min(P × C × (cpp_base_rate / cpp_employee_rate), cpp_maximum_base_contribution × (PM / 12))
        let cpp_max_allowed = cpp_config.max_base_contribution * (pm / dec!(12));
        let cpp_credit = (p * self.C * (cpp_config.base_rate / cpp_config.employee_rate)).min(cpp_max_allowed);
        let ei_credit = (p * self.EI).min(ei_config.max_contribution);
        let K2 = lowest_fed_rate * (cpp_credit + ei_credit);

        // K4 - Canada Employment Amount credit
        let CEA = fed_config.canada_employment_amount;
        let K4 = (lowest_fed_rate * self.A).min(lowest_fed_rate * CEA);

        // T3 = (R × A) – K – K1 – K2 – K3 – K4
        self.T3 = ((R * self.A) - K - K1 - K2 - self.K3 - K4).max(dec!(0));

        // T1 = T3 – (P × LCF) (Quebec handled above, no abatement for non-QC)
        self.T1 = (self.T3 - (p * self.LCF)).max(dec!(0));

        // Per-period tax
        let period_tax = (self.T1 / p) + self.L;

        period_tax.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
    }

    /// Calculate Provincial Tax (T2)
    fn prov_tax(&mut self) -> Decimal {
        // Quebec has separate tax system
        if self.province == Province::QC {
            self.T2 = dec!(0);
            return dec!(0);
        }

        let province_config = match self.config.provincial.province_configs.get(&self.province) {
            Some(cfg) => cfg,
            None => {
                self.T2 = dec!(0);
                return dec!(0);
            }
        };

        let p = Decimal::from(self.P);
        let cpp_config = &self.config.cpp;
        let ei_config = &self.config.ei;
        let pm = Decimal::from(self.PM);

        // Annual taxable income (same as federal A)
        // Already calculated in fed_tax()

        // Find provincial bracket (V and KP)
        let (V, KP) = self.find_provincial_bracket(self.A);

        // Lowest provincial rate
        let lowest_prov_rate = province_config.brackets.first().map(|b| b.rate).unwrap_or(dec!(0.05));

        // K1P = lowest_prov_rate × TCP
        let K1P = lowest_prov_rate * self.TCP;

        // K2P - CPP and EI credits (provincial)
        // Per T4127: K2P = min(P × C × (cpp_base_rate / cpp_employee_rate), cpp_maximum_base_contribution × (PM / 12))
        let cpp_max_allowed = cpp_config.max_base_contribution * (pm / dec!(12));
        let cpp_credit = (p * self.C * (cpp_config.base_rate / cpp_config.employee_rate)).min(cpp_max_allowed);
        let ei_credit = (p * self.EI).min(ei_config.max_contribution);
        let K2P = lowest_prov_rate * (cpp_credit + ei_credit);

        // K4P - Canada Employment Amount credit (provincial)
        // Per T4127 Step 4: K4P = lowest_provincial_tax_rate × min(A, CEA)
        let CEA = province_config.canada_employment_amount;
        let K4P = lowest_prov_rate * self.A.min(CEA);

        // K5P - Alberta-specific tax reduction (0 for other provinces)
        let K5P = if self.province == Province::AB {
            // For AB only: K5P = max(0, ((K1P + K2P) – threshold) × rate)
            // Use config values if available, otherwise default to 0
            if let (Some(threshold), Some(rate)) = (province_config.k5p_threshold, province_config.k5p_rate) {
                ((K1P + K2P) - threshold).max(dec!(0)) * rate
            } else {
                dec!(0)
            }
        } else {
            dec!(0)
        };

        // T4 = (V × A) – KP – K1P – K2P – K3P – K4P – K5P
        self.T4 = ((V * self.A) - KP - K1P - K2P - self.K3P - K4P - K5P).max(dec!(0));

        // V1 - Provincial surtax
        let V1 = self.calculate_provincial_surtax(province_config);

        // V2 - Ontario Health Premium
        let V2 = if self.province == Province::ON { self.calculate_ontario_health_premium() } else { dec!(0) };

        // S - Provincial tax reduction (Ontario and British Columbia)
        // Per T4127 p22 (ON): S = lesser of (i) T4+V1, (ii) [2×(S2+Y)]–[T4+V1]
        // Per T4127 p21 (BC): S depends on A thresholds
        let S = if self.province == Province::ON {
            let s2 = province_config.s2_amount;
            let t4_plus_v1 = self.T4 + V1;
            let s_calc = (dec!(2) * (s2 + self.Y)) - t4_plus_v1;
            t4_plus_v1.min(s_calc).max(dec!(0))
        } else if self.province == Province::BC {
            // BC S calculation per T4127 p21
            let s2 = province_config.s2_amount;
            if self.A <= dec!(25570) {
                self.T4.min(s2)
            } else if self.A <= dec!(41722) {
                let reduction = s2 - ((self.A - dec!(25570)) * dec!(0.0356));
                self.T4.min(reduction.max(dec!(0)))
            } else {
                dec!(0)
            }
        } else {
            dec!(0)
        };

        // T2 = T4 + V1 + V2 – S – (P × LCP)
        self.T2 = (self.T4 + V1 + V2 - S - (p * self.LCP)).max(dec!(0));

        // Per-period tax
        let period_tax = self.T2 / p;

        period_tax.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
    }

    /// Calculate all deductions for a pay period including non-periodic payments using T4127 formulas
    ///
    /// This function retrieves YTD values and employee information internally using the provided providers.
    ///
    /// # Parameters
    /// - `employee_id`: Employee ID for YTD and employee info lookup
    /// - `periodic_income`: Regular periodic earnings (I)
    /// - `non_periodic_income`: Bonuses, commissions, retroactive pay (B)
    /// - `pay_periods_per_year`: Number of pay periods per year
    /// - `pay_date`: The pay date for YTD lookup and age calculation
    /// - `ytd_provider`: Provider for retrieving YTD totals
    /// - `employee_provider`: Provider for retrieving employee information
    /// - `additional_deductions`: Additional deductions to map to T4127 variables (F, U1, L, etc.)
    pub fn calculate_all_deductions<Y: YtdProvider, E: EmployeeProvider>(
        &mut self,
        employee_id: i64,
        periodic_income: Decimal,
        non_periodic_income: Decimal,
        pay_periods_per_year: i32,
        pay_date: NaiveDate,
        ytd_provider: &Y,
        employee_provider: &E,
        additional_deductions: &[AdditionalDeduction],
    ) -> Result<PayrollDeductions, Box<dyn std::error::Error + Send + Sync>> {
        // Retrieve YTD totals
        let pay_date_year = pay_date.year();
        let ytd = ytd_provider.get_ytd_totals(employee_id, pay_date_year)?;

        // Retrieve employee information
        let employee_info = employee_provider.get_employee_info(employee_id, pay_date_year)?;

        // Calculate employee's age as of pay date
        let employee_age = employee_info.age_as_of(pay_date);

        let gross_pay = periodic_income + non_periodic_income;

        self.P = pay_periods_per_year;
        self.I = periodic_income;
        self.B = non_periodic_income;
        self.B1 = Decimal::ZERO; // ytd_non_periodic - not tracked separately in YtdTotals
        self.PI = gross_pay;
        self.IE = gross_pay;
        self.TC = employee_info.federal_personal_amount;
        self.TCP = employee_info.provincial_personal_amount;
        self.D = ytd.cpp;
        self.D2 = ytd.cpp2;
        self.D1 = ytd.ei;
        self.employee_age = employee_age;
        self.province = employee_info.province;
        self.cpp_exempt = employee_info.cpp_exempt;
        self.ei_exempt = employee_info.ei_exempt;

        // Map additional deductions to T4127 variables using DeductionType enum
        // F - RPP/RRSP contributions (Pension/RRSP)
        // U1 - Union dues
        // L - Additional tax deductions (addon_tax)
        // Note: Health/Group insurance is not a T4127 deduction variable
        for deduction in additional_deductions {
            if let Some(deduction_type) = DeductionType::from_str(&deduction.name) {
                if let Some(var) = deduction_type.t4127_variable() {
                    match var {
                        "F" => self.F = deduction.amount,
                        "U1" => self.U1 = deduction.amount,
                        "L" => self.L = deduction.amount,
                        _ => {}
                    }
                }
            }
        }

        Ok(self.calc())
    }

    /// Calculate all deductions
    /// This is the main entry point - call after setting all public input fields
    pub fn calc(&mut self) -> PayrollDeductions {
        let _cpp_total = self.cpp();
        let ei = self.ei();
        let fed_tax = self.fed_tax();
        let prov_tax = self.prov_tax();

        PayrollDeductions { cpp: self.C, cpp2: self.C2, ei, federal_tax: fed_tax, provincial_tax: prov_tax }
    }

    // ===== Helper Methods =====

    fn find_federal_bracket(&self, annual_income: Decimal) -> (Decimal, Decimal) {
        for bracket in &self.config.federal.brackets {
            if let Some(upper) = bracket.upper_limit {
                if annual_income <= upper {
                    return (bracket.rate, bracket.constant);
                }
            } else {
                return (bracket.rate, bracket.constant);
            }
        }
        (dec!(0.15), dec!(0))
    }

    fn find_provincial_bracket(&self, annual_income: Decimal) -> (Decimal, Decimal) {
        let province_config = match self.config.provincial.province_configs.get(&self.province) {
            Some(cfg) => cfg,
            None => return (dec!(0), dec!(0)),
        };

        for bracket in &province_config.brackets {
            if let Some(upper) = bracket.upper_limit {
                if annual_income <= upper {
                    return (bracket.rate, bracket.constant);
                }
            } else {
                return (bracket.rate, bracket.constant);
            }
        }
        (dec!(0), dec!(0))
    }

    fn calculate_provincial_surtax(&self, province_config: &super::ProvinceTaxRates) -> Decimal {
        let mut v1 = dec!(0);

        if !province_config.surtax_tiers.is_empty() {
            // Multi-tier surtax (e.g., Ontario)
            for tier in &province_config.surtax_tiers {
                if self.T4 > tier.threshold {
                    v1 += (self.T4 - tier.threshold) * tier.rate;
                }
            }
        } else if let Some(ref surtax) = province_config.surtax {
            // Single surtax
            if self.T4 > surtax.threshold {
                v1 = (self.T4 - surtax.threshold) * surtax.rate;
            }
        }

        v1
    }

    fn calculate_ontario_health_premium(&self) -> Decimal {
        if self.A <= dec!(20000) {
            dec!(0)
        } else if self.A <= dec!(25000) {
            dec!(0)
        } else if self.A <= dec!(36000) {
            dec!(300)
        } else if self.A <= dec!(48000) {
            dec!(450)
        } else if self.A <= dec!(72000) {
            dec!(600)
        } else if self.A <= dec!(200000) {
            dec!(750)
        } else {
            dec!(900)
        }
    }
}
