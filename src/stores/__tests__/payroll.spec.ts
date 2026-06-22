import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePayrollStore } from '../historyPayroll'
import { invoke } from '@tauri-apps/api/core'
import type { Payroll } from '@/types/payroll'

vi.mock('@tauri-apps/api/core')

describe('Payroll Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = usePayrollStore()
    expect(store.payrolls).toEqual([])
    expect(store.currentPayroll).toBe(null)
    expect(store.loading).toBe(false)
  })

  it('calculates payroll successfully', async () => {
    const mockPayroll: Payroll = {
      id: 1,
      employee_id: 1,
      pay_period_start: '2024-01-01',
      pay_period_end: '2024-01-15',
      pay_date: '2024-01-20',
      regular_hours: 80,
      additional_earnings: [],
      insured_earning: 2000,
      gross_pay: 2000,
      additional_earnings_total: 0,
      additional_tax_amount: 0,
      deductions: {
        cpp: 100,
        cpp2: 0,
        ei: 30,
        federal_tax: 300,
        provincial_tax: 150,
        additional: [],
      },
      net_pay: 1420,
      additional_deductions: 0,
      federal_personal_amount: 0,
      provincial_personal_amount: 0,
      province: 'ON',
      total_pay_periods: 26,
      created_at: '2024-01-20T00:00:00Z',
    }

    vi.mocked(invoke).mockResolvedValueOnce(mockPayroll)

    const store = usePayrollStore()
    const result = await store.calculatePayroll({
      employee_id: 1,
      pay_period_start: '2024-01-01',
      pay_period_end: '2024-01-15',
      pay_date: '2024-01-20',
    })

    expect(invoke).toHaveBeenCalledWith('calculate_payroll', expect.any(Object))
    expect(result).toEqual(mockPayroll)
    expect(store.currentPayroll).toEqual(mockPayroll)
  })

  it('fetches payroll records', async () => {
    const mockPayrolls: Payroll[] = [
      {
        id: 1,
        employee_id: 1,
        pay_period_start: '2024-01-01',
        pay_period_end: '2024-01-15',
        pay_date: '2024-01-20',
        regular_hours: 80,
        additional_earnings: [],
        insured_earning: 2000,
        gross_pay: 2000,
        additional_earnings_total: 0,
        additional_tax_amount: 0,
        deductions: {
          cpp: 100,
          cpp2: 0,
          ei: 30,
          federal_tax: 300,
          provincial_tax: 150,
          additional: [],
        },
        net_pay: 1420,
        additional_deductions: 0,
        federal_personal_amount: 0,
        provincial_personal_amount: 0,
        province: 'ON',
        total_pay_periods: 26,
        created_at: '2024-01-20T00:00:00Z',
      },
    ]

    vi.mocked(invoke).mockResolvedValueOnce({ payrolls: mockPayrolls, total_count: mockPayrolls.length })

    const store = usePayrollStore()
    await store.fetchPayrolls({ employee_id: 1 })

    expect(invoke).toHaveBeenCalledWith('list_payroll_history', { filters: { employee_id: 1 } })
    expect(store.payrolls).toEqual(mockPayrolls)
  })

  it('saves payroll successfully', async () => {
    const payrollToSave: Payroll = {
      employee_id: 1,
      pay_period_start: '2024-01-01',
      pay_period_end: '2024-01-15',
      pay_date: '2024-01-20',
      regular_hours: 80,
      additional_earnings: [],
      insured_earning: 2000,
      gross_pay: 2000,
      additional_earnings_total: 0,
      additional_tax_amount: 0,
      deductions: {
        cpp: 100,
        cpp2: 0,
        ei: 30,
        federal_tax: 300,
        provincial_tax: 150,
        additional: [],
      },
      net_pay: 1420,
      additional_deductions: 0,
      federal_personal_amount: 0,
      provincial_personal_amount: 0,
      province: 'ON',
      total_pay_periods: 26,
      created_at: '2024-01-20T00:00:00Z',
    }

    vi.mocked(invoke).mockResolvedValueOnce(1) // payroll ID
    vi.mocked(invoke).mockResolvedValueOnce([]) // fetchPayrolls

    const store = usePayrollStore()
    const id = await store.savePayroll(payrollToSave)

    expect(id).toBe(1)
    expect(invoke).toHaveBeenCalledWith('save_payroll', { payroll: payrollToSave })
  })

  it('filters payrolls by employee', () => {
    const store = usePayrollStore()
    store.payrolls = [
      {
        id: 1,
        employee_id: 1,
        pay_period_start: '2024-01-01',
        pay_period_end: '2024-01-15',
        pay_date: '2024-01-20',
        regular_hours: 80,
        additional_earnings: [],
        insured_earning: 2000,
        gross_pay: 2000,
        additional_earnings_total: 0,
        additional_tax_amount: 0,
        deductions: { cpp: 100, cpp2: 0, ei: 30, federal_tax: 300, provincial_tax: 150, additional: [] },
        net_pay: 1420,
        additional_deductions: 0,
        federal_personal_amount: 0,
        provincial_personal_amount: 0,
        province: 'ON',
        total_pay_periods: 26,
        created_at: '2024-01-20T00:00:00Z',
      },
      {
        id: 2,
        employee_id: 2,
        pay_period_start: '2024-01-01',
        pay_period_end: '2024-01-15',
        pay_date: '2024-01-20',
        regular_hours: 80,
        additional_earnings: [],
        insured_earning: 3000,
        gross_pay: 3000,
        additional_earnings_total: 0,
        additional_tax_amount: 0,
        deductions: { cpp: 150, cpp2: 0, ei: 45, federal_tax: 450, provincial_tax: 225, additional: [] },
        net_pay: 2130,
        additional_deductions: 0,
        federal_personal_amount: 0,
        provincial_personal_amount: 0,
        province: 'ON',
        total_pay_periods: 26,
        created_at: '2024-01-20T00:00:00Z',
      },
    ]

    const employeePayrolls = store.getPayrollsByEmployee(1)
    expect(employeePayrolls).toHaveLength(1)
    expect(employeePayrolls[0].employee_id).toBe(1)
  })
})
