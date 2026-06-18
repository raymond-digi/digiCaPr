use chrono::NaiveDate;
use printpdf::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpr_core::models::{Employee, Company};

/// ROE (Record of Employment) data
#[derive(Debug, Clone)]
pub struct RoeData {
    pub employee: Employee,
    pub serial_number: String,
    pub first_day_worked: NaiveDate,
    pub last_day_paid: NaiveDate,
    pub final_pay_period_end: NaiveDate,
    pub reason_for_separation: SeparationReason,
    pub insurable_hours: Decimal,
    pub insurable_earnings: Decimal,
    pub pay_period_type: PayPeriodType,
    pub vacation_pay: Option<Decimal>,
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparationReason {
    Shortage,           // Code A - Shortage of work
    Strike,             // Code B - Strike or lockout
    ReturnToSchool,     // Code C - Return to school
    Illness,            // Code D - Illness or injury
    Quit,               // Code E - Quit
    Retirement,         // Code F - Retirement
    Maternity,          // Code G - Maternity
    Other,              // Code K - Other
    Dismissal,          // Code M - Dismissal
}

impl SeparationReason {
    pub fn code(&self) -> &'static str {
        match self {
            SeparationReason::Shortage => "A",
            SeparationReason::Strike => "B",
            SeparationReason::ReturnToSchool => "C",
            SeparationReason::Illness => "D",
            SeparationReason::Quit => "E",
            SeparationReason::Retirement => "F",
            SeparationReason::Maternity => "G",
            SeparationReason::Other => "K",
            SeparationReason::Dismissal => "M",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            SeparationReason::Shortage => "Shortage of work",
            SeparationReason::Strike => "Strike or lockout",
            SeparationReason::ReturnToSchool => "Return to school",
            SeparationReason::Illness => "Illness or injury",
            SeparationReason::Quit => "Quit",
            SeparationReason::Retirement => "Retirement",
            SeparationReason::Maternity => "Maternity",
            SeparationReason::Other => "Other",
            SeparationReason::Dismissal => "Dismissal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayPeriodType {
    Weekly,
    BiWeekly,
    SemiMonthly,
    Monthly,
    Thirteen,
    TwentyTwo,
    Hourly,
    Daily,
}

impl PayPeriodType {
    pub fn code(&self) -> i32 {
        match self {
            PayPeriodType::Weekly => 1,
            PayPeriodType::BiWeekly => 2,
            PayPeriodType::SemiMonthly => 3,
            PayPeriodType::Monthly => 4,
            PayPeriodType::Thirteen => 5,
            PayPeriodType::TwentyTwo => 6,
            PayPeriodType::Hourly => 7,
            PayPeriodType::Daily => 8,
        }
    }
}

/// Generate an ROE PDF
pub fn generate_roe<P: AsRef<Path>>(
    output_path: P,
    roe_data: &RoeData,
    company: &Company,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "Record of Employment",
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
    current_layer.use_text(
        "RECORD OF EMPLOYMENT - ROE",
        16.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 8.0;
    
    current_layer.use_text(
        "Protected B when completed",
        9.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    y_position -= 15.0;
    
    // Serial number
    current_layer.use_text(
        &format!("Serial Number: {}", roe_data.serial_number),
        11.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 20.0;
    
    // Block 1 - Employer information
    current_layer.use_text(
        "EMPLOYER INFORMATION",
        11.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 8.0;
    
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Name",
        &company.name,
    );
    y_position -= 8.0;
    
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Address",
        &company.address,
    );
    y_position -= 8.0;
    
    if let Some(ref bn) = company.business_number {
        draw_roe_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Business Number",
            bn,
        );
        y_position -= 15.0;
    } else {
        y_position -= 7.0;
    }
    
    // Block 2 - Employee information
    current_layer.use_text(
        "EMPLOYEE INFORMATION",
        11.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 8.0;
    
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Name",
        &format!("{}, {}", roe_data.employee.last_name, roe_data.employee.first_name),
    );
    y_position -= 8.0;
    
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Social Insurance Number",
        &format_sin(&roe_data.employee.sin),
    );
    y_position -= 8.0;
    
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Address",
        &format!("{}, {}, {} {}", 
            roe_data.employee.address.street,
            roe_data.employee.address.city,
            roe_data.employee.address.province,
            roe_data.employee.address.postal_code),
    );
    y_position -= 15.0;
    
    // Block 10 - First day worked
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 10 - First day worked",
        &roe_data.first_day_worked.format("%Y-%m-%d").to_string(),
    );
    y_position -= 8.0;
    
    // Block 11 - Last day paid
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 11 - Last day for which paid",
        &roe_data.last_day_paid.format("%Y-%m-%d").to_string(),
    );
    y_position -= 8.0;
    
    // Block 12 - Final pay period ending date
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 12 - Final pay period ending date",
        &roe_data.final_pay_period_end.format("%Y-%m-%d").to_string(),
    );
    y_position -= 15.0;
    
    // Block 15A - Insurable hours
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 15A - Insurable hours",
        &format!("{:.0}", roe_data.insurable_hours),
    );
    y_position -= 8.0;
    
    // Block 15B - Insurable earnings
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 15B - Insurable earnings",
        &format_currency(roe_data.insurable_earnings),
    );
    y_position -= 15.0;
    
    // Block 16 - Reason for separation
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 16 - Reason for issuing this ROE",
        &format!("{} - {}", 
            roe_data.reason_for_separation.code(),
            roe_data.reason_for_separation.description()),
    );
    y_position -= 15.0;
    
    // Block 17 - Vacation pay
    if let Some(vac_pay) = roe_data.vacation_pay {
        draw_roe_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Block 17 - Vacation pay",
            &format_currency(vac_pay),
        );
        y_position -= 15.0;
    }
    
    // Block 18 - Pay period type
    draw_roe_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Block 18 - Pay period type",
        &format!("Code {}", roe_data.pay_period_type.code()),
    );
    y_position -= 15.0;
    
    // Comments
    if let Some(ref comments) = roe_data.comments {
        current_layer.use_text(
            "Comments:",
            10.0,
            Mm(20.0),
            Mm(y_position),
            &font_bold,
        );
        y_position -= 8.0;
        
        current_layer.use_text(
            comments,
            9.0,
            Mm(20.0),
            Mm(y_position),
            &font_regular,
        );
        y_position -= 15.0;
    }
    
    // Footer
    current_layer.use_text(
        "This ROE should be issued within 5 calendar days from the first day of the interruption of earnings",
        8.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    y_position -= 6.0;
    
    current_layer.use_text(
        "or from the day the employer becomes aware of the interruption of earnings.",
        8.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    
    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))?;
    
    Ok(())
}

/// Draw an ROE field with label and value
fn draw_roe_field(
    layer: &PdfLayerReference,
    font_regular: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    x: f64,
    y: f64,
    label: &str,
    value: &str,
) {
    layer.use_text(label, 9.0, Mm(x), Mm(y), font_regular);
    layer.use_text(value, 10.0, Mm(x + 80.0), Mm(y), font_bold);
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

/// Format currency for display
fn format_currency(value: Decimal) -> String {
    format!("${:.2}", value)
}
