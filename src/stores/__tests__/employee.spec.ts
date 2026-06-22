import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useEmployeeStore } from '../employee'
import { invoke } from '@tauri-apps/api/core'
import type { Employee } from '@/types/employee'
import { PayType } from '@/types/employee'

vi.mock('@tauri-apps/api/core')

describe('Employee Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useEmployeeStore()
    expect(store.employees).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.error).toBe(null)
  })

  it('fetches employees successfully', async () => {
    const mockEmployees: Employee[] = [
      {
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
        pay_type: PayType.Annual,
        pay_rate: 50000,
        vacation_pay_rate: 4,
        overtime_multiplier: 1.5,
        is_active: true,
        created_at: '2024-01-01T00:00:00Z',
      },
    ]

    vi.mocked(invoke).mockResolvedValueOnce(mockEmployees)

    const store = useEmployeeStore()
    await store.fetchEmployees()

    expect(invoke).toHaveBeenCalledWith('list_employees')
    expect(store.employees).toEqual(mockEmployees)
    expect(store.loading).toBe(false)
    expect(store.error).toBe(null)
  })

  it('handles fetch employees error', async () => {
    const errorMessage = 'Database connection failed'
    vi.mocked(invoke).mockRejectedValueOnce(new Error(errorMessage))

    const store = useEmployeeStore()
    
    await expect(store.fetchEmployees()).rejects.toThrow(errorMessage)
    expect(store.error).toBe(errorMessage)
    expect(store.loading).toBe(false)
  })

  it('creates employee successfully', async () => {
    const newEmployee: Employee = {
      employee_number: 'E002',
      first_name: 'Jane',
      last_name: 'Smith',
      sin: '987-654-321',
      date_of_birth: '1992-05-15',
      address: {
        street: '456 Oak Ave',
        city: 'Vancouver',
        province: 'BC',
        postal_code: 'V1V 1V1',
      },
      hire_date: '2024-02-01',
      hire_province: 'BC',
      pay_type: PayType.Hourly,
      pay_rate: 25.50,
      vacation_pay_rate: 4,
      overtime_multiplier: 1.5,
      is_active: true,
      created_at: new Date().toISOString(),
    }

    vi.mocked(invoke).mockResolvedValueOnce(2) // employee ID
    vi.mocked(invoke).mockResolvedValueOnce([]) // fetchEmployees

    const store = useEmployeeStore()
    const id = await store.createEmployee(newEmployee)

    expect(id).toBe(2)
    expect(invoke).toHaveBeenCalledWith('create_employee', { employee: newEmployee })
  })

  it('filters active employees', () => {
    const store = useEmployeeStore()
    store.employees = [
      {
        id: 1,
        employee_number: 'E001',
        first_name: 'John',
        last_name: 'Doe',
        sin: '123-456-789',
        date_of_birth: '1990-01-01',
        address: { street: '', city: '', province: 'ON', postal_code: '' },
        hire_date: '2024-01-01',
        hire_province: 'ON',
        pay_type: PayType.Annual,
        pay_rate: 50000,
        vacation_pay_rate: 4,
        overtime_multiplier: 1.5,
        is_active: true,
        created_at: '2024-01-01T00:00:00Z',
      },
      {
        id: 2,
        employee_number: 'E002',
        first_name: 'Jane',
        last_name: 'Smith',
        sin: '987-654-321',
        date_of_birth: '1992-05-15',
        address: { street: '', city: '', province: 'BC', postal_code: '' },
        hire_date: '2023-01-01',
        hire_province: 'BC',
        pay_type: PayType.Hourly,
        pay_rate: 25.50,
        vacation_pay_rate: 4,
        overtime_multiplier: 1.5,
        termination_date: '2024-01-01',
        is_active: false,
        created_at: '2023-01-01T00:00:00Z',
      },
    ]

    const activeEmployees = store.activeEmployees
    expect(activeEmployees).toHaveLength(1)
    expect(activeEmployees[0].employee_number).toBe('E001')
  })

  it('gets employee by ID', () => {
    const store = useEmployeeStore()
    store.employees = [
      {
        id: 1,
        employee_number: 'E001',
        first_name: 'John',
        last_name: 'Doe',
        sin: '123-456-789',
        date_of_birth: '1990-01-01',
        address: { street: '', city: '', province: 'ON', postal_code: '' },
        hire_date: '2024-01-01',
        hire_province: 'ON',
        pay_type: PayType.Annual,
        pay_rate: 50000,
        vacation_pay_rate: 4,
        overtime_multiplier: 1.5,
        is_active: true,
        created_at: '2024-01-01T00:00:00Z',
      },
    ]

    const employee = store.getEmployeeById(1)
    expect(employee).toBeDefined()
    expect(employee?.employee_number).toBe('E001')

    const notFound = store.getEmployeeById(999)
    expect(notFound).toBeUndefined()
  })
})
