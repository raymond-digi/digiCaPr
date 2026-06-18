use printpdf::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpr_core::models::payroll::{Payroll, YtdTotals};
use cpr_core::models::{Company, Employee, PayType};
use crate::payroll_report::format_hours_hhmm;

// Page dimensions (Letter portrait)
const PAGE_WIDTH: f64 = 215.9;
const PAGE_HEIGHT: f64 = 279.4;
const MARGIN_LEFT: f64 = 15.0;
const MARGIN_RIGHT: f64 = 15.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM: f64 = 10.0;
const CONTENT_WIDTH: f64 = PAGE_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
const CONTENT_HEIGHT: f64 = PAGE_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;

/// Context for paystub generation
struct PaystubContext {
    current_layer: PdfLayerReference,
    font_bold: IndirectFontRef,
    font_regular: IndirectFontRef,
}

impl PaystubContext {
    fn new(doc: &PdfDocumentReference, page: PdfPageIndex, layer: PdfLayerIndex) -> Result<Self, Box<dyn std::error::Error>> {
        let current_layer = doc.get_page(page).get_layer(layer);
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        Ok(PaystubContext { current_layer, font_bold, font_regular })
    }

    // ******************************************************************
    /// Calculate text width in mm
    fn text_width_mm(&self, text: &str, size_pt: f64) -> f64 {
        let unit_narrow_1 = 250.0;
        let unit_space = 278.0;
        let unit_narrow_2 = 333.0;
        let unit_upper_narrow = 278.0;
        let unit_upper_medium = 556.0;
        let unit_upper_wide = 722.0;
        let unit_digit = 556.0;
        let unit_avg = 500.0;
        let scale = size_pt / 1000.0;
        let width_pt: f64 = text
            .chars()
            .map(|c| match c {
                'i' | 'l' | '!' | ':' | ';' | '|' | '\'' => unit_narrow_1,
                ' ' => unit_space,
                '-' | '.' | ',' | '*' | '/' | '(' | ')' | '[' | ']' | '{' | '}' | 'f' | 'r' | 't' => unit_narrow_2,
                'I' | 'J' | 'L' | 'T' => unit_upper_narrow,
                'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'U' | 'V' | 'X' | 'Y' | 'Z' => unit_upper_medium,
                'A' | 'M' | 'W' => unit_upper_wide,
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => unit_digit,
                _ => unit_avg,
            })
            .sum::<f64>()
            * scale;
        width_pt * (25.4 / 72.0)
    }

    // ******************************************************************
    fn print_text(&self, text: &str, size_pt: f64, x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
        self.current_layer.use_text(text, size_pt, Mm(MARGIN_LEFT + x_mm), Mm(MARGIN_BOTTOM + y_mm), font);
    }

    // ******************************************************************
    fn print_left(&self, text: &str, size_pt: f64, x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
        self.print_text(text, size_pt, x_mm, y_mm, font);
    }

    // ******************************************************************
    fn print_right(&self, text: &str, size_pt: f64, right_x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
        let w = self.text_width_mm(text, size_pt);
        self.print_text(text, size_pt, right_x_mm - w, y_mm, font);
    }

    // ******************************************************************
    fn print_center(&self, text: &str, size_pt: f64, center_x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
        let w = self.text_width_mm(text, size_pt);
        self.print_text(text, size_pt, center_x_mm - w * 0.5, y_mm, font);
    }

    // ******************************************************************
    fn draw_line(&self, x1_mm: f64, y1_mm: f64, x2_mm: f64, y2_mm: f64, thickness: f64) {
        use printpdf::{Line, Point};
        self.current_layer.set_outline_thickness(thickness);
        let line = Line {
            points: vec![
                (Point::new(Mm(MARGIN_LEFT + x1_mm), Mm(MARGIN_BOTTOM + y1_mm)), false),
                (Point::new(Mm(MARGIN_LEFT + x2_mm), Mm(MARGIN_BOTTOM + y2_mm)), false),
            ],
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        self.current_layer.add_shape(line);
    }

    // ******************************************************************
    fn draw_rect(&self, x_left_mm: f64, y_top_mm: f64, w_mm: f64, h_mm: f64, thickness: f64) {
        use printpdf::{Line, Point};
        self.current_layer.set_outline_thickness(thickness);
        let x = MARGIN_LEFT + x_left_mm;
        let y = MARGIN_BOTTOM + y_top_mm;
        let line = Line {
            points: vec![
                (Point::new(Mm(x), Mm(y)), false),
                (Point::new(Mm(x + w_mm), Mm(y)), false),
                (Point::new(Mm(x + w_mm), Mm(y - h_mm)), false),
                (Point::new(Mm(x), Mm(y - h_mm)), false),
            ],
            is_closed: true,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        self.current_layer.add_shape(line);
    }
}

/// Generate a pay stub PDF for the given payroll
pub fn generate_paystub<P: AsRef<Path>>(
    output_path: P,
    employee: &Employee,
    payroll: &Payroll,
    ytd: &YtdTotals,
    company: &Company,
) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Pay Stub", Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Layer 1");

    let ctx = PaystubContext::new(&doc, page1, layer1)?;

    let mut y = CONTENT_HEIGHT;
    let title_size = 10.0;
    let subtitle_size = 8.0;
    let pt_size = 9.0;
    let line_height = pt_size * 0.50; // 0.5625  (9x0.5555=>5)

    // === HEADER SECTION ===
    // "EARNINGS STATEMENT" centered
    // let header_y = PAGE_HEIGHT - MARGIN_TOP - 5.0;
    ctx.print_center("EARNINGS STATEMENT", 14.0, CONTENT_WIDTH * 0.5, y, &ctx.font_bold);
    y -= line_height * 1.3;

    // Company name (left)
    ctx.print_left(&company.name, title_size, 0.0, y, &ctx.font_bold);
    y -= line_height;

    // Company address (left, multi-line)
    let addr_lines = parse_address(&company.address);
    for line in &addr_lines {
        ctx.print_left(line, pt_size, 0.0, y, &ctx.font_regular);
        y -= line_height;
    }

    // Horizontal line after header
    y += line_height * 0.3;
    ctx.draw_line(0.0, y, CONTENT_WIDTH, y, 0.5);
    y -= line_height;

    // Business number if present
    // if let Some(ref bn) = company.business_number {
    //     ctx.print_left(&format!("BN: {}", bn), pt_size, 0.0, y, &ctx.font_regular);
    //     y -= line_height;
    // }

    // Right-aligned info
    let right_x = CONTENT_WIDTH;
    let mut right_y = CONTENT_HEIGHT - line_height * 2.2;
    ctx.print_right(&format!("Period Ending: {}", payroll.pay_period_end.format("%Y-%m-%d")), pt_size, right_x, right_y, &ctx.font_regular);
    right_y -= line_height;
    ctx.print_right(&format!("Pay Date: {}", payroll.pay_date.format("%Y-%m-%d")), pt_size, right_x, right_y, &ctx.font_regular);

    // === EMPLOYEE INFO ===
    y -= line_height;
    ctx.print_left(&format!("{} {}", employee.first_name, employee.last_name), title_size, CONTENT_WIDTH * 0.15, y, &ctx.font_regular);
    ctx.print_left(&format!("S.I.N.: {}", mask_sin(&employee.sin)), pt_size, CONTENT_WIDTH * 0.60, y, &ctx.font_regular);
    y -= line_height;

    ctx.print_left(
        &format!("{}, {}, {} {}", employee.address.street, employee.address.city, employee.address.province, employee.address.postal_code),
        pt_size,
        CONTENT_WIDTH * 0.15,
        y,
        &ctx.font_regular,
    );
    y -= line_height;

    // Horizontal line after employee

    // === EARNINGS TABLE ===
    // Column positions for earnings (5 columns)
    let earn_desc_x = CONTENT_WIDTH * 0.1;
    let earn_rate_x = CONTENT_WIDTH * 0.3;
    let earn_hours_x = CONTENT_WIDTH * 0.4;
    let earn_current_x = CONTENT_WIDTH * 0.6;
    let earn_ytd_x = CONTENT_WIDTH * 0.9;

    y -= line_height;
    ctx.draw_line(earn_desc_x, y, earn_ytd_x, y, 0.5);
    y -= line_height * 1.5;
    ctx.print_center("EARNINGS", title_size, CONTENT_WIDTH * 0.5, y, &ctx.font_bold);
    y -= line_height;
    ctx.draw_line(earn_desc_x, y, earn_ytd_x, y, 0.3);
    y -= line_height * 1.0;

    // Column headers
    ctx.print_left("Income", subtitle_size, earn_desc_x, y, &ctx.font_bold);
    ctx.print_right("Rate", subtitle_size, earn_rate_x, y, &ctx.font_bold);
    ctx.print_right("Hours", subtitle_size, earn_hours_x, y, &ctx.font_bold);
    ctx.print_right("Current Total", subtitle_size, earn_current_x, y, &ctx.font_bold);
    ctx.print_right("Year to Date", subtitle_size, earn_ytd_x, y, &ctx.font_bold);
    y -= line_height * 0.5;
    ctx.draw_line(earn_desc_x, y, earn_ytd_x, y, 0.2);
    y -= line_height * 1.5;

    // Earnings rows based on pay type
    match employee.pay_type {
        PayType::Hourly => {
            // Regular pay
            let reg_hours = payroll.regular_hours.unwrap_or(Decimal::ZERO);
            let reg_pay = reg_hours * employee.pay_rate;
            ctx.print_left("Regular", pt_size, earn_desc_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(employee.pay_rate)), pt_size, earn_rate_x, y, &ctx.font_regular);
            ctx.print_right(&format_hours_hhmm(reg_hours), pt_size, earn_hours_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(reg_pay)), pt_size, earn_current_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(ytd.gross_pay)), pt_size, earn_ytd_x, y, &ctx.font_regular);
            y -= line_height;

            // Overtime pay
            let ot_hours = payroll.overtime_hours.unwrap_or(Decimal::ZERO);
            let ot_rate = employee.pay_rate * employee.overtime_multiplier;
            let ot_pay = ot_hours * ot_rate;
            ctx.print_left("Overtime", pt_size, earn_desc_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(ot_rate)), pt_size, earn_rate_x, y, &ctx.font_regular);
            ctx.print_right(&format_hours_hhmm(ot_hours), pt_size, earn_hours_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(ot_pay)), pt_size, earn_current_x, y, &ctx.font_regular);
            ctx.print_right("$0.00", pt_size, earn_ytd_x, y, &ctx.font_regular);
            y -= line_height;
        }
        _ => {
            // Salary or other non-hourly
            ctx.print_left("Salary", pt_size, earn_desc_x, y, &ctx.font_regular);
            ctx.print_right("-", pt_size, earn_rate_x, y, &ctx.font_regular);
            ctx.print_right("-", pt_size, earn_hours_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(payroll.gross_pay)), pt_size, earn_current_x, y, &ctx.font_regular);
            ctx.print_right(&format!("${}", format_decimal(ytd.gross_pay)), pt_size, earn_ytd_x, y, &ctx.font_regular);
            y -= line_height;
        }
    }

    // Additional earnings
    for earning in &payroll.additional_earnings {
        ctx.print_left(&earning.earning_type, pt_size, earn_desc_x, y, &ctx.font_regular);
        ctx.print_right("-", pt_size, earn_rate_x, y, &ctx.font_regular);
        if let Some(h) = earning.hours {
            ctx.print_right(&format_hours_hhmm(h), pt_size, earn_hours_x, y, &ctx.font_regular);
        } else {
            ctx.print_right("-", pt_size, earn_hours_x, y, &ctx.font_regular);
        }
        ctx.print_right(&format!("${}", format_decimal(earning.amount)), pt_size, earn_current_x, y, &ctx.font_regular);
        ctx.print_right("$0.00", pt_size, earn_ytd_x, y, &ctx.font_regular);
        y -= line_height;
    }

    // Gross Pay total
    ctx.draw_line(earn_desc_x, y, earn_ytd_x, y, 0.2);
    y -= line_height * 1.5;

    ctx.print_left("Gross Pay", pt_size, earn_desc_x, y, &ctx.font_bold);
    ctx.print_right(&format!("${}", format_decimal(payroll.gross_pay)), pt_size, earn_current_x, y, &ctx.font_bold);
    ctx.print_right(&format!("${}", format_decimal(ytd.gross_pay)), pt_size, earn_ytd_x, y, &ctx.font_regular);
    y -= line_height;
    ctx.draw_line(earn_desc_x, y, earn_ytd_x, y, 0.5);
    y -= line_height * 1.5;

    // === DEDUCTIONS TABLE ===============
    let ded_desc_x = CONTENT_WIDTH * 0.10;
    let ded_current_x = CONTENT_WIDTH * 0.60;
    let ded_ytd_x = CONTENT_WIDTH * 0.90;

    y -= line_height;
    ctx.draw_line(ded_desc_x, y, ded_ytd_x, y, 0.5);
    y -= line_height * 1.5;
    ctx.print_center("DEDUCTIONS", title_size, CONTENT_WIDTH * 0.5, y, &ctx.font_bold);
    y -= line_height;
    ctx.draw_line(ded_desc_x, y, ded_ytd_x, y, 0.3);
    y -= line_height;

    // Column headers
    ctx.print_left("Description", subtitle_size, ded_desc_x, y, &ctx.font_bold);
    ctx.print_right("Current Total", subtitle_size, ded_current_x, y, &ctx.font_bold);
    ctx.print_right("Year to Date", subtitle_size, ded_ytd_x, y, &ctx.font_bold);
    y -= line_height * 0.5;

    ctx.draw_line(ded_desc_x, y, ded_ytd_x, y, 0.2);
    y -= line_height * 1.5;

    // CPP
    ctx.print_left("CPP", pt_size, ded_desc_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(payroll.deductions.cpp)), pt_size, ded_current_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(ytd.cpp)), pt_size, ded_ytd_x, y, &ctx.font_regular);
    y -= line_height;

    // CPP2
    ctx.print_left("CPP2", pt_size, ded_desc_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(payroll.deductions.cpp2)), pt_size, ded_current_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(ytd.cpp2)), pt_size, ded_ytd_x, y, &ctx.font_regular);
    y -= line_height;

    // EI
    ctx.print_left("EI", pt_size, ded_desc_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(payroll.deductions.ei)), pt_size, ded_current_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(ytd.ei)), pt_size, ded_ytd_x, y, &ctx.font_regular);
    y -= line_height;

    // Federal Tax
    ctx.print_left("Federal Tax", pt_size, ded_desc_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(payroll.deductions.federal_tax)), pt_size, ded_current_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(ytd.federal_tax)), pt_size, ded_ytd_x, y, &ctx.font_regular);
    y -= line_height;

    // Provincial Tax
    ctx.print_left("Provincial Tax", pt_size, ded_desc_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(payroll.deductions.provincial_tax)), pt_size, ded_current_x, y, &ctx.font_regular);
    ctx.print_right(&format!("${}", format_decimal(ytd.provincial_tax)), pt_size, ded_ytd_x, y, &ctx.font_regular);
    y -= line_height;

    // Additional deductions (current only)
    for deduction in &payroll.deductions.additional {
        ctx.print_left(&deduction.name, pt_size, ded_desc_x, y, &ctx.font_regular);
        ctx.print_right(&format!("${}", format_decimal(deduction.amount)), pt_size, ded_current_x, y, &ctx.font_regular);
        ctx.print_right("$0.00", pt_size, ded_ytd_x + 25.0, y, &ctx.font_regular);
        y -= line_height;
    }

    // Total Deductions
    ctx.draw_line(ded_desc_x, y, ded_ytd_x, y, 0.2);
    y -= line_height * 1.5;

    let ytd_deductions_total = ytd.cpp + ytd.cpp2 + ytd.ei + ytd.federal_tax + ytd.provincial_tax;
    ctx.print_left("Total Deductions", pt_size, ded_desc_x, y, &ctx.font_bold);
    ctx.print_right(&format!("${}", format_decimal(payroll.deductions.total())), pt_size, ded_current_x, y, &ctx.font_bold);
    ctx.print_right(&format!("${}", format_decimal(ytd_deductions_total)), pt_size, ded_ytd_x, y, &ctx.font_regular);
    y -= line_height;
    ctx.draw_line(ded_desc_x, y, ded_ytd_x, y, 0.5);

    // === NET PAY ===
    let net_pay_left = CONTENT_WIDTH * 0.10;
    let net_pay_title_x = CONTENT_WIDTH * 0.40;
    let net_pay_cur_x = CONTENT_WIDTH * 0.60;
    let net_pay_ytd_x = CONTENT_WIDTH * 0.90;
    let net_pay_right = CONTENT_WIDTH * 0.91;
    y = line_height * 3.0;
    ctx.draw_rect(net_pay_left, y, net_pay_right - net_pay_left, line_height * 3.0, 0.5);
    y -= line_height * 1.75;

    ctx.print_left("NET PAY", pt_size, net_pay_title_x, y, &ctx.font_bold);
    ctx.print_right(&format!("${}", format_decimal(payroll.net_pay)), pt_size, net_pay_cur_x, y, &ctx.font_bold);
    ctx.print_right(&format!("${}", format_decimal(ytd.net_pay)), pt_size, net_pay_ytd_x, y, &ctx.font_regular);

    // Save PDF
    doc.save(&mut BufWriter::new(File::create(output_path)?))?;

    Ok(())
}

/// Parse company address into lines (split by comma or newline)
fn parse_address(address: &str) -> Vec<String> {
    if address.contains('\n') {
        address.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else {
        address.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    }
}

/// Mask SIN to show only last 3 digits
fn mask_sin(sin: &str) -> String {
    let digits: String = sin.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 3 {
        format!("*** *** {}", &digits[digits.len() - 3..])
    } else {
        "*** *** ***".to_string()
    }
}

/// Format decimal for display (2 decimal places with thousand separators)
fn format_decimal(value: Decimal) -> String {
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = if parts.len() > 1 { parts[1] } else { "00" };
    
    // Add thousand separators to integer part
    let mut result = String::new();
    let chars: Vec<char> = integer_part.chars().collect();
    let len = chars.len();
    
    for (i, c) in chars.iter().enumerate() {
        result.push(*c);
        // Add comma after every 3 digits from the right, but not at the start
        let position_from_right = len - i - 1;
        if position_from_right > 0 && position_from_right % 3 == 0 {
            result.push(',');
        }
    }
    
    format!("{}.{}", result, decimal_part)
}
