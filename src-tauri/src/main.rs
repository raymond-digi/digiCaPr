// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod state;

use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Database commands
            commands::database::create_database,
            commands::database::open_database,
            commands::database::close_database,
            commands::database::get_current_database_path,
            commands::database::is_database_open,
            
            // Employee commands
            commands::employee::list_employees,
            commands::employee::get_employee,
            commands::employee::create_employee,
            commands::employee::update_employee,
            commands::employee::delete_employee,
            commands::employee::search_employees,
            commands::employee::get_pay_rate_history,
            commands::employee::get_employment_history,
            commands::employee::list_active_employees,
            commands::employee::get_personal_amount,
            commands::employee::get_personal_amounts,
            commands::employee::get_latest_personal_amount_by_province,
            commands::employee::create_personal_amount,
            commands::employee::update_personal_amount,
            commands::employee::get_basic_personal_amounts,
            commands::employee::get_available_tax_years,
            commands::employee::export_employees_csv,
            commands::employee::import_employees_csv,
            commands::employee::get_employee_autofill,
            commands::employee::get_active_employee_autofill,
            commands::employee::save_employee_autofill,
            commands::employee::delete_employee_autofill,
            commands::employee::delete_all_employee_autofill,
            commands::employee::get_tax_rates,
            
            // Payroll commands
            commands::payroll::calculate_payroll,
            commands::payroll::save_payroll,
            commands::payroll::list_payroll,
            commands::payroll::list_payroll_history,
            commands::payroll::list_employee_payroll,
            commands::payroll::get_payroll,
            commands::payroll::get_ytd_totals,
            commands::payroll::delete_payroll,
            commands::payroll::create_current_payroll,
            commands::payroll::get_available_employees_for_current_payroll,
            commands::payroll::check_history_payroll_dates_exist,
            commands::payroll::post_current_to_history,
            commands::payroll::get_current_payroll_dates,
            commands::payroll::list_current_payroll,
            commands::payroll::clear_current_payroll,
            commands::payroll::update_current_payroll,
            commands::payroll::add_to_current_payroll,
            commands::payroll::export_current_payroll_csv,
            commands::payroll::import_current_payroll_csv,
            commands::payroll::import_history_payroll_csv,
            commands::payroll::export_history_payroll_csv,
            commands::payroll::delete_history_payroll,
            commands::payroll::update_history_payroll,
            commands::payroll::save_raw_payroll,
            commands::payroll::list_payroll_years,
            commands::payroll::list_payroll_periods,
            
            // Remittance commands
            commands::remittance::get_remittance_summary,
            commands::remittance::create_remittance,
            commands::remittance::list_remittances,
            commands::remittance::get_remittance_years,
            commands::remittance::get_remittance,
            commands::remittance::delete_remittance,
            
            // Company commands
            commands::company::get_company,
            commands::company::save_company,
            
            // Report commands
            commands::reports::generate_payroll_report,
            commands::reports::generate_history_payroll_report,
            commands::reports::generate_payroll_paystubs,
            commands::reports::generate_payroll_paystub,
            commands::reports::generate_t4,
            commands::reports::export_payroll_csv,
            commands::reports::generate_payroll_t4,
            commands::reports::generate_remittance_report,
            commands::reports::generate_personal_amount_report,
            commands::reports::calculate_t4_for_year,
            commands::reports::generate_t4_summary_pdf,
            commands::reports::get_t4_summary,
            commands::reports::export_t4_xml,
            commands::reports::export_t4_csv,
            
            // Registry commands
            commands::registry::registry_set,
            commands::registry::registry_get,
            commands::registry::registry_delete,
            commands::registry::registry_exists,
            commands::registry::registry_list_keys,
            commands::registry::registry_get_all,
            commands::registry::registry_delete_all,
            
            // T4 commands (flexible schema)
            commands::t4::list_t4_slips_for_year,
            commands::t4::get_or_create_t4_slip,
            commands::t4::create_t4_slip_version,
            commands::t4::get_t4_box_values,
            commands::t4::save_t4_box_value,
            commands::t4::calculate_t4_for_year_v2,
            commands::t4::file_t4_slip,
            commands::t4::lock_t4_slip,
            commands::t4::unlock_t4_slip,
            commands::t4::get_t4_years,
            commands::t4::get_t4_slips_for_year,
            commands::t4::update_t4_box_values,
            
            // Vacation commands
            commands::vacation::get_vacation_balance,
            commands::vacation::get_vacation_history,
            commands::vacation::record_vacation_accrual,
            commands::vacation::record_vacation_adjustment,
            commands::vacation::create_vacation_time_off,
            commands::vacation::update_vacation_time_off,
            commands::vacation::delete_vacation_time_off,
            commands::vacation::get_vacation_time_off_history,

            // Update commands
            commands::update::check_for_updates,
            commands::update::install_update,
            commands::update::check_config_updates,
            commands::update::download_config_update,
            
            // Recent database commands
            commands::recent::get_recent_databases,
            commands::recent::add_recent_database,
            commands::recent::remove_recent_database,
            commands::recent::update_recent_database_company,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
