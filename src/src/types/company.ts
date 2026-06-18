// TypeScript types for Company

export interface Company {
  id?: number
  name: string
  business_number: string
  address: CompanyAddress
  payroll_account_number: string
  contact_person: string
  phone: string
  email: string
  created_at?: string
  updated_at?: string
}

export interface CompanyAddress {
  street: string
  city: string
  province: string
  postal_code: string
}
