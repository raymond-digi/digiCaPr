// TypeScript types for T4 Year-End Feature

import type { Employee } from '@/types/employee'

/** T4 box type information */
export interface T4BoxTypeInfo {
  readonly box_code: string       // "box_14", "box_16", etc.
  readonly box_number: string     // "14", "16", "16a", "18", "22", "24", "26", "45", "52"
  readonly display_name: string   // "Employment income", etc.
}

// Pre-defined types matching Rust enum T4BoxType (crates/cpr-core/src/models/payroll.rs)
export const T4_BOX_TYPES: readonly T4BoxTypeInfo[] = [
  { box_code: 'box_45', box_number: '45', display_name: 'Dental benefit' },
  { box_code: 'box_14', box_number: '14', display_name: 'Employment income' },
  { box_code: 'box_16', box_number: '16', display_name: 'CPP contributions' },
  { box_code: 'box_16a', box_number: '16a', display_name: 'CPP2 contributions' },
  { box_code: 'box_26', box_number: '26', display_name: 'CPP pensionable earnings' },
  { box_code: 'box_18', box_number: '18', display_name: 'EI premiums' },
  { box_code: 'box_24', box_number: '24', display_name: 'EI insurable earnings' },
  { box_code: 'box_22', box_number: '22', display_name: 'Income tax deducted' },
  { box_code: 'box_20', box_number: '20', display_name: 'RPP contributions' },
  { box_code: 'box_52', box_number: '52', display_name: 'Pension adjustment' },
] as const

/** T4 slip status values */
export type T4SlipStatus = 'draft' | 'calculated' | 'filed' | 'locked'

/** T4 slip record (from t4_slip table) */
export interface T4Slip {
  id?: number
  employee_id: number
  year: number
  slip_version: number           // Version for recalculations (1, 2, 3, ...)
  status: T4SlipStatus
  filed_at?: string | null       // When the T4 was filed
  filed_by?: string | null       // Who filed it
  created_at: string
  updated_at: string
}

/** T4 box value (from t4_box_value table) - flexible key-value storage */
export interface T4BoxValue {
  id?: number
  t4_slip_id: number
  box_type: string               // "box_14", "box_16", etc.
  calculated_value: number       // Store as cents in backend
  adjustment_value: number       // Store as cents in backend
}

/** T4 box value with display info (for UI) */
export interface T4BoxValueDisplay extends T4BoxValue {
  box_number: number
  display_name: string
  final_value: number            // calculated_value + adjustment_value
}

/** Summary of T4 values for all employees in a year */
export interface T4Summary {
  year: number
  total_box_14: number
  total_box_16: number
  total_box_18: number
  total_box_22: number
  employee_count: number
}

/** T4 Summary data (T4 Summary boxes - different from T4 slip boxes) */
export interface T4SummaryData {
  year: number
  /** Box 88 - Total number of T4 slips filed */
  total_slips: number
  /** Box 14 - Total employment income */
  total_employment_income: number
  /** Box 20 - Total RPP contributions */
  total_rpp_contributions: number
  /** Box 52 - Total pension adjustment */
  total_pension_adjustment: number
  /** Box 16 - Total employee CPP contributions */
  total_employee_cpp: number
  /** Box 16a - Total employee CPP2 contributions */
  total_employee_cpp2: number
  /** Box 27 - Total employer CPP contributions (= employee CPP) */
  total_employer_cpp: number
  /** Box 27a - Total employer CPP2 contributions (= employee CPP2) */
  total_employer_cpp2: number
  /** Box 18 - Total employee EI premiums */
  total_employee_ei: number
  /** Box 19 - Total employer EI premiums (= employee EI × 1.4) */
  total_employer_ei: number
  /** Box 22 - Total income tax deducted */
  total_income_tax: number
  /** Box 80 - Total deductions reported (sum of 16, 16a, 27, 27a, 18, 19, 22) */
  total_deductions_reported: number
  /** Box 82 - Total remittances paid for the year */
  total_remittances_paid: number
  /** Difference between Box 80 and Box 82 */
  difference: number
}

/** T4 slip data with additional fields for summary */
export interface T4SlipLegacyExtended extends T4SlipLegacy {
  /** Box 16a - Employee's CPP2 contributions */
  cpp2_contributions: number
  /** Box 20 - RPP contributions */
  rpp_contributions: number
  /** Box 52 - Pension adjustment */
  pension_adjustment: number
  /** Net pay from payroll history (ground truth) */
  net_pay: number
  /** Computed net pay from T4 box values for comparison */
  computed_net_pay: number
}

/** T4 slip data returned from backend (for slip display) */
export interface T4SlipData {
  slip: T4Slip
  employee: Employee
  box_values: T4BoxValueDisplay[]
  employment_code: string | null
  province_of_employment: string
}

/** Legacy T4Slip interface for backward compatibility */
export interface T4SlipLegacy {
  employee: Employee
  year: number
  /** Box 14 - Employment income (dollars) */
  employment_income: number
  /** Box 16 - Employee's CPP contributions (dollars) */
  cpp_contributions: number
  /** Box 16a - Employee's CPP2 contributions (dollars) */
  cpp2_contributions: number
  /** Box 18 - Employee's EI premiums (dollars) */
  ei_premiums: number
  /** Box 20 - RPP contributions (dollars) */
  rpp_contributions: number
  /** Box 22 - Income tax deducted (dollars) */
  income_tax_deducted: number
  /** Box 24 - EI insurable earnings (dollars) */
  ei_insurable_earnings: number
  /** Box 26 - CPP pensionable earnings (dollars) */
  cpp_pensionable_earnings: number
  /** Box 45 - Employer-offered dental benefit (1=No, 2=Basic, 3=Comprehensive) */
  dental_benefit: number
  /** Box 52 - Pension adjustment (dollars) */
  pension_adjustment: number
  employment_code: string | null
  province_of_employment: string
  /** Net pay from payroll history (ground truth - sum of net_pay from payroll records) */
  net_pay: number
  /** Computed net pay from T4 box values: Box 14 - Box 16 - Box 16a - Box 18 - Box 22 - Box 20 */
  computed_net_pay: number
}

/** T4 box value update - applies adjustment diffs to box values */
export interface T4BoxValueUpdate {
  employee_id: number
  year: number
  box_14_adjustment: number
  box_16_adjustment: number
  box_16a_adjustment: number
  box_18_adjustment: number
  box_20_adjustment: number
  box_22_adjustment: number
  box_24_adjustment: number
  box_26_adjustment: number
  box_45_adjustment: number
  box_52_adjustment: number
}
