use printpdf::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpr_core::models::{Employee, Company};

/// T4A (Statement of Pension, Retirement, Annuity, and Other Income) data
#[derive(Debug, Clone)]
pub struct T4AData {
    pub employee: Employee,
    pub year: i32,
    pub pension_or_superannuation: Option<Decimal>,  // Box 016
    pub lump_sum_payments: Option<Decimal>,           // Box 018
    pub self_employed_commissions: Option<Decimal>,   // Box 020
    pub income_tax_deducted: Option<Decimal>,         // Box 022
    pub annuities: Option<Decimal>,                   // Box 024
    pub fees_for_services: Option<Decimal>,           // Box 048
    pub other_income: Option<Decimal>,                // Box 028
}

/// Generate a T4A slip PDF
pub fn generate_t4a<P: AsRef<Path>>(
    output_path: P,
    t4a_data: &T4AData,
    company: &Company,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "T4A Statement",
        Mm(215.9), // Letter size width
        Mm(279.4), // Letter size height
        "Layer 1",
    );
    
    let current_layer = doc.get_page(page1).get_layer(layer1);
    
    // Load fonts
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    
    let mut y_position = 260.0;
    
    // Title
    current_layer.use_text(
        &format!("Statement of Pension, Retirement, Annuity, and Other Income - {}",
                 t4a_data.year),
        14.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 6.0;
    
    current_layer.use_text(
        "T4A - Protected B when completed",
        10.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    y_position -= 15.0;
    
    // Payer information (Employer/Company)
    current_layer.use_text("PAYER INFORMATION", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;
    
    draw_field(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Name", &company.name);
    y_position -= 7.0;
    
    draw_field(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Address", &company.address);
    y_position -= 7.0;
    
    if let Some(ref bn) = company.business_number {
        draw_field(&current_layer, &font_regular, &font_bold, 20.0, y_position, "Business Number", bn);
        y_position -= 15.0;
    } else {
        y_position -= 8.0;
    }
    
    // Recipient information (Employee)
    current_layer.use_text("RECIPIENT INFORMATION", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;
    
    draw_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Name",
        &format!("{} {}", t4a_data.employee.first_name, t4a_data.employee.last_name),
    );
    y_position -= 7.0;
    
    draw_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Social Insurance Number",
        &format_sin(&t4a_data.employee.sin),
    );
    y_position -= 7.0;
    
    draw_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Address",
        &format!("{}, {}, {} {}", 
            t4a_data.employee.address.street,
            t4a_data.employee.address.city,
            t4a_data.employee.address.province,
            t4a_data.employee.address.postal_code),
    );
    y_position -= 15.0;
    
    // Income boxes
    current_layer.use_text("INCOME DETAILS", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;
    
    if let Some(amount) = t4a_data.pension_or_superannuation {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 016 - Pension or superannuation",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t4a_data.lump_sum_payments {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 018 - Lump-sum payments",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t4a_data.self_employed_commissions {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 020 - Self-employed commissions",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t4a_data.income_tax_deducted {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 022 - Income tax deducted",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t4a_data.annuities {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 024 - Annuities",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t4a_data.fees_for_services {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 048 - Fees for services",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t4a_data.other_income {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 028 - Other income",
            &format_currency(amount),
        );
        y_position -= 15.0;
    }
    
    // Footer
    current_layer.use_text(
        "File this slip with your income tax return",
        8.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    
    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))?;
    
    Ok(())
}

fn draw_field(
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

fn format_sin(sin: &str) -> String {
    let digits: String = sin.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 9 {
        format!("{} {} {}", &digits[0..3], &digits[3..6], &digits[6..9])
    } else {
        sin.to_string()
    }
}

fn format_currency(value: Decimal) -> String {
    format!("${:.2}", value)
}
