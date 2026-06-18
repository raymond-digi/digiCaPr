use chrono::{NaiveDate, Utc};
use cpr_core::models::{Company, Employee, Payroll};
use printpdf::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Clone)]
struct ReportRow {
    emp_num: String,
    name: String,
    reg_hours: Decimal,
    ot_hours: Decimal,
    base_earning: Decimal,
    allowance_earning: Decimal,
    benefit_earning: Decimal,
    bonus_earning: Decimal,
    commission_earning: Decimal,
    vacation_earning: Decimal,
    other_earning: Decimal,
    gross_earning: Decimal,
    pension_deduct: Decimal,
    taxable_income: Decimal,
    union_deduct: Decimal,
    group_ins_deduct: Decimal,
    other_deduct: Decimal,
    cpp: Decimal,
    cpp2: Decimal,
    ei: Decimal,
    federal_tax: Decimal,
    provincial_tax: Decimal,
    net_pay_adjust: Decimal,
    net_pay: Decimal,
}

// ******************************************************************
/// Context for report generation, holding page state and configuration
struct ReportContext<'a> {
    current_layer: PdfLayerReference,
    y: f64,
    page_num: u32,
    font_bold: IndirectFontRef,
    font_reg: IndirectFontRef,
    company: &'a Company,
    print_date: String,
    period_text: &'a str,
    col_x: Vec<f64>,
    col_widths: Vec<f64>,
    include_table_headers: bool,
}

// ******************************************************************
// A4 landscape mode 210 (length) x 297 (width)
const PAGE_WIDTH: f64 = 297.0;
const PAGE_HEIGHT: f64 = 210.0;
const MARGIN_LEFT: f64 = 15.0;
const MARGIN_RIGHT: f64 = 15.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM: f64 = 10.0;
const CONTENT_WIDTH: f64 = PAGE_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
const CONTENT_LENGTH: f64 = PAGE_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;

impl<'a> ReportContext<'a> {
    /// Create a new ReportContext with initialized column layout
    fn new(
        doc: &PdfDocumentReference,
        page: PdfPageIndex,
        layer: PdfLayerIndex,
        company: &'a Company,
        period_text: &'a str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let current_layer = doc.get_page(page).get_layer(layer);
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let font_reg = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let print_date = Utc::now().naive_utc().format("%Y-%m-%d %H:%M").to_string();
        // Column widths (mm) and col_gap
        let left_margin = 2.0;
        let col_gap = 1.0;
        let col_widths = vec![
            10.0, // col 0: Code / Employee_Name / Employee_No
            20.0, // col 1: Earning /  / Gross
            15.0, // col 2: Hours / Reg_Hr / OT_Hr
            20.0, // col 3: Allow/Benefit / Allowance / Benefit
            20.0, // col 4: Bonus/Comm / Bonus / Commission
            20.0, // col 5: Vacat/Others / Vacation_Pay / Other
            20.0, // col 6: Gross /  / Earning
            20.0, // col 7: Pension / Pension
            20.0, // col 8: Union/Deduct / Union / Group_Ins
            20.0, // col 9: Taxable / Taxable Income
            15.0, // col10: CPP / CPP1 / CPP2
            15.0, // col11: EI/Adj / Adjust / EI
            15.0, // col12: Tax / Fed / Prov
            20.0, // col13: Net /  / Net_Pay
        ];
        let mut col_x = vec![left_margin];
        for &width in &col_widths {
            col_x.push(*col_x.last().unwrap() + width + col_gap);
        }

        Ok(ReportContext {
            current_layer,
            y: CONTENT_LENGTH,
            page_num: 1,
            company,
            print_date,
            period_text,
            col_x,
            col_widths,
            font_bold,
            font_reg,
            include_table_headers: true,
        })
    }

    // ******************************************************************
    /// Print the table headers for the payroll data columns
    fn print_table_headers(&mut self) {
        let size = 7.5;
        let line_height = size * 0.45;
        let font_bold = self.font_bold.clone();

        // Header line 1
        // self.print_center("Employee", size, self.col_x[0], self.col_widths[0], &font_bold);
        // self.print_right("Earnings", size, self.col_x[1], self.col_widths[1], &font_bold);
        self.print_right("OT Hrs", size, self.col_x[2], self.col_widths[2], &font_bold);
        self.print_right("Allowance", size, self.col_x[3], self.col_widths[3], &font_bold);
        self.print_right("Bonus", size, self.col_x[4], self.col_widths[4], &font_bold);
        self.print_right("Vacation", size, self.col_x[5], self.col_widths[5], &font_bold);
        // self.print_right("Gross", size, self.col_x[6], self.col_widths[6], &font_bold);
        self.print_right("Pension", size, self.col_x[7], self.col_widths[7], &font_bold);
        self.print_right("Insurance", size, self.col_x[8], self.col_widths[8], &font_bold);
        // self.print_right("Taxable", size, self.col_x[9], self.col_widths[9], &font_bold);
        self.print_right("CPP2", size, self.col_x[10], self.col_widths[10], &font_bold);
        self.print_right("Adjust", size, self.col_x[11], self.col_widths[11], &font_bold);
        self.print_right("Fed", size, self.col_x[12], self.col_widths[12], &font_bold);
        // self.print_right("Net", size, self.col_x[13], self.col_widths[13], &font_bold);
        self.y -= line_height * 0.75;

        // Header line 1
        self.print_center("Employee", size, self.col_x[0], self.col_widths[0], &font_bold);
        self.print_right("Earnings", size, self.col_x[1], self.col_widths[1], &font_bold);
        self.print_right("Reg Hrs", size, self.col_x[2], self.col_widths[2], &font_bold);
        self.print_right("Benefits", size, self.col_x[3], self.col_widths[3], &font_bold);
        self.print_right("Commission", size, self.col_x[4], self.col_widths[4], &font_bold);
        self.print_right("Others", size, self.col_x[5], self.col_widths[5], &font_bold);
        self.print_right("Gross", size, self.col_x[6], self.col_widths[6], &font_bold);
        self.print_right("Union", size, self.col_x[7], self.col_widths[7], &font_bold);
        self.print_right("Deducted", size, self.col_x[8], self.col_widths[8], &font_bold);
        self.print_right("Taxable", size, self.col_x[9], self.col_widths[9], &font_bold);
        self.print_right("CPP", size, self.col_x[10], self.col_widths[10], &font_bold);
        self.print_right("EI", size, self.col_x[11], self.col_widths[11], &font_bold);
        self.print_right("Prov", size, self.col_x[12], self.col_widths[12], &font_bold);
        self.print_right("Net", size, self.col_x[13], self.col_widths[13], &font_bold);
        self.y -= line_height * 0.75;

        // Draw a single continuous line below headers spanning all columns
        let start_x = self.col_x[0];
        let end_x = self.col_x[13] + self.col_widths[13];
        self.draw_line(start_x, self.y, end_x, self.y, 0.3);
        self.y -= line_height * 1.5;
    }

    // ******************************************************************
    /// Print the standard page header (company name, page number, date, period)
    fn print_standard_page_header(&mut self) {
        let font_bold = self.font_bold.clone();
        let font_reg = self.font_reg.clone();
        self.print_center("PAYROLL REGISTER", 14.0, 0.0, CONTENT_WIDTH, &font_bold);
        self.y -= 6.0;
        self.print_left(&self.company.name, 12.0, 0.0, CONTENT_WIDTH, &font_bold);
        self.print_right(&format!("Page {}", self.page_num), 10.0, 0.0, CONTENT_WIDTH, &font_reg);
        self.y -= 4.0;
        self.print_center(self.period_text, 10.0, 0.0, CONTENT_WIDTH, &font_reg);
        self.y -= 10.0;

        let saved_y = self.y;
        self.y = 0.0;
        self.print_left(&format!("{}", self.print_date), 8.0, 0.0, CONTENT_WIDTH, &font_reg);
        self.y = saved_y;
    }

    // ******************************************************************
    /// Check if a new page is needed based on the space required.
    /// If a new page is needed, creates a new page, updates y and page_num,
    /// and prints the appropriate headers.
    fn check_new_page(&mut self, doc: &mut PdfDocumentReference, space_needed: f64) {
        if self.y < space_needed {
            let (new_page, new_layer) = doc.add_page(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Layer1");
            self.current_layer = doc.get_page(new_page).get_layer(new_layer);
            self.y = CONTENT_LENGTH;
            self.page_num += 1;
            self.print_standard_page_header();
            if self.include_table_headers {
                self.print_table_headers();
            }
        }
    }

    // ******************************************************************
    /// Update y position with page break handling.
    /// Takes a delta_y value (typically negative for moving down the page),
    /// checks if a new page is needed for the requested movement,
    /// and only updates y if no new page was created.
    /// This prevents y from being updated when a page break occurs,
    /// as check_new_page already resets y to CONTENT_LENGTH.
    fn update_y(&mut self, doc: &mut PdfDocumentReference, delta_y: f64) {
        let space_needed = delta_y.abs();
        self.check_new_page(doc, space_needed);
        // Only update y if no new page was created (y wasn't reset by check_new_page)
        if self.y != CONTENT_LENGTH {
            self.y -= space_needed;
        }
    }

    // ******************************************************************
    /// Calculate the width of a text string in millimeters for a given font and size.
    fn text_width_mm(&self, text: &str, size_pt: f64) -> f64 {
        // Approximation for Helvetica font: average character width ~0.5em at 10pt
        // Character widths in 1000 units per em (standard Helvetica metrics)
        let unit_narrow_1 = 250.0; // narrower characters: i l ! : ; | '
        let unit_space = 278.0; // space character
        let unit_narrow_2 = 333.0; // narrower characters: - . , * / ( ) [ ] { } f r t
        let unit_upper_narrow = 278.0; // narrow uppercase: I J L T
        let unit_upper_medium = 556.0; // medium uppercase: B C D E F G H K N O P Q R S U V X Y Z
        let unit_upper_wide = 722.0; // wide uppercase: A M W
        let unit_digit = 556.0; // digits (monospaced in Helvetica): 0 1 2 3 4 5 6 7 8 9
        let unit_avg = 500.0; // default for lowercase and other characters
                              // 6.0 if font is Courier for all chars
                              // Correct PDF scaling: (Units / 1000) * Font Size
        let scale = size_pt / 1000.0;

        let width_pt: f64 = text
            .chars()
            .map(|c| match c {
                // Lowercase narrow
                'i' | 'l' | '!' | ':' | ';' | '|' | '\'' => unit_narrow_1,
                // Space
                ' ' => unit_space,
                // Lowercase narrow and punctuation
                '-' | '.' | ',' | '*' | '/' | '(' | ')' | '[' | ']' | '{' | '}' | 'f' | 'r' | 't' => unit_narrow_2,
                // Uppercase narrow
                'I' | 'J' | 'L' | 'T' => unit_upper_narrow,
                // Uppercase medium
                'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'U' | 'V' | 'X' | 'Y' | 'Z' => unit_upper_medium,
                // Uppercase wide
                'A' | 'M' | 'W' => unit_upper_wide,
                // Digits (monospaced in Helvetica)
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => unit_digit,
                // Default (lowercase and other characters)
                _ => unit_avg,
            })
            .sum::<f64>()
            * scale; // Multiply total units by scale to get points

        // Convert Points to Millimeters
        width_pt * (25.4 / 72.0)
    }

    // ******************************************************************
    fn print_text(&self, text: &str, size_pt: f64, x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
        self.current_layer.use_text(text, size_pt, Mm(MARGIN_LEFT + x_mm), Mm(MARGIN_BOTTOM + y_mm), font);
    }

    // ******************************************************************
    // Text printing helpers - x positions are offset by MARGIN_LEFT
    fn print_left(&mut self, text: &str, size_pt: f64, x_mm: f64, _col_width_mm: f64, font: &printpdf::IndirectFontRef) {
        self.print_text(text, size_pt, x_mm, self.y, font);
    }

    // ******************************************************************
    fn print_right(&mut self, text: &str, size_pt: f64, x_mm: f64, col_width_mm: f64, font: &printpdf::IndirectFontRef) {
        let w = self.text_width_mm(text, size_pt) + 0.2;
        self.print_text(text, size_pt, x_mm + col_width_mm - w, self.y, font);
    }

    // ******************************************************************
    fn print_center(&mut self, text: &str, size_pt: f64, x_mm: f64, col_width_mm: f64, font: &printpdf::IndirectFontRef) {
        let w = self.text_width_mm(text, size_pt);
        self.print_text(text, size_pt, x_mm + (col_width_mm - w) * 0.5, self.y, font);
    }

    // ******************************************************************
    // fn print_left_at(&self, text: &str, size_pt: f64, x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
    //     self.print_text(text, size_pt, x_mm, y_mm, font);
    // }

    // ******************************************************************
    // fn print_right_at(&self, text: &str, size_pt: f64, x_right_mm: f64, y_mm: f64, font: &IndirectFontRef) {
    //     let w = self.text_width_mm(text, size_pt);
    //     self.print_text(text, size_pt, x_right_mm - w, y_mm, font);
    // }

    // ******************************************************************
    // fn print_center_at(&self, text: &str, size_pt: f64, center_x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
    //     let w = self.text_width_mm(text, size_pt);
    //     self.print_text(text, size_pt, center_x_mm - w * 0.5, y_mm, font);
    // }

    // ******************************************************************
    /// Draw a line from (x1, y1) to (x2, y2) with the specified thickness.
    /// Coordinates are offset by MARGIN_LEFT and MARGIN_BOTTOM,
    /// consistent with the print aligned functions.
    fn draw_line(&mut self, x1_mm: f64, y1_mm: f64, x2_mm: f64, y2_mm: f64, thickness_mm: f64) {
        use printpdf::{Line, Mm, Point};
        self.current_layer.set_outline_thickness(thickness_mm);
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
}

// ******************************************************************
pub fn generate_payroll_report<P: AsRef<Path>>(
    output_path: P,
    payrolls: &[Payroll],
    employees: &[Employee],
    company: &Company,
    period_start: NaiveDate,
    period_end: NaiveDate,
    pay_date: NaiveDate,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut doc, page1, layer1) = PdfDocument::new("Canadian Payroll Report", Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Layer1");

    let period_start_str = period_start.format("%Y-%m-%d").to_string();
    let period_end_str = period_end.format("%Y-%m-%d").to_string();
    let paid_on_str = pay_date.format("%Y-%m-%d").to_string();
    let period_text = format!("Payroll Period from {} to {} Paid on {}", period_start_str, period_end_str, paid_on_str);

    // Prepare rows and totals
    let mut rows: Vec<ReportRow> = Vec::new();
    let mut total_reg_hours = Decimal::ZERO;
    let mut total_ot_hours = Decimal::ZERO;
    let mut total_base_earning = Decimal::ZERO;
    let mut total_allowance_earning = Decimal::ZERO;
    let mut total_benefit_earning = Decimal::ZERO;
    let mut total_bonus_earning = Decimal::ZERO;
    let mut total_commission_earning = Decimal::ZERO;
    let mut total_vacation_earning = Decimal::ZERO;
    let mut total_other_earning = Decimal::ZERO;
    let mut total_gross_earning = Decimal::ZERO;
    let mut total_pension_deduct = Decimal::ZERO;
    let mut total_union_deduct = Decimal::ZERO;
    let mut total_group_ins_deduct = Decimal::ZERO;
    let mut total_other_deduct = Decimal::ZERO;
    let mut total_cpp = Decimal::ZERO;
    let mut total_cpp2 = Decimal::ZERO;
    let mut total_ei = Decimal::ZERO;
    let mut total_federal_tax = Decimal::ZERO;
    let mut total_provincial_tax = Decimal::ZERO;
    let mut total_net_pay_adjust = Decimal::ZERO;
    let mut total_net_pay = Decimal::ZERO;
    let mut total_taxable_income = Decimal::ZERO;

    for payroll in payrolls {
        if let Some(emp) = employees.iter().find(|e| e.id.map_or(false, |eid| eid == payroll.employee_id)) {
            let emp_num = emp.employee_number.clone();
            let name = format!("{}, {}", emp.last_name, emp.first_name);
            let reg_h = payroll.regular_hours.unwrap_or(Decimal::ZERO);
            let ot_h = payroll.overtime_hours.unwrap_or(Decimal::ZERO);
            let base_earning = payroll.gross_pay;
            let gross_earning = payroll.gross_pay + payroll.additional_earnings_total;
            let cpp = payroll.deductions.cpp;
            let cpp2 = payroll.deductions.cpp2;
            let ei = payroll.deductions.ei;
            let federal_tax = payroll.deductions.federal_tax;
            let provincial_tax = payroll.deductions.provincial_tax;
            let net_pay_adjust = Decimal::ZERO;

            let mut allowance_earning = Decimal::ZERO;
            let mut benefit_earning = Decimal::ZERO;
            let mut bonus_earning = Decimal::ZERO;
            let mut commission_earning = Decimal::ZERO;
            let mut vacation_earning = Decimal::ZERO;
            let mut other_earning = Decimal::ZERO;
            for earning in &payroll.additional_earnings {
                let t_lower = earning.earning_type.to_lowercase();
                if t_lower.contains("vacat") || t_lower.contains("vacation") {
                    vacation_earning += earning.amount;
                } else if t_lower.contains("commis") || t_lower.contains("commission") {
                    commission_earning += earning.amount;
                } else if t_lower.contains("allow") || t_lower.contains("allowance") || t_lower.contains("dir") {
                    allowance_earning += earning.amount;
                } else if t_lower.contains("bonus") || t_lower.contains("lump") {
                    bonus_earning += earning.amount;
                } else if t_lower.contains("benefit") {
                    benefit_earning += earning.amount;
                } else {
                    other_earning += earning.amount;
                }
            }

            let mut pension_deduct = Decimal::ZERO;
            let mut union_deduct = Decimal::ZERO;
            let mut group_ins_deduct = Decimal::ZERO;
            let mut other_deduct = Decimal::ZERO;
            for d in &payroll.deductions.additional {
                let n_lower = d.name.to_lowercase();
                if n_lower.contains("pension") {
                    pension_deduct += d.amount;
                } else if n_lower.contains("union") {
                    union_deduct += d.amount;
                } else if n_lower.contains("group")
                    || n_lower.contains("insur")
                    || n_lower.contains("insurance")
                    || n_lower.contains("health")
                    || n_lower.contains("medical")
                    || n_lower.contains("ben")
                {
                    group_ins_deduct += d.amount;
                } else {
                    other_deduct += d.amount;
                }
            }

            let taxable_income = gross_earning - pension_deduct;

            let row = ReportRow {
                emp_num,
                name,
                reg_hours: reg_h,
                ot_hours: ot_h,
                base_earning,
                allowance_earning,
                benefit_earning,
                bonus_earning,
                commission_earning,
                vacation_earning,
                other_earning,
                gross_earning,
                pension_deduct,
                taxable_income,
                union_deduct,
                group_ins_deduct,
                other_deduct,
                cpp,
                cpp2,
                ei,
                federal_tax,
                provincial_tax,
                net_pay_adjust,
                net_pay: payroll.net_pay,
            };
            rows.push(row);

            total_reg_hours += reg_h;
            total_ot_hours += ot_h;
            total_base_earning += base_earning;
            total_allowance_earning += allowance_earning;
            total_benefit_earning += benefit_earning;
            total_bonus_earning += bonus_earning;
            total_commission_earning += commission_earning;
            total_vacation_earning += vacation_earning;
            total_other_earning += other_earning;
            total_gross_earning += gross_earning;
            total_pension_deduct += pension_deduct;
            total_union_deduct += union_deduct;
            total_group_ins_deduct += group_ins_deduct;
            total_other_deduct += other_deduct;
            total_cpp += cpp;
            total_cpp2 += cpp2;
            total_ei += ei;
            total_federal_tax += federal_tax;
            total_provincial_tax += provincial_tax;
            total_net_pay_adjust += net_pay_adjust;
            total_net_pay += payroll.net_pay;
            total_taxable_income += taxable_income;
        }
    }

    let total_gross = total_gross_earning;
    let total_src_deduct = total_pension_deduct + total_union_deduct + total_group_ins_deduct + total_other_deduct;
    let total_tax = total_federal_tax + total_provincial_tax;
    let total_net = total_net_pay;

    // Sort rows by employee number
    rows.sort_by(|a, b| {
        let na: i32 = a.emp_num.parse().unwrap_or(0);
        let nb: i32 = b.emp_num.parse().unwrap_or(0);
        na.cmp(&nb)
    });

    // Create report context
    let mut ctx = ReportContext::new(&doc, page1, layer1, company, &period_text)?;

    ctx.print_standard_page_header();
    ctx.print_table_headers();

    // Print rows
    let font_reg = ctx.font_reg.clone();
    let font_bold = ctx.font_bold.clone();
    let line_size = 8.5;
    let line_height = line_size * 0.50; // (0.5625)

    let row_gap = line_height * 1.0; //gap between data record row
    for row in &rows {
        let row_space: f64 = line_height * 2.0; // space for a record row
        ctx.check_new_page(&mut doc, row_space);

        // Line A (Employee_Name, OT_Hr, ..., Fed)
        ctx.print_left(&row.name, line_size, ctx.col_x[0], CONTENT_WIDTH, &font_reg);
        if row.ot_hours != Decimal::ZERO {
            ctx.print_right(&format_hours_hhmm(row.ot_hours), line_size, ctx.col_x[2], ctx.col_widths[2], &font_reg);
        }
        ctx.print_right(&format_decimal(row.allowance_earning), line_size, ctx.col_x[3], ctx.col_widths[3], &font_reg);
        ctx.print_right(&format_decimal(row.bonus_earning), line_size, ctx.col_x[4], ctx.col_widths[4], &font_reg);
        ctx.print_right(&format_decimal(row.vacation_earning), line_size, ctx.col_x[5], ctx.col_widths[5], &font_reg);
        ctx.print_right(&format_decimal(row.pension_deduct), line_size, ctx.col_x[7], ctx.col_widths[7], &font_reg);
        ctx.print_right(&format_decimal(row.group_ins_deduct), line_size, ctx.col_x[8], ctx.col_widths[8], &font_reg);
        ctx.print_right(&format_decimal(row.cpp2), line_size, ctx.col_x[10], ctx.col_widths[10], &font_reg);
        ctx.print_right(&format_decimal(row.net_pay_adjust), line_size, ctx.col_x[11], ctx.col_widths[11], &font_reg);
        ctx.print_right(&format_decimal(row.federal_tax), line_size, ctx.col_x[12], ctx.col_widths[12], &font_reg);
        // col12 blank for line A
        ctx.update_y(&mut doc, line_height);

        // Line B (Employee_No, Reg_Hr, Gross, ..., Net_Pay)
        ctx.print_left(&row.emp_num, line_size, ctx.col_x[0], ctx.col_widths[0], &font_reg);
        ctx.print_right(&format_decimal(row.base_earning), line_size, ctx.col_x[1], ctx.col_widths[1], &font_reg);
        if row.reg_hours != Decimal::ZERO {
            ctx.print_right(&format_hours_hhmm(row.reg_hours), line_size, ctx.col_x[2], ctx.col_widths[2], &font_reg);
            // ctx.print_right(&format_decimal(row.reg_hours), line_size, ctx.col_x[2], ctx.col_widths[2], &font_reg);
        }
        ctx.print_right(&format_decimal(row.benefit_earning), line_size, ctx.col_x[3], ctx.col_widths[3], &font_reg);
        ctx.print_right(&format_decimal(row.commission_earning), line_size, ctx.col_x[4], ctx.col_widths[4], &font_reg);
        ctx.print_right(&format_decimal(row.other_earning), line_size, ctx.col_x[5], ctx.col_widths[5], &font_reg);
        ctx.print_right(&format_decimal(row.gross_earning), line_size, ctx.col_x[6], ctx.col_widths[6], &font_reg);
        ctx.print_right(&format_decimal(row.union_deduct), line_size, ctx.col_x[7], ctx.col_widths[7], &font_reg);
        ctx.print_right(&format_decimal(row.other_deduct), line_size, ctx.col_x[8], ctx.col_widths[8], &font_reg);
        ctx.print_right(&format_decimal(row.taxable_income), line_size, ctx.col_x[9], ctx.col_widths[8], &font_reg);
        ctx.print_right(&format_decimal(row.cpp), line_size, ctx.col_x[10], ctx.col_widths[10], &font_reg);
        ctx.print_right(&format_decimal(row.ei), line_size, ctx.col_x[11], ctx.col_widths[11], &font_reg);
        ctx.print_right(&format_decimal(row.provincial_tax), line_size, ctx.col_x[12], ctx.col_widths[12], &font_reg);
        ctx.print_right(&format_decimal(row.net_pay), line_size, ctx.col_x[13], ctx.col_widths[13], &font_reg);
        ctx.update_y(&mut doc, line_height + row_gap); // space after row
    }
    ctx.update_y(&mut doc, line_height * 1.0);

    ctx.include_table_headers = false;
    let sum_height: f64 = line_height * 7.0;
    ctx.check_new_page(&mut doc, sum_height);

    // Summary column widths and positions (independent from list columns)
    let sum_left_margin = 78.0;
    let sum_col_gap = 2.0;
    let sum_col_widths = vec![
        21.0, // col 0: Gross
        21.0, // col 1: Deductions
        21.0, // col 2: Taxable Income
        17.0, // col 3: CPP
        17.0, // col 4: CPP2
        17.0, // col 5: EI
        17.0, // col 6: Taxes
        17.0, // col 7: net_pay_adjust
        21.0, // col 8: Net
    ];
    let mut sum_col_x = vec![sum_left_margin];
    for &width in &sum_col_widths {
        sum_col_x.push(*sum_col_x.last().unwrap() + width + sum_col_gap);
    }

    // Aligned totals headers using summary column positions
    let sum_headers_size = 8.0;
    let headers = vec!["Gross", "Deduction", "Income", "CPP", "CPP2", "EI", "Tax", "Adj", "Net"];

    for (i, header) in headers.iter().enumerate() {
        ctx.print_right(header, sum_headers_size, sum_col_x[i], sum_col_widths[i], &font_bold);
    }
    ctx.update_y(&mut doc, line_height * 0.5);

    let start_x = sum_col_x[0];
    let end_x = sum_col_x[8] + sum_col_widths[8];
    ctx.draw_line(start_x, ctx.y, end_x, ctx.y, 0.3);
    ctx.update_y(&mut doc, line_height * 1.5);

    // Total line positioned, right-aligned using summary columns
    ctx.print_right(&format_decimal(total_gross), line_size, sum_col_x[0], sum_col_widths[0], &font_bold);
    ctx.print_right(&format_decimal(total_src_deduct), line_size, sum_col_x[1], sum_col_widths[1], &font_bold);
    ctx.print_right(&format_decimal(total_taxable_income), line_size, sum_col_x[2], sum_col_widths[2], &font_bold);
    ctx.print_right(&format_decimal(total_cpp), line_size, sum_col_x[3], sum_col_widths[3], &font_bold);
    ctx.print_right(&format_decimal(total_cpp2), line_size, sum_col_x[4], sum_col_widths[4], &font_bold);
    ctx.print_right(&format_decimal(total_ei), line_size, sum_col_x[5], sum_col_widths[5], &font_bold);
    ctx.print_right(&format_decimal(total_tax), line_size, sum_col_x[6], sum_col_widths[6], &font_bold);
    ctx.print_right(&format_decimal(total_net_pay_adjust), line_size, sum_col_x[7], sum_col_widths[7], &font_bold);
    ctx.print_right(&format_decimal(total_net), line_size, sum_col_x[8], sum_col_widths[8], &font_bold);
    ctx.update_y(&mut doc, line_height * 2.0);

    // Employee's Deduction (same numbers)
    let font_reg = ctx.font_reg.clone();
    ctx.print_right("Employee's Deduction:", line_size, 20.0, 60.0, &font_reg);
    ctx.print_right(&format_decimal(total_cpp), line_size, sum_col_x[3], sum_col_widths[3], &font_reg);
    ctx.print_right(&format_decimal(total_cpp2), line_size, sum_col_x[4], sum_col_widths[4], &font_reg);
    ctx.print_right(&format_decimal(total_ei), line_size, sum_col_x[5], sum_col_widths[5], &font_reg);
    ctx.print_right(&format_decimal(total_tax), line_size, sum_col_x[6], sum_col_widths[6], &font_reg);
    ctx.update_y(&mut doc, line_height);

    // Employer's Contribution
    let employer_ei = total_ei * Decimal::new(14, 1);
    ctx.print_right("Employer's Contribution:", line_size, 20.0, 60.0, &font_reg);
    ctx.print_right(&format_decimal(total_cpp), line_size, sum_col_x[3], sum_col_widths[3], &font_reg);
    ctx.print_right(&format_decimal(total_cpp2), line_size, sum_col_x[4], sum_col_widths[4], &font_reg);
    ctx.print_right(&format_decimal(employer_ei), line_size, sum_col_x[5], sum_col_widths[5], &font_reg);
    ctx.update_y(&mut doc, line_height);

    // Payable to Government
    let payable_cpp = total_cpp * Decimal::from(2u32);
    let payable_cpp2 = total_cpp2 * Decimal::from(2u32);
    let payable_ei = total_ei + employer_ei;
    let payable_tax = total_tax;
    let payable_total = payable_cpp + payable_cpp2 + payable_ei + payable_tax;
    ctx.print_right("Payable to Government:", line_size, 20.0, 60.0, &font_reg);
    ctx.print_right(&format_decimal(payable_cpp), line_size, sum_col_x[3], sum_col_widths[3], &font_reg);
    ctx.print_right(&format_decimal(payable_cpp2), line_size, sum_col_x[4], sum_col_widths[4], &font_reg);
    ctx.print_right(&format_decimal(payable_ei), line_size, sum_col_x[5], sum_col_widths[5], &font_reg);
    ctx.print_right(&format_decimal(payable_tax), line_size, sum_col_x[6], sum_col_widths[6], &font_reg);
    ctx.print_right("Total:", line_size, sum_col_x[7], sum_col_widths[7], &font_bold);
    ctx.print_right(&format_decimal(payable_total), line_size, sum_col_x[8], sum_col_widths[8], &font_bold);

    let mut buf_writer = BufWriter::new(File::create(output_path)?);
    doc.save(&mut buf_writer)?;

    Ok(())
}

/// Format hours as hh:mm (rounded to nearest minute)
pub fn format_hours_hhmm(hours: Decimal) -> String {
    let total_minutes = (hours * Decimal::from(60)).round_dp(0).to_string().parse::<i64>().unwrap_or(0);
    let h = total_minutes / 60;
    let m = total_minutes % 60;
    format!("{}:{:02}", h, m)
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
