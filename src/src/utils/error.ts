// Utility functions for error handling

/**
 * Extract a readable error message from any error object
 * Handles Tauri-specific error structures
 */
export function getErrorMessage(error: unknown): string {
  // Handle Error instances
  if (error instanceof Error) {
    return error.message
  }
  
  // Handle Tauri error objects (they have a specific structure)
  if (error && typeof error === 'object') {
    // Tauri errors often come as { message: string } or have other properties
    const err = error as Record<string, any>
    
    // Check for common error properties
    if ('message' in err && typeof err.message === 'string') {
      return err.message
    }
    
    // Some Tauri errors might be in a nested structure
    if ('error' in err && typeof err.error === 'string') {
      return err.error
    }
    
    // Try to JSON stringify the object as last resort
    try {
      return JSON.stringify(error)
    } catch {
      return 'Unknown error occurred'
    }
  }
  
  // Handle string errors
  if (typeof error === 'string') {
    return error
  }
  
  // Fallback
  return String(error)
}
