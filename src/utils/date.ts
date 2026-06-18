/**
 * Date utility functions to handle timezone-safe date operations
 */

/**
 * Format a date string (YYYY-MM-DD) for display without timezone shifts
 * @param dateStr - Date string in YYYY-MM-DD format
 * @returns Formatted date string for display
 */
export function formatDateLocal(dateStr: string): string {
  if (!dateStr) return ''
  
  // Parse date as local date to avoid timezone shifts
  // For dates like "2024-01-15", split and create local date
  const parts = dateStr.split('-')
  if (parts.length === 3) {
    const year = parseInt(parts[0], 10)
    const month = parseInt(parts[1], 10) - 1 // months are 0-indexed
    const day = parseInt(parts[2], 10)
    return new Date(year, month, day).toLocaleDateString()
  }
  
  return new Date(dateStr).toLocaleDateString()
}

/**
 * Convert a Date object to YYYY-MM-DD format without timezone shifts
 * @param date - Date object
 * @returns Date string in YYYY-MM-DD format
 */
export function toDateString(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

/**
 * Parse a date string (YYYY-MM-DD) to a Date object without timezone shifts
 * @param dateStr - Date string in YYYY-MM-DD format
 * @returns Date object
 */
export function parseLocalDate(dateStr: string): Date {
  const parts = dateStr.split('-')
  if (parts.length === 3) {
    const year = parseInt(parts[0], 10)
    const month = parseInt(parts[1], 10) - 1 // months are 0-indexed
    const day = parseInt(parts[2], 10)
    return new Date(year, month, day)
  }
  return new Date(dateStr)
}
