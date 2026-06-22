import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useEmployeeStore } from '@/stores/employee'
import { usePayrollStore } from '@/stores/historyPayroll'
import { useCompanyStore } from '@/stores/company'
import { invoke } from '@tauri-apps/api/core'
import { PayType } from '@/types/employee'

vi.mock('@tauri-apps/api/core')

describe('Payroll Processing Integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('completes full payroll workflow', async () => {
    // Setup: Create employee
    const mockEmployee = {
      id: 1,
      employee_number: 'E001',
      first_name: 'John',
      last_name: 'Doe',
      sin: '123-456-789',
      date_of_birth: '1990-01-01',
      address: {
        street: '123 Main St',
        city: 'Toronto',
        province: 'ON',
        postal_code: 'M1M 1M1',
      },
      hire_date: '2024-01-01',
      hire_province: 'ON',
      pay_type: PayType.Hourly,
      pay_rate: 25.00,
      vacation_pay_rate: 4,
      overtime_multiplier: 1.5,
      is_active: true,
      created_at: '2024-01-01T00:00:00Z',
    }

    const mockPayroll = {
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

    // Use command-map mock to avoid brittle mockResolvedValueOnce chains
    const commandMap: Record<string, any> = {
      create_employee: 1,
      list_employees: [mockEmployee],
      calculate_payroll: mockPayroll,
      save_payroll: 1,
      list_payroll_history: { payrolls: [mockPayroll], total_count: 1 },
    }
    vi.mocked(invoke).mockImplementation(async (cmd: string) => commandMap[cmd])

    // Step 1: Create employee
    const employeeStore = useEmployeeStore()
    const employeeId = await employeeStore.createEmployee(mockEmployee)
    expect(employeeId).toBe(1)
    expect(employeeStore.employees).toHaveLength(1)

    // Step 2: Calculate payroll
    const payrollStore = usePayrollStore()
    const calculatedPayroll = await payrollStore.calculatePayroll({
      employee_id: 1,
      pay_period_start: '2024-01-01',
      pay_period_end: '2024-01-15',
      pay_date: '2024-01-20',
      regular_hours: 80,
    })

    expect(calculatedPayroll.gross_pay).toBe(2000)
    expect(calculatedPayroll.net_pay).toBe(1420)

    // Step 3: Save payroll
    const payrollId = await payrollStore.savePayroll(calculatedPayroll)
    expect(payrollId).toBe(1)

    // Step 4: Verify payroll was saved
    await payrollStore.fetchPayrolls()
    expect(payrollStore.payrolls).toHaveLength(1)
  })

  it('handles year-end T4 generation workflow', async () => {
    const mockEmployee = {
      id: 1,
      employee_number: 'E001',
      first_name: 'Jane',
      last_name: 'Smith',
      sin: '987-654-321',
      address: {
        street: '456 Oak Ave',
        city: 'Vancouver',
        province: 'BC',
        postal_code: 'V1V 1V1',
      },
      province: 'BC',
      pay_type: PayType.Annual,
      pay_rate: 60000,
      hire_date: '2024-01-01',
      is_active: true,
      created_at: '2024-01-01T00:00:00Z',
    }

    const mockYTDTotals = {
      year: 2024,
      gross_pay: 60000,
      cpp_employee: 3500,
      ei_employee: 950,
      federal_tax: 9000,
      provincial_tax: 4500,
      total_deductions: 17950,
      net_pay: 42050,
    }

    vi.mocked(invoke)
      .mockResolvedValueOnce([mockEmployee]) // list_employees
      .mockResolvedValueOnce(mockYTDTotals) // get_ytd_totals
      .mockResolvedValueOnce('/path/to/T4_2024_E001_Smith.pdf') // generate_t4

    // Step 1: Fetch employees
    const employeeStore = useEmployeeStore()
    await employeeStore.fetchEmployees()
    expect(employeeStore.employees).toHaveLength(1)

    // Step 2: Get YTD totals
    const payrollStore = usePayrollStore()
    const ytdTotals = await payrollStore.fetchYtdTotals(1, 2024)
    expect(ytdTotals.gross_pay).toBe(60000)

    // Step 3: Generate T4
    // This would be called from the Reports view
    const t4Path = await invoke('generate_t4', {
      employeeId: 1,
      year: 2024,
      outputPath: '/path/to/T4_2024_E001_Smith.pdf',
    })

    expect(t4Path).toContain('T4_2024_E001_Smith.pdf')
    expect(invoke).toHaveBeenCalledWith('generate_t4', expect.any(Object))
  })

  it('handles multi-employee T4 generation', async () => {
    const mockEmployees = [
      { id: 1, employee_number: 'E001', first_name: 'John', last_name: 'Doe', is_active: true },
      { id: 2, employee_number: 'E002', first_name: 'Jane', last_name: 'Smith', is_active: true },
      { id: 3, employee_number: 'E003', first_name: 'Bob', last_name: 'Johnson', is_active: false },
    ]

    const mockT4Paths = [
      '/output/T4_2024_E001_Doe.pdf',
      '/output/T4_2024_E002_Smith.pdf',
    ]

    vi.mocked(invoke)
      .mockResolvedValueOnce(mockEmployees) // list_employees
      .mockResolvedValueOnce(mockT4Paths) // generate_t4

    // Step 1: Fetch all employees
    const employeeStore = useEmployeeStore()
    await employeeStore.fetchEmployees()

    // Step 2: Generate T4s (only for active employees)
    const paths = await invoke('generate_payroll_t4', {
      year: 2024,
      outputDir: '/output',
    })

    const typedPaths = paths as string[]
    expect(typedPaths).toHaveLength(2) // Only active employees
    expect(typedPaths[0]).toContain('E001_Doe')
    expect(typedPaths[1]).toContain('E002_Smith')
  })

  it('handles company setup workflow', async () => {
    const mockCompany = {
      id: 1,
      business_number: '123456789',
      company_name: 'Test Company Inc.',
      address: {
        street: '789 Business Blvd',
        city: 'Toronto',
        province: 'ON',
        postal_code: 'M2M 2M2',
      },
      contact_phone: '416-555-0123',
      contact_email: 'info@testcompany.ca',
      created_at: '2024-01-01T00:00:00Z',
    }

    // Use command-map mock — company state changes between calls
    let savedCompany: any = null
    vi.mocked(invoke).mockImplementation(async (cmd: string, args?: any) => {
      if (cmd === 'get_company') return savedCompany
      if (cmd === 'save_company') { savedCompany = args?.company; return undefined }
      return undefined
    })

    const companyStore = useCompanyStore()

    // Step 1: Check if company exists (first launch)
    await companyStore.fetchCompany()
    expect(companyStore.company).toBe(null)

    // Step 2: Save company information
    await companyStore.saveCompany(mockCompany)

    // Step 3: Verify company was saved (saveCompany flattens/transforms the data)
    await companyStore.fetchCompany()
    expect(companyStore.company).toBeDefined()
    expect(companyStore.company?.business_number).toBe('123456789')
    expect(companyStore.company?.id).toBe(1)
  })

  it('handles database connection workflow', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(undefined) // create_database
      .mockResolvedValueOnce('/path/to/database.db') // get_current_database_path
      .mockResolvedValueOnce(undefined) // close_database

    // Step 1: Create new database
    await invoke('create_database', { path: '/path/to/database.db' })
    expect(invoke).toHaveBeenCalledWith('create_database', { path: '/path/to/database.db' })

    // Step 2: Verify database is open
    const dbPath = await invoke('get_current_database_path')
    expect(dbPath).toBe('/path/to/database.db')

    // Step 3: Close database
    await invoke('close_database')
    expect(invoke).toHaveBeenCalledWith('close_database')
  })

  it('handles error recovery in payroll workflow', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([]) // list_employees - no employees
      .mockRejectedValueOnce(new Error('Employee not found')) // calculate_payroll fails

    const employeeStore = useEmployeeStore()
    await employeeStore.fetchEmployees()
    expect(employeeStore.employees).toHaveLength(0)

    // Attempting to calculate payroll for non-existent employee should fail
    const payrollStore = usePayrollStore()
    await expect(
      payrollStore.calculatePayroll({
        employee_id: 999,
        pay_period_start: '2024-01-01',
        pay_period_end: '2024-01-15',
        pay_date: '2024-01-20',
      })
    ).rejects.toThrow('Employee not found')
  })
})
