use csv::Writer;
use printpdf::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use cpr_core::models::payroll::YtdTotals;
use cpr_core::models::{Company, Employee};

/// T619 Transmitter information for CRA Internet File Transfer
/// Stored in registry with key paths like "transmitter/bn15", "transmitter/name", etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T619Transmitter {
    /// BN15 with program identifier (e.g., "882504012RP0001")
    pub bn15: String,
    /// Transmitter name
    pub name: String,
    /// Contact person name
    pub contact_name: String,
    /// Phone area code (e.g., "416")
    pub phone_area: String,
    /// Phone number (e.g., "321-7654")
    pub phone: String,
    /// Contact email (optional)
    pub email: Option<String>,
    /// Submission reference ID (optional)
    pub submission_ref_id: Option<String>,
}

impl T619Transmitter {
    /// Create a new transmitter with default values
    pub fn new() -> Self {
        Self {
            bn15: String::new(),
            name: String::new(),
            contact_name: String::new(),
            phone_area: String::new(),
            phone: String::new(),
            email: None,
            submission_ref_id: None,
        }
    }
}

/// T4 slip data for an employee
#[derive(Debug, Clone, Serialize)]
pub struct T4Data {
    pub employee: Employee,
    pub year: i32,
    pub employment_income: Decimal,        // Box 14
    pub cpp_contributions: Decimal,        // Box 16 - Employee's CPP
    pub cpp2_contributions: Decimal,       // Box 16a - Employee's CPP2
    pub rpp_contributions: Decimal,        // Box 20 - RPP contributions
    pub ei_premiums: Decimal,              // Box 18
    pub income_tax_deducted: Decimal,      // Box 22
    pub ei_insurable_earnings: Decimal,    // Box 24 - EI insurable earnings
    pub cpp_pensionable_earnings: Decimal, // Box 26 - CPP pensionable earnings
    pub pension_adjustment: Decimal,       // Box 52 - Pension adjustment
    pub dental_benefit: i32,               // Box 45 - Employer-offered dental (1, 2, or 3)
    pub employment_code: Option<String>,
    pub province_of_employment: String,
    /// Net pay from payroll history (ground truth - sum of net_pay from payroll records)
    pub net_pay: Decimal,
    /// Computed net pay from T4 box values: Box 14 - Box 16 - Box 16a - Box 18 - Box 22 - Box 20
    /// Used to compare against net_pay to detect discrepancies
    pub computed_net_pay: Decimal,
}

impl T4Data {
    /// Create T4 data from YTD totals
    pub fn from_ytd_totals(employee: Employee, ytd: YtdTotals) -> Self {
        // Box 24 - EI insurable earnings = gross pay (all earnings are insurable per T4127)
        let ei_insurable_earnings = ytd.gross_pay;

        // Box 26 - CPP pensionable earnings = gross pay - CPP basic exemption
        // Note: Using annual exemption; for part-year employees, this should be prorated
        // CPP basic exemption is $3,500/year (2024)
        let cpp_exemption = rust_decimal_macros::dec!(3500.00);
        let cpp_pensionable_earnings = (ytd.gross_pay - cpp_exemption).max(rust_decimal_macros::dec!(0));

        // Box 45 - Dental benefit code from employee record
        let dental_benefit = employee.dental_benefit;

        // Computed net pay from box values for comparison
        let income_tax = ytd.federal_tax + ytd.provincial_tax;
        let computed_net_pay = ytd.gross_pay - ytd.cpp - ytd.cpp2 - ytd.ei - income_tax - ytd.rpp_contributions;

        // Use employee_number as the employment code for CRA T4 reporting
        let employment_code = Some(employee.employee_number.clone());

        Self {
            province_of_employment: employee.hire_province.to_string(),
            employee,
            year: ytd.year,
            employment_income: ytd.gross_pay,
            cpp_contributions: ytd.cpp,
            cpp2_contributions: ytd.cpp2,
            rpp_contributions: ytd.rpp_contributions,
            ei_premiums: ytd.ei,
            income_tax_deducted: income_tax,
            ei_insurable_earnings,
            cpp_pensionable_earnings,
            pension_adjustment: ytd.pension_adjustment,
            dental_benefit,
            employment_code,
            net_pay: ytd.net_pay,
            computed_net_pay,
        }
    }
}

/// Generate a T4 slip PDF
pub fn generate_t4<P: AsRef<Path>>(output_path: P, t4_data: &T4Data, company: &Company) -> Result<(), Box<dyn std::error::Error>> {
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "T4 Statement of Remuneration Paid",
        Mm(215.9), // Letter size width
        Mm(279.4), // Letter size height
        "Layer 1",
    );

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load fonts
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let mut y_position = 260.0; // Start from top

    // Title
    current_layer.use_text("T4 - Statement of Remuneration Paid", 16.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;

    current_layer.use_text(&format!("Year: {}", t4_data.year), 12.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 20.0;

    // Employer information
    current_layer.use_text("EMPLOYER INFORMATION", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;

    current_layer.use_text(&format!("Name: {}", company.name), 10.0, Mm(20.0), Mm(y_position), &font_regular);
    y_position -= 6.0;

    if let Some(ref bn) = company.business_number {
        current_layer.use_text(&format!("Business Number: {}", bn), 10.0, Mm(20.0), Mm(y_position), &font_regular);
        y_position -= 6.0;
    }

    current_layer.use_text(&format!("Address: {}", company.address), 10.0, Mm(20.0), Mm(y_position), &font_regular);
    y_position -= 15.0;

    // Employee information
    current_layer.use_text("EMPLOYEE INFORMATION", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;

    current_layer.use_text(&format!("Name: {} {}", t4_data.employee.first_name, t4_data.employee.last_name), 10.0, Mm(20.0), Mm(y_position), &font_regular);
    y_position -= 6.0;

    current_layer.use_text(&format!("Social Insurance Number: {}", format_sin(&t4_data.employee.sin)), 10.0, Mm(20.0), Mm(y_position), &font_regular);
    y_position -= 6.0;

    current_layer.use_text(
        &format!(
            "Address: {}, {}, {} {}",
            t4_data.employee.address.street, t4_data.employee.address.city, t4_data.employee.address.province, t4_data.employee.address.postal_code
        ),
        10.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    y_position -= 15.0;

    // T4 Amounts
    current_layer.use_text("REMUNERATION AND DEDUCTIONS", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 10.0;

    // Box 14 - Employment income
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Box 14 - Employment income", &format_currency(t4_data.employment_income));
    y_position -= 12.0;

    // Box 16 - Employee's CPP contributions (CPP only)
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Box 16 - Employee's CPP contributions",
        &format_currency(t4_data.cpp_contributions),
    );
    y_position -= 12.0;

    // Box 16A - Employee's CPP2 contributions
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Box 16A - Employee's CPP2 contributions",
        &format_currency(t4_data.cpp2_contributions),
    );
    y_position -= 12.0;

    // Box 18 - Employee's EI premiums
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Box 18 - Employee's EI premiums", &format_currency(t4_data.ei_premiums));
    y_position -= 12.0;

    // Box 20 - RPP contributions
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Box 20 - RPP contributions", &format_currency(t4_data.rpp_contributions));
    y_position -= 12.0;

    // Box 22 - Income tax deducted
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Box 22 - Income tax deducted", &format_currency(t4_data.income_tax_deducted));
    y_position -= 12.0;

    // Box 24 - EI insurable earnings
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Box 24 - EI insurable earnings",
        &format_currency(t4_data.ei_insurable_earnings),
    );
    y_position -= 12.0;

    // Box 26 - CPP pensionable earnings
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Box 26 - CPP pensionable earnings",
        &format_currency(t4_data.cpp_pensionable_earnings),
    );
    y_position -= 12.0;

    // Box 45 - Employer-offered dental benefit
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Box 52 - Pension adjustment", &format_currency(t4_data.pension_adjustment));
    y_position -= 12.0;

    // Box 45 - Employer-offered dental benefit
    let dental_benefit_str = match t4_data.dental_benefit {
        1 => "1 - No dental benefit",
        2 => "2 - Basic dental coverage",
        3 => "3 - Comprehensive dental coverage",
        _ => "1 - No dental benefit",
    };
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Box 45 - Employer-offered dental benefit", dental_benefit_str);
    y_position -= 12.0;

    // Province of employment
    draw_t4_box(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Province of employment", &t4_data.province_of_employment);
    y_position -= 20.0;

    // Footer notes
    current_layer.use_text(
        "This is a copy of your T4 slip. Please keep it for your records and use it when filing your income tax return.",
        9.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    y_position -= 6.0;

    current_layer.use_text("For more information, visit www.canada.ca/taxes", 9.0, Mm(20.0), Mm(y_position), &font_regular);

    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))?;

    Ok(())
}

/// Estimate text width in mm for Helvetica at 10pt
fn estimate_text_width_mm(text: &str) -> f64 {
    text.chars()
        .map(|c| match c {
            ',' | '.' | ':' | ' ' => 1.0,
            _ => 2.0,
        })
        .sum()
}

/// Draw a T4 box with left-aligned label and right-aligned value
fn draw_t4_box(layer: &PdfLayerReference, font_regular: &IndirectFontRef, font_bold: &IndirectFontRef, x: f64, y: f64, label: &str, value: &str) {
    layer.use_text(label, 10.0, Mm(x), Mm(y), font_regular);
    let text_width = estimate_text_width_mm(value);
    let value_x = RIGHT_MARGIN - text_width;
    layer.use_text(value, 10.0, Mm(value_x), Mm(y), font_bold);
}

/// Format SIN with spaces (XXX XXX XXX)
fn format_sin(sin: &str) -> String {
    let digits: String = sin.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 9 {
        format!("{} {} {}", &digits[0..3], &digits[3..6], &digits[6..9])
    } else {
        sin.to_string()
    }
}

/// Format currency for display with thousand separators
fn format_currency(value: Decimal) -> String {
    let sign = if value < Decimal::ZERO { "-" } else { "" };
    let abs_val = value.abs();
    let formatted = format!("{:.2}", abs_val);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts[1];

    // Add thousand separators to integer part
    let mut result = String::new();
    for (i, c) in int_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    let int_with_seps: String = result.chars().rev().collect();

    format!("{}${}.{}", sign, int_with_seps, dec_part)
}

// ── T4 Summary PDF layout constants ──────────────────────────────────
// Page layout (mm)
const MARGIN_LEFT: f64 = 20.0;
const START_Y: f64 = 260.0;
const RIGHT_MARGIN: f64 = 185.0;

// Font sizes (pt)
const TITLE_SIZE: f64 = 14.0;
const HEADING_SIZE: f64 = 11.0;
const SECTION_SIZE: f64 = 10.0;
const BOX_SIZE: f64 = 10.0;
const FOOTER_SIZE: f64 = 9.0;
const FOOTER_NOTE_SIZE: f64 = 8.0;

// Line heights (mm) – font_size × leading_factor
const TITLE_LEAD: f64 = TITLE_SIZE * 0.5;
const HEADING_LEAD: f64 = HEADING_SIZE * 0.5;
const SECTION_LEAD: f64 = SECTION_SIZE * 0.5;
const BOX_LEAD: f64 = BOX_SIZE * 0.5;
const FOOTER_LEAD: f64 = FOOTER_SIZE * 0.5;

/// T4 Summary data for all employees in a year (T4 Summary boxes)
/// These are the boxes on the T4 Summary (not the T4 slip) that CRA requires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T4SummaryData {
    pub year: i32,
    /// Box 88 - Total number of T4 slips filed
    pub total_slips: i32,
    /// Box 14 - Total employment income
    pub total_employment_income: Decimal,
    /// Box 20 - Total RPP contributions
    pub total_rpp_contributions: Decimal,
    /// Box 52 - Total pension adjustment
    pub total_pension_adjustment: Decimal,
    /// Box 16 - Total employee CPP contributions
    pub total_employee_cpp: Decimal,
    /// Box 16a - Total employee CPP2 contributions
    pub total_employee_cpp2: Decimal,
    /// Box 27 - Total employer CPP contributions (= employee CPP)
    pub total_employer_cpp: Decimal,
    /// Box 27a - Total employer CPP2 contributions (= employee CPP2)
    pub total_employer_cpp2: Decimal,
    /// Box 18 - Total employee EI premiums
    pub total_employee_ei: Decimal,
    /// Box 19 - Total employer EI premiums (= employee EI × 1.4)
    pub total_employer_ei: Decimal,
    /// Box 22 - Total income tax deducted
    pub total_income_tax: Decimal,
    /// Box 80 - Total deductions reported (sum of 16, 16a, 27, 27a, 18, 19, 22)
    pub total_deductions_reported: Decimal,
    /// Box 82 - Total remittances paid for the year
    pub total_remittances_paid: Decimal,
    /// Difference between Box 80 and Box 82
    pub difference: Decimal,
}

impl T4SummaryData {
    /// Calculate T4 summary from T4 slips and total remittances paid
    pub fn calculate(year: i32, t4_slips: &[T4Data], total_remittances: Decimal) -> Self {
        let total_slips = t4_slips.len() as i32;
        let total_employment_income: Decimal = t4_slips.iter().map(|t| t.employment_income).sum();
        let total_rpp_contributions: Decimal = t4_slips.iter().map(|t| t.rpp_contributions).sum();
        let total_pension_adjustment: Decimal = t4_slips.iter().map(|t| t.pension_adjustment).sum();
        let total_employee_cpp: Decimal = t4_slips.iter().map(|t| t.cpp_contributions).sum();
        let total_employee_cpp2: Decimal = t4_slips.iter().map(|t| t.cpp2_contributions).sum();
        // Employer CPP = Employee CPP (they are equal)
        let total_employer_cpp = total_employee_cpp;
        // Employer CPP2 = Employee CPP2 (they are equal)
        let total_employer_cpp2 = total_employee_cpp2;
        let total_employee_ei: Decimal = t4_slips.iter().map(|t| t.ei_premiums).sum();
        // Employer EI = Employee EI × 1.4
        let total_employer_ei = total_employee_ei * rust_decimal_macros::dec!(1.4);
        let total_income_tax: Decimal = t4_slips.iter().map(|t| t.income_tax_deducted).sum();

        // Box 80 - Total deductions reported
        let total_deductions_reported =
            total_employee_cpp + total_employee_cpp2 + total_employer_cpp + total_employer_cpp2 + total_employee_ei + total_employer_ei + total_income_tax;

        // Difference = Box 80 - Box 82
        let difference = total_deductions_reported - total_remittances;

        Self {
            year,
            total_slips,
            total_employment_income,
            total_rpp_contributions,
            total_pension_adjustment,
            total_employee_cpp,
            total_employee_cpp2,
            total_employer_cpp,
            total_employer_cpp2,
            total_employee_ei,
            total_employer_ei,
            total_income_tax,
            total_deductions_reported,
            total_remittances_paid: total_remittances,
            difference,
        }
    }
}

/// Generate T4 Summary (for all employees) - full CRA summary
pub fn generate_t4_summary<P: AsRef<Path>>(output_path: P, year: i32, company: &Company, t4_slips: &[T4Data]) -> Result<(), Box<dyn std::error::Error>> {
    // Calculate summary (without remittance data for standalone PDF generation)
    let summary = T4SummaryData::calculate(year, t4_slips, Decimal::ZERO);

    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new("T4 Summary", Mm(215.9), Mm(279.4), "Layer 1");

    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let mut y_position = START_Y;

    // Title
    current_layer.use_text("T4 Summary of Remuneration Paid", TITLE_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= TITLE_LEAD;

    current_layer.use_text(&format!("Year: {}", year), HEADING_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= HEADING_LEAD * 2.0;

    // Company info
    current_layer.use_text(&format!("Company: {}", company.name), SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_regular);
    y_position -= SECTION_LEAD;

    if let Some(ref bn) = company.business_number {
        current_layer.use_text(&format!("Business Number: {}", bn), SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_regular);
        y_position -= SECTION_LEAD;
    }
    y_position -= SECTION_LEAD;

    // Summary of Remuneration Paid
    current_layer.use_text("SUMMARY OF REMUNERATION PAID", HEADING_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= HEADING_LEAD * 2.0;

    // Box 88 - Number of T4 slips
    draw_t4_box(&current_layer, &font_regular, &font_bold, MARGIN_LEFT, y_position, "Box 88 - Number of T4 slips filed", &summary.total_slips.to_string());
    y_position -= BOX_LEAD;

    // Box 14 - Employment income
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 14 - Employment income",
        &format_currency(summary.total_employment_income),
    );
    y_position -= BOX_LEAD;

    // Box 20 - RPP contributions
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 20 - RPP contributions",
        &format_currency(summary.total_rpp_contributions),
    );
    y_position -= BOX_LEAD;

    // Box 52 - Pension adjustment
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 52 - Pension adjustment",
        &format_currency(summary.total_pension_adjustment),
    );
    y_position -= BOX_LEAD * 2.0;

    // Employee Contributions section
    current_layer.use_text("EMPLOYEE CONTRIBUTIONS", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 16 - Employee CPP
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 16 - Employee's CPP contributions",
        &format_currency(summary.total_employee_cpp),
    );
    y_position -= BOX_LEAD;

    // Box 16a - Employee CPP2
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 16a - Employee's CPP2 contributions",
        &format_currency(summary.total_employee_cpp2),
    );
    y_position -= BOX_LEAD;

    // Box 18 - Employee EI
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 18 - Employee's EI premiums",
        &format_currency(summary.total_employee_ei),
    );
    y_position -= BOX_LEAD * 2.0;

    // Employer Contributions section
    current_layer.use_text("EMPLOYER CONTRIBUTIONS", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 27 - Employer CPP
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 27 - Employer's CPP contributions",
        &format_currency(summary.total_employer_cpp),
    );
    y_position -= BOX_LEAD;

    // Box 27a - Employer CPP2
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 27a - Employer's CPP2 contributions",
        &format_currency(summary.total_employer_cpp2),
    );
    y_position -= BOX_LEAD;

    // Box 19 - Employer EI
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 19 - Employer's EI premiums",
        &format_currency(summary.total_employer_ei),
    );
    y_position -= BOX_LEAD * 2.0;

    // Income Tax Deducted section
    current_layer.use_text("INCOME TAX DEDUCTED", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 22 - Income tax deducted
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 22 - Income tax deducted",
        &format_currency(summary.total_income_tax),
    );
    y_position -= BOX_LEAD * 2.0;

    // Totals section
    current_layer.use_text("TOTALS", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 80 - Total deductions reported
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 80 - Total deductions reported",
        &format_currency(summary.total_deductions_reported),
    );
    y_position -= BOX_LEAD;

    // Box 82 - Total remittances paid
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 82 - Total remittances paid",
        &format_currency(summary.total_remittances_paid),
    );
    y_position -= BOX_LEAD;

    // Difference
    draw_t4_box(&current_layer, &font_regular, &font_bold, MARGIN_LEFT, y_position, "Difference (Box 80 - Box 82)", &format_currency(summary.difference));
    y_position -= HEADING_LEAD;

    // Footer notes
    current_layer.use_text(
        "This is the T4 Summary of Remuneration Paid. File with CRA along with T4 slips.",
        FOOTER_SIZE,
        Mm(MARGIN_LEFT),
        Mm(y_position),
        &font_regular,
    );
    y_position -= FOOTER_LEAD;

    current_layer.use_text(
        "Employer CPP = Employee CPP; Employer CPP2 = Employee CPP2; Employer EI = Employee EI × 1.4",
        FOOTER_NOTE_SIZE,
        Mm(MARGIN_LEFT),
        Mm(y_position),
        &font_regular,
    );

    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))?;

    Ok(())
}

/// Generate T4 Summary PDF with remittance data (Box 82)
pub fn generate_t4_summary_with_remittances<P: AsRef<Path>>(
    output_path: P,
    year: i32,
    company: &Company,
    t4_slips: &[T4Data],
    total_remittances: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    // Calculate summary with remittance data
    let summary = T4SummaryData::calculate(year, t4_slips, total_remittances);

    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new("T4 Summary", Mm(215.9), Mm(279.4), "Layer 1");

    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let mut y_position = START_Y;

    // Title
    current_layer.use_text("T4 Summary of Remuneration Paid", TITLE_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= TITLE_LEAD;

    current_layer.use_text(&format!("Year: {}", year), HEADING_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= HEADING_LEAD;

    // Company info
    current_layer.use_text(&format!("Company: {}", company.name), SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_regular);
    y_position -= SECTION_LEAD;

    if let Some(ref bn) = company.business_number {
        current_layer.use_text(&format!("Business Number: {}", bn), SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_regular);
        y_position -= SECTION_LEAD;
    }
    y_position -= SECTION_LEAD;

    // Summary of Remuneration Paid
    current_layer.use_text("SUMMARY OF REMUNERATION PAID", HEADING_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= HEADING_LEAD;

    // Box 88 - Number of T4 slips
    draw_t4_box(&current_layer, &font_regular, &font_bold, MARGIN_LEFT, y_position, "Box 88 - Number of T4 slips filed", &summary.total_slips.to_string());
    y_position -= BOX_LEAD;

    // Box 14 - Employment income
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 14 - Employment income",
        &format_currency(summary.total_employment_income),
    );
    y_position -= BOX_LEAD;

    // Box 20 - RPP contributions
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 20 - RPP contributions",
        &format_currency(summary.total_rpp_contributions),
    );
    y_position -= BOX_LEAD;

    // Box 52 - Pension adjustment
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 52 - Pension adjustment",
        &format_currency(summary.total_pension_adjustment),
    );
    y_position -= BOX_LEAD * 2.0;

    // Employee Contributions section
    current_layer.use_text("EMPLOYEE CONTRIBUTIONS", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 16 - Employee CPP
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 16 - Employee's CPP contributions",
        &format_currency(summary.total_employee_cpp),
    );
    y_position -= BOX_LEAD;

    // Box 16a - Employee CPP2
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 16a - Employee's CPP2 contributions",
        &format_currency(summary.total_employee_cpp2),
    );
    y_position -= BOX_LEAD;

    // Box 18 - Employee EI
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 18 - Employee's EI premiums",
        &format_currency(summary.total_employee_ei),
    );
    y_position -= BOX_LEAD * 2.0;

    // Employer Contributions section
    current_layer.use_text("EMPLOYER CONTRIBUTIONS", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 27 - Employer CPP
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 27 - Employer's CPP contributions",
        &format_currency(summary.total_employer_cpp),
    );
    y_position -= BOX_LEAD;

    // Box 27a - Employer CPP2
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 27a - Employer's CPP2 contributions",
        &format_currency(summary.total_employer_cpp2),
    );
    y_position -= BOX_LEAD;

    // Box 19 - Employer EI
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 19 - Employer's EI premiums",
        &format_currency(summary.total_employer_ei),
    );
    y_position -= BOX_LEAD * 2.0;

    // Income Tax Deducted section
    current_layer.use_text("INCOME TAX DEDUCTED", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 22 - Income tax deducted
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 22 - Income tax deducted",
        &format_currency(summary.total_income_tax),
    );
    y_position -= BOX_LEAD * 2.0;

    // Totals section
    current_layer.use_text("TOTALS", SECTION_SIZE, Mm(MARGIN_LEFT), Mm(y_position), &font_bold);
    y_position -= SECTION_LEAD;

    // Box 80 - Total deductions reported
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 80 - Total deductions reported",
        &format_currency(summary.total_deductions_reported),
    );
    y_position -= BOX_LEAD;

    // Box 82 - Total remittances paid
    draw_t4_box(
        &current_layer,
        &font_regular,
        &font_bold,
        MARGIN_LEFT,
        y_position,
        "Box 82 - Total remittances paid",
        &format_currency(summary.total_remittances_paid),
    );
    y_position -= BOX_LEAD * 2.0;

    // Difference
    draw_t4_box(&current_layer, &font_regular, &font_bold, MARGIN_LEFT, y_position, "Difference (Box 80 - Box 82)", &format_currency(summary.difference));
    y_position -= BOX_LEAD * 2.0;

    // Footer notes
    current_layer.use_text(
        "This is the T4 Summary of Remuneration Paid. File with CRA along with T4 slips.",
        FOOTER_SIZE,
        Mm(MARGIN_LEFT),
        Mm(y_position),
        &font_regular,
    );
    y_position -= FOOTER_LEAD;

    current_layer.use_text(
        "Employer CPP = Employee CPP; Employer CPP2 = Employee CPP2; Employer EI = Employee EI × 1.4",
        FOOTER_NOTE_SIZE,
        Mm(MARGIN_LEFT),
        Mm(y_position),
        &font_regular,
    );

    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))?;

    Ok(())
}

/// Generate T4 XML efile for CRA submission in T619 Internet File Transfer format
/// Format follows CRA T619 schema: Submission > T619 > Return > T4 > T4Slip
pub fn generate_t4_xml<P: AsRef<Path>>(
    output_path: P,
    year: i32,
    company: &Company,
    t4_slips: &[T4Data],
    transmitter: &T619Transmitter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;

    // Write XML declaration
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(file, "<Submission xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">")?;

    // ── T619 Header (Transmitter Information) ──────────────────────
    writeln!(file, "<T619>")?;
    writeln!(file, "<TransmitterAccountNumber>")?;
    writeln!(file, "<bn15>{}</bn15>", escape_xml(&transmitter.bn15))?;
    writeln!(file, "</TransmitterAccountNumber>")?;

    if let Some(ref ref_id) = transmitter.submission_ref_id {
        writeln!(file, "<sbmt_ref_id>{}</sbmt_ref_id>", escape_xml(ref_id))?;
    } else {
        writeln!(file, "<sbmt_ref_id></sbmt_ref_id>")?;
    }

    writeln!(file, "<summ_cnt>{}</summ_cnt>", t4_slips.len())?;
    writeln!(file, "<lang_cd>E</lang_cd>")?;

    writeln!(file, "<TransmitterName>")?;
    writeln!(file, "<l1_nm>{}</l1_nm>", escape_xml(&transmitter.name))?;
    writeln!(file, "</TransmitterName>")?;

    writeln!(file, "<TransmitterCountryCode>CAN</TransmitterCountryCode>")?;

    // Contact information
    writeln!(file, "<CNTC>")?;
    writeln!(file, "<cntc_nm>{}</cntc_nm>", escape_xml(&transmitter.contact_name))?;
    writeln!(file, "<cntc_area_cd>{}</cntc_area_cd>", escape_xml(&transmitter.phone_area))?;
    writeln!(file, "<cntc_phn_nbr>{}</cntc_phn_nbr>", escape_xml(&transmitter.phone))?;
    match &transmitter.email {
        Some(email) => writeln!(file, "<cntc_email_area>{}</cntc_email_area>", escape_xml(email))?,
        None => writeln!(file, "<cntc_email_area></cntc_email_area>")?,
    }
    writeln!(file, "</CNTC>")?;
    writeln!(file, "</T619>")?;

    // ── Return Section ──────────────────────────────────────────────
    writeln!(file, "<Return>")?;
    writeln!(file, "<T4>")?;

    // ── T4 Slips ───────────────────────────────────────────────────
    for t4 in t4_slips {
        writeln!(file, "<T4Slip>")?;

        // Employee name
        writeln!(file, "<EMPE_NM>")?;
        writeln!(file, "<snm>{}</snm>", escape_xml(&t4.employee.last_name))?;
        writeln!(file, "<gvn_nm>{}</gvn_nm>", escape_xml(&t4.employee.first_name))?;
        writeln!(file, "</EMPE_NM>")?;

        // Employee address
        writeln!(file, "<EMPE_ADDR>")?;
        writeln!(file, "<addr_l1_txt>{}</addr_l1_txt>", escape_xml(&t4.employee.address.street))?;
        writeln!(file, "<addr_l2_txt></addr_l2_txt>")?;
        writeln!(file, "<cty_nm>{}</cty_nm>", escape_xml(&t4.employee.address.city))?;
        writeln!(file, "<prov_cd>{}</prov_cd>", t4.employee.address.province)?;
        writeln!(file, "<cntry_cd>CAN</cntry_cd>")?;
        writeln!(file, "<pstl_cd>{}</pstl_cd>", escape_xml(&t4.employee.address.postal_code))?;
        writeln!(file, "</EMPE_ADDR>")?;

        // SIN
        writeln!(file, "<sin>{}</sin>", t4.employee.sin)?;

        // Employee number
        writeln!(file, "<empe_nbr>{}</empe_nbr>", escape_xml(&t4.employee.employee_number))?;

        // Business number (employer BN)
        writeln!(file, "<bn>{}</bn>", escape_xml(company.business_number.as_deref().unwrap_or("")))?;

        // Exemption codes (0 = not exempt, 1 = exempt)
        writeln!(
            file,
            "<cpp_qpp_xmpt_cd>{}</cpp_qpp_xmpt_cd>",
            if t4.cpp_contributions.is_zero() && t4.cpp_pensionable_earnings.is_zero() { "1" } else { "0" }
        )?;
        writeln!(file, "<ei_xmpt_cd>{}</ei_xmpt_cd>", if t4.ei_premiums.is_zero() && t4.ei_insurable_earnings.is_zero() { "1" } else { "0" })?;

        // Report type code: O = Original, A = Amendment
        writeln!(file, "<rpt_tcd>O</rpt_tcd>")?;

        // Province of employment
        writeln!(file, "<empt_prov_cd>{}</empt_prov_cd>", t4.province_of_employment)?;

        // Dental benefit code (1=No, 2=Basic, 3=Comprehensive)
        writeln!(file, "<empr_dntl_ben_rpt_cd>{}</empr_dntl_ben_rpt_cd>", t4.dental_benefit)?;

        // T4 Amounts (in dollars with 2 decimal places)
        writeln!(file, "<T4_AMT>")?;
        writeln!(file, "<empt_incamt>{}</empt_incamt>", decimal_to_dollars_string(&t4.employment_income))?;
        writeln!(file, "<cpp_cntrb_amt>{}</cpp_cntrb_amt>", decimal_to_dollars_string(&t4.cpp_contributions))?;
        writeln!(file, "<cppe_cntrb_amt>{}</cppe_cntrb_amt>", decimal_to_dollars_string(&t4.cpp2_contributions))?;
        writeln!(file, "<empe_eip_amt>{}</empe_eip_amt>", decimal_to_dollars_string(&t4.ei_premiums))?;
        writeln!(file, "<itx_ddct_amt>{}</itx_ddct_amt>", decimal_to_dollars_string(&t4.income_tax_deducted))?;
        writeln!(file, "<ei_insu_ern_amt>{}</ei_insu_ern_amt>", decimal_to_dollars_string(&t4.ei_insurable_earnings))?;
        writeln!(file, "<cpp_qpp_ern_amt>{}</cpp_qpp_ern_amt>", decimal_to_dollars_string(&t4.cpp_pensionable_earnings))?;
        writeln!(file, "</T4_AMT>")?;

        // Other information (RPP contributions, pension adjustment, etc.)
        writeln!(file, "<OTH_INFO>")?;
        if !t4.rpp_contributions.is_zero() {
            writeln!(file, "<rpp_amt>{}</rpp_amt>", decimal_to_dollars_string(&t4.rpp_contributions))?;
        }
        if !t4.pension_adjustment.is_zero() {
            writeln!(file, "<pen_adj_amt>{}</pen_adj_amt>", decimal_to_dollars_string(&t4.pension_adjustment))?;
        }
        writeln!(file, "</OTH_INFO>")?;

        writeln!(file, "</T4Slip>")?;
    }

    // ── T4 Summary ─────────────────────────────────────────────────
    writeln!(file, "<T4Summary>")?;
    writeln!(file, "<bn>{}</bn>", escape_xml(company.business_number.as_deref().unwrap_or("")))?;
    writeln!(file, "<EMPR_NM>")?;
    writeln!(file, "<l1_nm>{}</l1_nm>", escape_xml(&company.name))?;
    writeln!(file, "</EMPR_NM>")?;
    writeln!(file, "<EMPR_ADDR>")?;
    writeln!(file, "<addr_l1_txt>{}</addr_l1_txt>", escape_xml(&company.address))?;
    writeln!(file, "<addr_l2_txt></addr_l2_txt>")?;
    writeln!(file, "<cty_nm></cty_nm>")?;
    writeln!(file, "<prov_cd>{}</prov_cd>", company.province.code())?;
    writeln!(file, "<cntry_cd>CAN</cntry_cd>")?;
    writeln!(file, "<pstl_cd></pstl_cd>")?;
    writeln!(file, "</EMPR_ADDR>")?;

    // Contact for summary
    writeln!(file, "<CNTC>")?;
    writeln!(file, "<cntc_nm>{}</cntc_nm>", escape_xml(&transmitter.contact_name))?;
    writeln!(file, "<cntc_area_cd>{}</cntc_area_cd>", escape_xml(&transmitter.phone_area))?;
    writeln!(file, "<cntc_phn_nbr>{}</cntc_phn_nbr>", escape_xml(&transmitter.phone))?;
    writeln!(file, "</CNTC>")?;

    writeln!(file, "<tx_yr>{}</tx_yr>", year)?;
    writeln!(file, "<slp_cnt>{}</slp_cnt>", t4_slips.len())?;
    writeln!(file, "<rpt_tcd>O</rpt_tcd>")?;

    // Summary totals
    let tot_empt_incamt: Decimal = t4_slips.iter().map(|t| t.employment_income).sum();
    let tot_empe_cpp: Decimal = t4_slips.iter().map(|t| t.cpp_contributions).sum();
    let tot_empe_cppe: Decimal = t4_slips.iter().map(|t| t.cpp2_contributions).sum();
    let tot_empe_eip: Decimal = t4_slips.iter().map(|t| t.ei_premiums).sum();
    let tot_itx_ddct: Decimal = t4_slips.iter().map(|t| t.income_tax_deducted).sum();
    let tot_empr_cpp = tot_empe_cpp; // Employer CPP = Employee CPP
    let tot_empr_cppe = tot_empe_cppe; // Employer CPP2 = Employee CPP2
    let tot_empr_eip = tot_empe_eip * rust_decimal_macros::dec!(1.4); // Employer EI = Employee EI × 1.4

    writeln!(file, "<T4_TAMT>")?;
    writeln!(file, "<tot_empt_incamt>{}</tot_empt_incamt>", decimal_to_dollars_string(&tot_empt_incamt))?;
    writeln!(file, "<tot_empe_cpp_amt>{}</tot_empe_cpp_amt>", decimal_to_dollars_string(&tot_empe_cpp))?;
    writeln!(file, "<tot_empe_cppe_amt>{}</tot_empe_cppe_amt>", decimal_to_dollars_string(&tot_empe_cppe))?;
    writeln!(file, "<tot_empe_eip_amt>{}</tot_empe_eip_amt>", decimal_to_dollars_string(&tot_empe_eip))?;
    writeln!(file, "<tot_itx_ddct_amt>{}</tot_itx_ddct_amt>", decimal_to_dollars_string(&tot_itx_ddct))?;
    writeln!(file, "<tot_empr_cpp_amt>{}</tot_empr_cpp_amt>", decimal_to_dollars_string(&tot_empr_cpp))?;
    writeln!(file, "<tot_empr_cppe_amt>{}</tot_empr_cppe_amt>", decimal_to_dollars_string(&tot_empr_cppe))?;
    writeln!(file, "<tot_empr_eip_amt>{}</tot_empr_eip_amt>", decimal_to_dollars_string(&tot_empr_eip))?;
    writeln!(file, "</T4_TAMT>")?;

    writeln!(file, "</T4Summary>")?;
    writeln!(file, "</T4>")?;
    writeln!(file, "</Return>")?;
    writeln!(file, "</Submission>")?;

    Ok(())
}

/// Generate T4 CSV efile for CRA submission
/// Format follows CRA T4 CSV specification
pub fn generate_t4_csv<P: AsRef<Path>>(output_path: P, year: i32, company: &Company, t4_slips: &[T4Data]) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = Writer::from_path(output_path)?;

    // Write header row
    wtr.write_record(&[
        "TaxYear",
        "BusinessNumber",
        "EmployerName",
        "SIN",
        "EmployeeNumber",
        "LastName",
        "FirstName",
        "ProvinceOfEmployment",
        "Box14_EmploymentIncome",
        "Box16_CPPContributions",
        "Box16a_CPP2Contributions",
        "Box18_EIPremiums",
        "Box20_RPPContributions",
        "Box22_IncomeTaxDeducted",
        "Box24_EIInsurableEarnings",
        "Box26_CPPPensionableEarnings",
        "Box52_PensionAdjustment",
        "Box45_DentalBenefit",
        "EmploymentCode",
    ])?;

    // Write data rows
    for t4 in t4_slips {
        wtr.write_record(&[
            year.to_string(),
            company.business_number.clone().unwrap_or_default(),
            company.name.clone(),
            t4.employee.sin.clone(),
            t4.employee.employee_number.clone(),
            t4.employee.last_name.clone(),
            t4.employee.first_name.clone(),
            t4.province_of_employment.clone(),
            decimal_to_cents_string(&t4.employment_income),
            decimal_to_cents_string(&t4.cpp_contributions),
            decimal_to_cents_string(&t4.cpp2_contributions),
            decimal_to_cents_string(&t4.rpp_contributions),
            decimal_to_cents_string(&t4.ei_premiums),
            decimal_to_cents_string(&t4.income_tax_deducted),
            decimal_to_cents_string(&t4.ei_insurable_earnings),
            decimal_to_cents_string(&t4.cpp_pensionable_earnings),
            decimal_to_cents_string(&t4.pension_adjustment),
            t4.dental_benefit.to_string(),
            t4.employment_code.clone().unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

/// Generate T4 Summary CSV
pub fn generate_t4_summary_csv<P: AsRef<Path>>(output_path: P, year: i32, company: &Company, t4_slips: &[T4Data]) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = Writer::from_path(output_path)?;

    // Write header
    wtr.write_record(&[
        "TaxYear",
        "BusinessNumber",
        "EmployerName",
        "NumberOfSlips",
        "TotalEmploymentIncome",
        "TotalCPPContributions",
        "TotalEIPremiums",
        "TotalIncomeTaxDeducted",
        "TotalEIInsurableEarnings",
        "TotalCPPPensionableEarnings",
    ])?;

    let total_14: Decimal = t4_slips.iter().map(|t| t.employment_income).sum();
    let total_16: Decimal = t4_slips.iter().map(|t| t.cpp_contributions).sum();
    let total_18: Decimal = t4_slips.iter().map(|t| t.ei_premiums).sum();
    let total_22: Decimal = t4_slips.iter().map(|t| t.income_tax_deducted).sum();
    let total_24: Decimal = t4_slips.iter().map(|t| t.ei_insurable_earnings).sum();
    let total_26: Decimal = t4_slips.iter().map(|t| t.cpp_pensionable_earnings).sum();

    wtr.write_record(&[
        year.to_string(),
        company.business_number.clone().unwrap_or_default(),
        company.name.clone(),
        t4_slips.len().to_string(),
        decimal_to_cents_string(&total_14),
        decimal_to_cents_string(&total_16),
        decimal_to_cents_string(&total_18),
        decimal_to_cents_string(&total_22),
        decimal_to_cents_string(&total_24),
        decimal_to_cents_string(&total_26),
    ])?;

    wtr.flush()?;
    Ok(())
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

/// Convert Decimal to cents string (no decimals, just the integer cents value)
fn decimal_to_cents_string(value: &Decimal) -> String {
    // Multiply by 100 to convert dollars to cents
    let cents = *value * Decimal::from(100);
    format!("{}", cents.round())
}

/// Convert Decimal to dollars string with 2 decimal places (e.g., "48000.00")
/// Used for CRA T619 XML format where amounts are in dollars, not cents
fn decimal_to_dollars_string(value: &Decimal) -> String {
    format!("{:.2}", value)
}
