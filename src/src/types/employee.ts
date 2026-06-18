// TypeScript types mirroring Rust models from cpr-core

export interface Employee {
  id?: number
  is_active: boolean
  employee_number: string
  first_name: string
  last_name: string
  sin: string
  date_of_birth: string
  address: Address
  hire_date: string
  hire_province: string  // Province for tax calculations (independent of address province)
  termination_date?: string | null
  pay_type: PayType
  pay_rate: number
  vacation_pay_rate: number
  vacation_balance?: number
  vacation_balance_days?: number
  overtime_multiplier: number
  ei_exempt?: boolean  // Whether employee is exempt from EI deductions
  cpp_exempt?: boolean  // Whether employee is exempt from CPP deductions
  dental_benefit?: number  // 1, 2, or 3 (default: 1) - T4 Box 45
  additional_tax_amount?: number
  notes?: string
  created_at?: string
}

export interface Address {
  street: string
  city: string
  province: string
  postal_code: string
}

export enum Province {
  ON = 'ON',
  QC = 'QC',
  BC = 'BC',
  AB = 'AB',
  MB = 'MB',
  SK = 'SK',
  NS = 'NS',
  NB = 'NB',
  NL = 'NL',
  PE = 'PE',
  NT = 'NT',
  YT = 'YT',
  NU = 'NU'
}

export enum PayType {
  Hourly = 'Hourly',
  Annual = 'Annual',
  Weekly = 'Weekly',
  Monthly = 'Monthly'
}


export interface PayRateHistory {
  id: number
  employee_id: number
  pay_rate: number
  pay_type: PayType
  effective_date: string
  end_date?: string | null
  reason?: string
  created_at: string
}

export interface EmploymentHistory {
  id: number
  employee_id: number
  hire_date: string
  termination_date?: string | null
  termination_reason?: string
  rehire_eligible: boolean
  notes?: string
  created_at: string
}

export interface YtdTotals {
  employee_id: number
  year: number
  gross_pay: number
  cpp: number
  ei: number
  federal_tax: number
  provincial_tax: number
  net_pay: number
}

export interface PersonalAmount {
  id?: number
  employee_id: number
  province: string
  year: number
  federal_amount: number
  provincial_amount: number
  indexed_at?: string
}

export enum AutofillType {
  Earning = 'earning',
  Deduction = 'deduction'
}

export interface EmployeeAutofill {
  id?: number
  employee_id: number
  autofill_type: AutofillType
  type_name: string
  amount: number
  is_active: boolean
  created_at?: string | null
}
