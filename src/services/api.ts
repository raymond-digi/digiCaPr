// API service layer for Tauri commands
import { invoke } from '@tauri-apps/api/tauri'
import type { Employee, PayRateHistory, EmploymentHistory, PersonalAmount, EmployeeAutofill } from '@/types/employee'
import type { Payroll, PayrollCalculationInput, YtdTotals } from '@/types/payroll'
import type { Company } from '@/types/company'

// Database Commands
export const databaseApi = {
  createDatabase: (path: string) => 
    invoke<string>('create_database', { path }),
  
  openDatabase: (path: string) => 
    invoke<string>('open_database', { path }),
  
  closeDatabase: () => 
    invoke<void>('close_database'),
  
  getCurrentDatabasePath: () => 
    invoke<string | null>('get_current_database_path'),
  
  isDatabaseOpen: () => 
    invoke<boolean>('is_database_open')
}

// Employee Commands
export const employeeApi = {
  listEmployees: () => 
    invoke<Employee[]>('list_employees'),
  
  listActiveEmployees: () => 
    invoke<Employee[]>('list_active_employees'),
  
  getEmployee: (id: number) => 
    invoke<Employee>('get_employee', { id }),
  
  createEmployee: (employee: Employee) => 
    invoke<number>('create_employee', { employee }),
  
  updateEmployee: (employee: Employee) => 
    invoke<void>('update_employee', { employee }),
  
  deleteEmployee: (id: number) => 
    invoke<void>('delete_employee', { id }),
  
  searchEmployees: (query: string) => 
    invoke<Employee[]>('search_employees', { query }),
  
  getPayRateHistory: (employeeId: number) =>
    invoke<PayRateHistory[]>('get_pay_rate_history', { employeeId }),
  
  getEmploymentHistory: (employeeId: number) =>
    invoke<EmploymentHistory[]>('get_employment_history', { employeeId }),
  
  exportEmployeesCsv: (filePath: string) =>
    invoke<number>('export_employees_csv', { filePath }),
  
  importEmployeesCsv: (filePath: string) =>
    invoke<ImportResult>('import_employees_csv', { filePath }),

  // Employee Autofill Commands
  getEmployeeAutofill: (employeeId: number) =>
    invoke<EmployeeAutofill[]>('get_employee_autofill', { employeeId }),

  getActiveEmployeeAutofill: (employeeId: number) =>
    invoke<EmployeeAutofill[]>('get_active_employee_autofill', { employeeId }),

  saveEmployeeAutofill: (autofill: EmployeeAutofill) =>
    invoke<number>('save_employee_autofill', { autofill }),

  deleteEmployeeAutofill: (id: number) =>
    invoke<void>('delete_employee_autofill', { id }),

  deleteAllEmployeeAutofill: (employeeId: number) =>
    invoke<void>('delete_all_employee_autofill', { employeeId }),

  getTaxRates: (year: number) =>
    invoke<{
      cpp_employee_rate: number;
      cpp_basic_exemption: number;
      cpp_ympe: number;
      cpp_max_contribution: number;
      cpp2_rate: number;
      cpp2_max_earnings: number;
      cpp2_max_contribution: number;
      ei_rate: number;
      ei_max_insurable_earnings: number;
      ei_max_contribution: number;
    }>('get_tax_rates', { year }),
}

// Import result type
export interface ImportResult {
  imported: number;
  skipped: number;
  errors: Array<{
    employee_number: string;
    employee_name: string;
    error: string;
  }>;
}

export const personalAmountApi = {
  getPersonalAmount: (employeeId: number, province: string, year: number) =>
    invoke<PersonalAmount>('get_personal_amount', { employeeId, province, year }),
  
  getPersonalAmounts: (employeeId: number) =>
    invoke<PersonalAmount[]>('get_personal_amounts', { employeeId }),
  
  getLatestPersonalAmountByProvince: (employeeId: number, province: string) =>
    invoke<PersonalAmount | null>('get_latest_personal_amount_by_province', { employeeId, province }),
  
  createPersonalAmount: (personalAmount: PersonalAmount) =>
    invoke<number>('create_personal_amount', { personalAmount }),
  
  updatePersonalAmount: (personalAmount: PersonalAmount) =>
    invoke<void>('update_personal_amount', { personalAmount }),
  
  getBasicPersonalAmounts: (province: string, year: number) =>
    invoke<{ federal_amount: number; provincial_amount: number }>('get_basic_personal_amounts', { province, year }),
  
  getAvailableTaxYears: () =>
    invoke<number[]>('get_available_tax_years'),
}

// Payroll Commands
export const payrollApi = {
  calculatePayroll: (params: PayrollCalculationInput) =>
    invoke<Payroll>('calculate_payroll', {
      employeeId: params.employee_id,
      payPeriodStart: params.pay_period_start,
      payPeriodEnd: params.pay_period_end,
      payDate: params.pay_date,
      regularHours: params.regular_hours,
      overtimeHours: params.overtime_hours,
      grossPay: params.gross_pay,
      additionalEarnings: params.additional_earnings,
      additionalDeductions: params.additional_deductions
    }),
  
  savePayroll: (payroll: Payroll) =>
    invoke<number>('save_payroll', { payroll }),
  
  listPayroll: () =>
    invoke<Payroll[]>('list_payroll'),
  
  listEmployeePayroll: (employeeId: number) =>
    invoke<Payroll[]>('list_employee_payroll', { employeeId }),
  
  getPayroll: (id: number) =>
    invoke<Payroll>('get_payroll', { id }),
  
  getYtdTotals: (employeeId: number, year: number) =>
    invoke<YtdTotals>('get_ytd_totals', { employeeId, year }),
  
  deletePayroll: (id: number) =>
    invoke<void>('delete_payroll', { id }),

  createPayroll: (params: import('@/types/payroll').CurrentPayrollInput) =>
    invoke<import('@/types/payroll').CurrentPayrollResult>('create_current_payroll', {
      payPeriodStart: params.pay_period_start,
      payPeriodEnd: params.pay_period_end,
      payDate: params.pay_date,
      employeeIds: params.employee_ids
    }),

  getAvailableEmployeesForCurrentPayroll: (payPeriodStart: string, payPeriodEnd: string) =>
    invoke<Employee[]>('get_available_employees_for_current_payroll', { payPeriodStart, payPeriodEnd }),

  checkHistoryPayrollDatesExist: (payPeriodStart: string, payPeriodEnd: string, payDate: string) =>
    invoke<boolean>('check_history_payroll_dates_exist', { payPeriodStart, payPeriodEnd, payDate }),

  postCurrentToHistory: (payrollIds: number[]) =>
    invoke<number[]>('post_current_to_history', { payrollIds }),

  getCurrentPayrollDates: () =>
    invoke<import('@/types/payroll').CurrentPayrollDates | null>('get_current_payroll_dates'),

  listCurrentPayroll: () =>
    invoke<import('@/types/payroll').Payroll[]>('list_current_payroll'),

  updateCurrentPayroll: (payroll: import('@/types/payroll').Payroll) =>
    invoke<void>('update_current_payroll', { payroll }),

  clearCurrentPayroll: () =>
    invoke<void>('clear_current_payroll'),

  addToCurrentPayroll: (payroll: import('@/types/payroll').Payroll) =>
    invoke<number>('add_to_current_payroll', { payroll }),

  exportCurrentPayrollCsv: (outputPath: string) =>
    invoke<string>('export_current_payroll_csv', { outputPath }),

  importCurrentPayrollCsv: (
    filePath: string,
    payPeriodStart?: string,
    payPeriodEnd?: string,
    payDate?: string
  ) =>
    invoke<import('@/types/payroll').CurrentPayrollResult>('import_current_payroll_csv', {
      filePath,
      payPeriodStart,
      payPeriodEnd,
      payDate
    }),

  importHistoryPayrollCsv: (filePath: string) =>
    invoke<{ imported: number; errors: string[] }>('import_history_payroll_csv', { filePath }),

  exportHistoryPayrollCsv: (
    outputPath: string,
    employeeId?: number | null,
    payDateFrom?: string | null,
    payDateTo?: string | null,
    searchTerm?: string | null
  ) =>
    invoke<string>('export_history_payroll_csv', {
      outputPath,
      employeeId,
      payDateFrom,
      payDateTo,
      searchTerm
    }),

  listPayrollHistory: (filters?: import('@/types/payroll').PayrollHistoryFilters) =>
    invoke<import('@/types/payroll').PayrollHistoryListResult>('list_payroll_history', { filters }),

  listPayrollYears: (employeeId?: number) =>
    invoke<number[]>('list_payroll_years', { employeeId: employeeId ?? null }),

  listPayrollPeriods: (year: number, employeeId?: number) =>
    invoke<import('@/types/payroll').PayrollPeriod[]>('list_payroll_periods', { year, employeeId: employeeId ?? null }),

  deleteHistoryPayroll: (id: number) =>
    invoke<void>('delete_history_payroll', { id }),

  updateHistoryPayroll: (payroll: import('@/types/payroll').Payroll) =>
    invoke<void>('update_history_payroll', { payroll }),

  saveHistoryPayroll: (payroll: import('@/types/payroll').Payroll) =>
    invoke<number>('save_payroll', { payroll }),

  saveRawPayroll: (payroll: import('@/types/payroll').Payroll) =>
    invoke<number>('save_raw_payroll', { payroll }),
}

// Remittance Commands
export const remittanceApi = {
  getRemittanceSummary: (cutoffDate: string) =>
    invoke<import('@/types/payroll').RemittanceSummary>('get_remittance_summary', { cutoffDate }),

  createRemittance: (cutoffDate: string, craConfirmation?: string) =>
    invoke<number>('create_remittance', { cutoffDate, craConfirmation }),

  listRemittances: (year?: number) =>
    invoke<import('@/types/payroll').Remittance[]>('list_remittances', { year }),

  getRemittanceYears: () =>
    invoke<number[]>('get_remittance_years'),

  getRemittance: (id: number) =>
    invoke<import('@/types/payroll').Remittance>('get_remittance', { id }),

  deleteRemittance: (id: number) =>
    invoke<void>('delete_remittance', { id })
}

// Company Commands
export const companyApi = {
  getCompany: () => 
    invoke<Company | null>('get_company'),
  
  saveCompany: (company: Company) => 
    invoke<void>('save_company', { company })
}

// Report Commands
export const reportsApi = {
  generatePaystub: (payrollId: number, outputPath: string) =>
    invoke<string>('generate_paystub', { payrollId, outputPath }),
  
  generateT4: (employeeId: number, year: number, outputPath: string) =>
    invoke<string>('generate_t4', { employeeId, year, outputPath }),
  
  exportPayrollCsv: (year: number, outputPath: string) =>
    invoke<string>('export_payroll_csv', { year, outputPath }),
  
  // current payroll generation commands
  generateCurrentPayrollReport: (payrollIds: number[], outputDir: string) =>
    invoke<string>('generate_payroll_report', { payrollIds, outputDir }),

  // history payroll report
  generateHistoryPayrollReport: (payrollIds: number[], outputDir: string) =>
    invoke<string>('generate_history_payroll_report', { payrollIds, outputDir }),

  generateCurrentPayrollPaystubs: (payrollIds: number[], outputDir: string) =>
    invoke<string[]>('generate_payroll_paystubs', { payrollIds, outputDir }),

  generateRemittanceReport: (remittanceId: number, outputDir: string) =>
    invoke<string>('generate_remittance_report', { remittanceId, outputDir }),

  generatePersonalAmountReport: (year: number, outputPath: string) =>
    invoke<string>('generate_personal_amount_report', { year, outputPath }),

  calculateT4ForYear: (year: number) =>
    invoke<import('@/types/t4').T4SlipLegacy[]>('calculate_t4_for_year', { year }),

  getT4Summary: (year: number) =>
    invoke<import('@/types/t4').T4SummaryData>('get_t4_summary', { year }),

  generateT4SummaryPdf: (year: number, outputPath: string) =>
    invoke<string>('generate_t4_summary_pdf', { year, outputPath }),

  exportT4Xml: (year: number, outputPath: string) =>
    invoke<string>('export_t4_xml', { year, outputPath }),

  exportT4Csv: (year: number, outputPath: string) =>
    invoke<string>('export_t4_csv', { year, outputPath }),

  generatePayrollT4: (year: number, outputDir: string) =>
    invoke<string[]>('generate_payroll_t4', { year, outputDir }),
}

// T4 Commands (flexible schema)
export const t4Api = {
  getT4Years: () =>
    invoke<number[]>('get_t4_years'),

  listT4SlipsForYear: (year: number) =>
    invoke<import('@/types/t4').T4Slip[]>('list_t4_slips_for_year', { year }),

  getOrCreateT4Slip: (employeeId: number, year: number) =>
    invoke<import('@/types/t4').T4Slip>('get_or_create_t4_slip', { employeeId, year }),

  createT4SlipVersion: (employeeId: number, year: number) =>
    invoke<import('@/types/t4').T4Slip>('create_t4_slip_version', { employeeId, year }),

  getT4BoxValues: (slipId: number) =>
    invoke<import('@/types/t4').T4BoxValue[]>('get_t4_box_values', { slipId }),

  saveT4BoxValue: (boxValue: import('@/types/t4').T4BoxValue) =>
    invoke<number>('save_t4_box_value', { boxValue }),

  calculateT4ForYear: (year: number) =>
    invoke<import('@/types/t4').T4Slip[]>('calculate_t4_for_year', { year }),

  fileT4Slip: (slipId: number, filedBy: string) =>
    invoke<void>('file_t4_slip', { slipId, filedBy }),

  lockT4Slip: (slipId: number) =>
    invoke<void>('lock_t4_slip', { slipId }),

  unlockT4Slip: (slipId: number) =>
    invoke<void>('unlock_t4_slip', { slipId }),

  getT4SlipsForYear: (year: number) =>
    invoke<import('@/types/t4').T4SlipLegacy[]>('get_t4_slips_for_year', { year }),

  updateBoxValues: (update: import('@/types/t4').T4BoxValueUpdate) =>
    invoke<number>('update_t4_box_values', { update }),
}

// Registry Commands
export interface RegistryEntry {
  id?: number;
  key_path: string;
  value: RegistryValue;
  created_at: string;
  updated_at: string;
}

export type RegistryValue =
  | { type: 'String'; value: string }
  | { type: 'Integer'; value: number }
  | { type: 'Boolean'; value: boolean }
  | { type: 'Json'; value: any };

export const registryApi = {
  set: (keyPath: string, valueType: string, value: string) =>
    invoke<void>('registry_set', { request: { key_path: keyPath, value_type: valueType, value } }),

  get: (keyPath: string) =>
    invoke<RegistryEntry | null>('registry_get', { request: { key_path: keyPath } }),

  delete: (keyPath: string) =>
    invoke<void>('registry_delete', { request: { key_path: keyPath } }),

  exists: (keyPath: string) =>
    invoke<boolean>('registry_exists', { request: { key_path: keyPath } }),

  listKeys: (pathPrefix: string) =>
    invoke<string[]>('registry_list_keys', { request: { path_prefix: pathPrefix } }),

  getAll: (pathPrefix: string) =>
    invoke<RegistryEntry[]>('registry_get_all', { request: { path_prefix: pathPrefix } }),

  deleteAll: (pathPrefix: string) =>
    invoke<void>('registry_delete_all', { request: { path_prefix: pathPrefix } }),

  // Helper methods for common types
  setString: (keyPath: string, value: string) =>
    registryApi.set(keyPath, 'String', value),

  setInteger: (keyPath: string, value: number) =>
    registryApi.set(keyPath, 'Integer', value.toString()),

  setBoolean: (keyPath: string, value: boolean) =>
    registryApi.set(keyPath, 'Boolean', value.toString()),

  setJson: (keyPath: string, value: any) =>
    registryApi.set(keyPath, 'Json', JSON.stringify(value)),

  getString: async (keyPath: string): Promise<string | null> => {
    const entry = await registryApi.get(keyPath);
    if (entry && entry.value.type === 'String') {
      return entry.value.value;
    }
    return null;
  },

  getInteger: async (keyPath: string): Promise<number | null> => {
    const entry = await registryApi.get(keyPath);
    if (entry && entry.value.type === 'Integer') {
      return entry.value.value;
    }
    return null;
  },

  getBoolean: async (keyPath: string): Promise<boolean | null> => {
    const entry = await registryApi.get(keyPath);
    if (entry && entry.value.type === 'Boolean') {
      return entry.value.value;
    }
    return null;
  },

  getJson: async (keyPath: string): Promise<any | null> => {
    const entry = await registryApi.get(keyPath);
    if (entry && entry.value.type === 'Json') {
      return entry.value.value;
    }
    return null;
  },
}

// Vacation Commands
export const vacationApi = {
  getBalance: (employeeId: number) =>
    invoke<{ employee_id: number, balance: number, balance_cents: number, balance_days: number, total_accrued: number, total_paid: number }>('get_vacation_balance', { employeeId }),

  getHistory: (employeeId: number) =>
    invoke<any[]>('get_vacation_history', { employeeId }),

  recordAccrual: (employeeId: number, payrollId: number | null, grossPay: number, vacationPayRate: number) =>
    invoke<any>('record_vacation_accrual', { employeeId, payrollId, grossPay, vacationPayRate }),

  recordAdjustment: (employeeId: number, amount: number, amountDays: number | null, notes: string | null) =>
    invoke<any>('record_vacation_adjustment', { employeeId, amount, amountDays, notes }),

  createTimeOff: (employeeId: number, startDate: string, endDate: string, estimatedPayout: number, payoutAmount: number, notes: string | null) =>
    invoke<any>('create_vacation_time_off', { employeeId, startDate, endDate, estimatedPayout, payoutAmount, notes }),

  updateTimeOff: (timeOffId: number, startDate: string, endDate: string, payoutAmount: number, notes: string | null) =>
    invoke<any>('update_vacation_time_off', { timeOffId, startDate, endDate, payoutAmount, notes }),

  deleteTimeOff: (timeOffId: number) =>
    invoke<void>('delete_vacation_time_off', { timeOffId }),

  getTimeOffHistory: (employeeId: number) =>
    invoke<any[]>('get_vacation_time_off_history', { employeeId }),
}
