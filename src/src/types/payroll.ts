// TypeScript types for Payroll

export interface AdditionalEarning {
  id?: number;
  payroll_id: number;
  earning_type: string;
  amount: number;
  hours?: number | null;
  is_periodic: boolean;  // Periodic vs non-periodic for tax calculations (per CRA T4127)
}

export interface Payroll {
  id?: number
  employee_id: number
  pay_period_start: string
  pay_period_end: string
  pay_date: string
  regular_hours?: number | null
  overtime_hours?: number | null
  additional_earnings: AdditionalEarning[]
  insured_earning: number
  gross_pay: number
  additional_earnings_total: number
  additional_tax_amount: number
  deductions: Deductions
  net_pay: number
  pay_period_number?: number
  total_pay_periods: number
  total_deductions?: number
  additional_deductions: number
  federal_personal_amount: number
  provincial_personal_amount: number
  province: string
  remittance_id?: number | null
  created_at?: string
}

export interface Deductions {
  cpp: number
  cpp2: number
  ei: number
  federal_tax: number
  provincial_tax: number
  additional: AdditionalDeduction[]
}

export interface AdditionalDeduction {
  name: string
  amount: number
}

export interface YtdTotals {
  gross_pay: number
  cpp: number
  cpp2: number
  ei: number
  federal_tax: number
  provincial_tax: number
  net_pay: number
}

// Payroll Period for hierarchical navigation (year -> period -> payroll records)
export interface PayrollPeriod {
  pay_period_start: string;
  pay_period_end: string;
  pay_date: string;
}

export interface PayrollCalculationInput {
  employee_id: number
  pay_period_start: string
  pay_period_end: string
  pay_date: string
  regular_hours?: number | null
  overtime_hours?: number | null
  gross_pay?: number | null
  additional_earnings?: AdditionalEarning[] | null
  additional_deductions?: AdditionalDeduction[] | null
}

// Remittance types
export interface Remittance {
  id?: number
  period_start: string
  period_end: string
  total_employees: number
  total_earnings: number
  total_cpp: number
  total_cpp2: number
  total_ei: number
  total_federal_tax: number
  total_provincial_tax: number
  grand_total: number
  cra_report_reference?: string | null
  generated_at: string
}

export interface RemittanceInput {
  cutoff_date: string  // Get all unfiled payrolls before this date
}

export interface RemittanceSummary {
  unfiled_payrolls_count: number
  total_earnings: number
  total_cpp: number
  total_cpp2: number
  total_ei: number
  total_federal_tax: number
  total_provincial_tax: number
  grand_total: number
  period_start: string
  period_end: string
}

// Current Payroll types (drafts not affecting YTD)
export interface CurrentPayrollError {
   employee_id: number
   employee_name: string
   error: string
 }

export interface EmployeeImportError {
   employee_number: string
   employee_name: string
   error: string
 }
export interface CurrentAdditionalEarning {
  id?: number;
  payroll_id: number;
  earning_type: string;
  amount: number;
  hours?: number | null;
  is_periodic: boolean;  // Periodic vs non-periodic for tax calculations (per CRA T4127)
}


export interface CurrentPayrollDeduction {
  name: string;
  amount: number;
}

export interface CurrentPayrollInput {
  pay_period_start: string
  pay_period_end: string
  pay_date: string
  employee_ids?: number[]  // Optional: specific employees
  pay_period_number?: number
  total_pay_periods?: number
}

export interface CurrentPayrollResult {
  payrolls: Payroll[]
  errors: CurrentPayrollError[]
  created: number
  updated: number
}

export interface CurrentPayrollDates {
  pay_period_start: string;
  pay_period_end: string;
  pay_date: string;
  pay_period_number?: number;
  total_pay_periods?: number;
}

// Payroll History Search/Filter types
export interface PayrollHistoryFilters {
  employee_id?: number | null;
  pay_date_from?: string | null;
  pay_date_to?: string | null;
  search_term?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface PayrollHistoryListResult {
  payrolls: Payroll[];
  total_count: number;
}

export interface EarningTypeInfo {
  name: string;
  display_name: string;
  is_periodic: boolean;
}

export interface DeductionTypeInfo {
  name: string;
  display_name: string;
  t4127_variable: string | null;
}

export interface AdditionalTypesResponse {
  earnings: EarningTypeInfo[];
  deductions: DeductionTypeInfo[];
}

// Pre-defined types matching Rust enums (crates/cpr-core/src/models)
export const EARNING_TYPES: readonly EarningTypeInfo[] = [
  { name: 'bonus', display_name: 'Bonus', is_periodic: false },
  { name: 'commission', display_name: 'Commission', is_periodic: false },
  { name: 'benefit', display_name: 'Benefit', is_periodic: true },
  { name: 'allowance', display_name: 'Allowance', is_periodic: true },
  { name: 'vacation', display_name: 'Vacation', is_periodic: true },
  { name: 'other', display_name: 'Other', is_periodic: false },
] as const;

// Pre-defined types matching Rust enums (crates/cpr-core/src/models)
export const DEDUCTION_TYPES: readonly DeductionTypeInfo[] = [
  { name: 'group_insurance', display_name: 'Group Insurance', t4127_variable: null },
  { name: 'pension_rrsp', display_name: 'Pension/RRSP', t4127_variable: 'F' },
  { name: 'union_dues', display_name: 'Union Dues', t4127_variable: 'U1' },
  { name: 'net_pay_adjust', display_name: 'Net Pay Adjustment', t4127_variable: null },
  { name: 'addon_tax', display_name: 'Additional Tax', t4127_variable: 'L' },
] as const;
