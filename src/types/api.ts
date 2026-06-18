// API response and error types

export interface CommandError {
  message: string
}

export interface ApiResponse<T> {
  data?: T
  error?: CommandError
}

export interface ReportGenerationParams {
  output_path: string
}

export interface T4Params extends ReportGenerationParams {
  employee_id: number
  year: number
}

export interface PaystubParams extends ReportGenerationParams {
  payroll_id: number
}
