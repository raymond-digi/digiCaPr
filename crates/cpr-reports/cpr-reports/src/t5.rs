use printpdf::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpr_core::models::{Employee, Company};

/// T5 (Statement of Investment Income) data
#[derive(Debug, Clone)]
pub struct T5Data {
    pub employee: Employee,
    pub year: i32,
    pub interest_from_canadian_sources: Option<Decimal>,  // Box 13
    pub dividends_other_than_eligible: Option<Decimal>,   // Box 10
    pub eligible_dividends: Option<Decimal>,              // Box 24
    pub actual_amount_of_eligible_dividends: Option<Decimal>, // Box 25
    pub taxable_amount_of_dividends: Option<Decimal>,     // Box 11
    pub foreign_income: Option<Decimal>,                  // Box 15
    pub foreign_tax_paid: Option<Decimal>,                // Box 16
    pub other_income: Option<Decimal>,                    // Box 14
}

/// Generate a T5 slip PDF
pub fn generate_t5<P: AsRef<Path>>(
    output_path: P,
    t5_data: &T5Data,
    company: &Company,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "T5 Statement",
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
        &format!("Statement of Investment Income - {}", t5_data.year),
        14.0,
        Mm(20.0),
        Mm(y_position),
        &font_bold,
    );
    y_position -= 6.0;
    
    current_layer.use_text(
        "T5 - Protected B when completed",
        10.0,
        Mm(20.0),
        Mm(y_position),
        &font_regular,
    );
    y_position -= 15.0;
    
    // Payer information
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
    
    // Recipient information
    current_layer.use_text("RECIPIENT INFORMATION", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;
    
    draw_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Name",
        &format!("{} {}", t5_data.employee.first_name, t5_data.employee.last_name),
    );
    y_position -= 7.0;
    
    draw_field(
        &current_layer,
        &font_regular,
        &font_bold,
        20.0,
        y_position,
        "Social Insurance Number",
        &format_sin(&t5_data.employee.sin),
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
            t5_data.employee.address.street,
            t5_data.employee.address.city,
            t5_data.employee.address.province,
            t5_data.employee.address.postal_code),
    );
    y_position -= 15.0;
    
    // Income boxes
    current_layer.use_text("INVESTMENT INCOME", 11.0, Mm(20.0), Mm(y_position), &font_bold);
    y_position -= 8.0;
    
    if let Some(amount) = t5_data.interest_from_canadian_sources {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 13 - Interest from Canadian sources",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.dividends_other_than_eligible {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 10 - Actual amount of dividends (other than eligible)",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.eligible_dividends {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 24 - Eligible dividends",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.actual_amount_of_eligible_dividends {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 25 - Actual amount of eligible dividends",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.taxable_amount_of_dividends {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 11 - Taxable amount of dividends",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.foreign_income {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 15 - Foreign income",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.foreign_tax_paid {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 16 - Foreign tax paid",
            &format_currency(amount),
        );
        y_position -= 7.0;
    }
    
    if let Some(amount) = t5_data.other_income {
        draw_field(
            &current_layer,
            &font_regular,
            &font_bold,
            20.0,
            y_position,
            "Box 14 - Other income",
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
