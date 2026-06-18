use chrono::{NaiveDate, Utc};
use cpr_core::models::{Company, Employee, Payroll};
use printpdf::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Clone)]
struct ReportRow {
    pay_period: String,
    emp_num: String,
    name: String,
    gross_earning: Decimal,
    pension_deduct: Decimal,
    union_deduct: Decimal,
    group_ins_deduct: Decimal,
    other_deduct: Decimal,
    taxable_income: Decimal,
    cpp: Decimal,
    cpp2: Decimal,
    ei: Decimal,
    federal_tax: Decimal,
    provincial_tax: Decimal,
    net_pay: Decimal,
}

// Context for report generation
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

// A4 landscape
const PAGE_WIDTH: f64 = 297.0;
const PAGE_HEIGHT: f64 = 210.0;
const MARGIN_LEFT: f64 = 15.0;
const MARGIN_RIGHT: f64 = 15.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM: f64 = 10.0;
const CONTENT_WIDTH: f64 = PAGE_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
const CONTENT_LENGTH: f64 = PAGE_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;

impl<'a> ReportContext<'a> {
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
        // Simplified columns: Pay Period, Emp No, Name, Gross, Pension, Union/Ins, Other Ded, Taxable, CPP, CPP2, EI, Fed Tax, Prov Tax, Net
        let left_margin = 2.0;
        let col_gap = 1.0;
        let col_widths = vec![
            18.0, // 0: Pay Period
            12.0, // 1: Emp No
            36.0, // 2: Name
            16.0, // 3: Gross Pay
            16.0, // 4: Pension
            16.0, // 5: Union/Ins Ded
            16.0, // 6: Other Ded
            16.0, // 7: Taxable Income
            14.0, // 8: CPP
            14.0, // 9: CPP2
            14.0, // 10: EI
            16.0, // 11: Fed Tax
            16.0, // 12: Prov Tax
            18.0, // 13: Net Pay
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

    fn print_table_headers(&mut self) {
        let size = 8.0;
        let line_height = size * 0.45;
        let font_bold = self.font_bold.clone();

        self.print_left("Period", size, self.col_x[0], self.col_widths[0], &font_bold);
        self.print_left("Emp No", size, self.col_x[1], self.col_widths[1], &font_bold);
        self.print_left("Name", size, self.col_x[2], self.col_widths[2], &font_bold);
        self.print_right("Gross", size, self.col_x[3], self.col_widths[3], &font_bold);
        self.print_right("Pension", size, self.col_x[4], self.col_widths[4], &font_bold);
        self.print_right("Union/Ins", size, self.col_x[5], self.col_widths[5], &font_bold);
        self.print_right("Other Ded", size, self.col_x[6], self.col_widths[6], &font_bold);
        self.print_right("Taxable", size, self.col_x[7], self.col_widths[7], &font_bold);
        self.print_right("CPP", size, self.col_x[8], self.col_widths[8], &font_bold);
        self.print_right("CPP2", size, self.col_x[9], self.col_widths[9], &font_bold);
        self.print_right("EI", size, self.col_x[10], self.col_widths[10], &font_bold);
        self.print_right("Fed Tax", size, self.col_x[11], self.col_widths[11], &font_bold);
        self.print_right("Prov Tax", size, self.col_x[12], self.col_widths[12], &font_bold);
        self.print_right("Net", size, self.col_x[13], self.col_widths[13], &font_bold);
        self.y -= line_height * 0.75;

        let start_x = self.col_x[0];
        let end_x = self.col_x[13] + self.col_widths[13];
        self.draw_line(start_x, self.y, end_x, self.y, 0.3);
        self.y -= line_height * 1.5;
    }

    fn print_standard_page_header(&mut self) {
        let font_bold = self.font_bold.clone();
        let font_reg = self.font_reg.clone();
        self.print_center("REMITTANCE REGISTER", 14.0, 0.0, CONTENT_WIDTH, &font_bold);
        self.y -= 8.0;
        self.print_left(&self.company.name, 12.0, 0.0, CONTENT_WIDTH, &font_bold);
        self.print_right(&format!("Page {}", self.page_num), 10.0, 0.0, CONTENT_WIDTH, &font_reg);
        self.y -= 4.0;
        self.print_center(self.period_text, 10.0, 0.0, CONTENT_WIDTH, &font_reg);
        self.y -= 9.0;

        let saved_y = self.y;
        self.y = 0.0;
        self.print_left(&format!("{}", self.print_date), 8.0, 0.0, CONTENT_WIDTH, &font_reg);
        self.y = saved_y;
    }

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

    fn update_y(&mut self, doc: &mut PdfDocumentReference, delta_y: f64) {
        let space_needed = delta_y.abs();
        self.check_new_page(doc, space_needed);
        if self.y != CONTENT_LENGTH {
            self.y -= space_needed;
        }
    }

    fn text_width_mm(&self, text: &str, size_pt: f64) -> f64 {
        let scale = size_pt / 1000.0;
        let width_pt: f64 = text
            .chars()
            .map(|c| match c {
                'i' | 'l' | '!' | ':' | ';' | '|' | '\'' => 250.0,
                ' ' => 278.0,
                '-' | '.' | ',' | '*' | '/' | '(' | ')' | '[' | ']' | '{' | '}' | 'f' | 'r' | 't' => 333.0,
                'I' | 'J' | 'L' | 'T' => 278.0,
                'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 556.0,
                'A' | 'M' | 'W' => 722.0,
                '0'..='9' => 556.0,
                _ => 500.0,
            })
            .sum::<f64>()
            * scale;
        width_pt * (25.4 / 72.0)
    }

    fn print_text(&self, text: &str, size_pt: f64, x_mm: f64, y_mm: f64, font: &IndirectFontRef) {
        self.current_layer.use_text(text, size_pt, Mm(MARGIN_LEFT + x_mm), Mm(MARGIN_BOTTOM + y_mm), font);
    }

    fn print_left(&mut self, text: &str, size_pt: f64, x_mm: f64, _col_width_mm: f64, font: &printpdf::IndirectFontRef) {
        self.print_text(text, size_pt, x_mm, self.y, font);
    }

    fn print_right(&mut self, text: &str, size_pt: f64, x_mm: f64, col_width_mm: f64, font: &printpdf::IndirectFontRef) {
        let w = self.text_width_mm(text, size_pt) + 0.2;
        self.print_text(text, size_pt, x_mm + col_width_mm - w, self.y, font);
    }

    fn print_center(&mut self, text: &str, size_pt: f64, x_mm: f64, col_width_mm: f64, font: &printpdf::IndirectFontRef) {
        let w = self.text_width_mm(text, size_pt);
        self.print_text(text, size_pt, x_mm + (col_width_mm - w) * 0.5, self.y, font);
    }

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

pub fn generate_remittance_report<P: AsRef<Path>>(
    output_path: P,
    payrolls: &[Payroll],
    employees: &[Employee],
    company: &Company,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut doc, page1, layer1) = PdfDocument::new("Canadian Remittance Report", Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Layer1");

    let period_start_str = period_start.format("%Y-%m-%d").to_string();
    let period_end_str = period_end.format("%Y-%m-%d").to_string();
    let period_text = format!("Remittance Period from {} to {}", period_start_str, period_end_str);

    // Prepare rows and totals
    let mut rows: Vec<ReportRow> = Vec::new();
    let mut total_gross_earning = Decimal::ZERO;
    let mut total_pension_deduct = Decimal::ZERO;
    let mut total_union_deduct = Decimal::ZERO;
    let mut total_group_ins_deduct = Decimal::ZERO;
    let mut total_other_deduct = Decimal::ZERO;
    let mut total_taxable_income = Decimal::ZERO;
    let mut total_cpp = Decimal::ZERO;
    let mut total_cpp2 = Decimal::ZERO;
    let mut total_ei = Decimal::ZERO;
    let mut total_federal_tax = Decimal::ZERO;
    let mut total_provincial_tax = Decimal::ZERO;
    let mut total_net_pay = Decimal::ZERO;

    for payroll in payrolls {
        if let Some(emp) = employees.iter().find(|e| e.id.map_or(false, |eid| eid == payroll.employee_id)) {
            let emp_num = emp.employee_number.clone();
            let name = format!("{}, {}", emp.last_name, emp.first_name);
            let period_str = format!("{}-{}", payroll.pay_period_start.format("%m/%d"), payroll.pay_period_end.format("%m/%d"));
            let gross_earning = payroll.gross_pay;
            let cpp = payroll.deductions.cpp;
            let cpp2 = payroll.deductions.cpp2;
            let ei = payroll.deductions.ei;
            let federal_tax = payroll.deductions.federal_tax;
            let provincial_tax = payroll.deductions.provincial_tax;

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
                {
                    group_ins_deduct += d.amount;
                } else {
                    other_deduct += d.amount;
                }
            }

            let taxable_income = gross_earning - pension_deduct;

            let row = ReportRow {
                pay_period: period_str,
                emp_num,
                name,
                gross_earning,
                pension_deduct,
                union_deduct,
                group_ins_deduct,
                other_deduct,
                taxable_income,
                cpp,
                cpp2,
                ei,
                federal_tax,
                provincial_tax,
                net_pay: payroll.net_pay,
            };
            rows.push(row);

            total_gross_earning += gross_earning;
            total_pension_deduct += pension_deduct;
            total_union_deduct += union_deduct;
            total_group_ins_deduct += group_ins_deduct;
            total_other_deduct += other_deduct;
            total_taxable_income += taxable_income;
            total_cpp += cpp;
            total_cpp2 += cpp2;
            total_ei += ei;
            total_federal_tax += federal_tax;
            total_provincial_tax += provincial_tax;
            total_net_pay += payroll.net_pay;
        }
    }

    // Sort rows by employee number
    rows.sort_by(|a, b| {
        let na: i32 = a.emp_num.parse().unwrap_or(0);
        let nb: i32 = b.emp_num.parse().unwrap_or(0);
        na.cmp(&nb)
    });

    // Create report context
    let mut ctx = ReportContext::new(&doc, page1, layer1, company, &period_text)?;

    let font_reg = ctx.font_reg.clone();
    // let font_bold = ctx.font_bold.clone();

    ctx.print_standard_page_header();
    ctx.print_table_headers();

    // Print rows
    let line_size = 9.0;
    let line_height = line_size * 0.50;

    for row in &rows {
        let row_space: f64 = line_height * 2.0;
        ctx.check_new_page(&mut doc, row_space);

        // Single line: pay_period, emp_num, name left-aligned; others right-aligned
        ctx.print_left(&row.pay_period, line_size, ctx.col_x[0], ctx.col_widths[0], &font_reg);
        ctx.print_left(&row.emp_num, line_size, ctx.col_x[1], ctx.col_widths[1], &font_reg);
        ctx.print_left(&row.name, line_size, ctx.col_x[2], ctx.col_widths[2], &font_reg);
        ctx.print_right(&format_decimal(row.gross_earning), line_size, ctx.col_x[3], ctx.col_widths[3], &font_reg);
        ctx.print_right(&format_decimal(row.pension_deduct), line_size, ctx.col_x[4], ctx.col_widths[4], &font_reg);
        ctx.print_right(&format_decimal(row.union_deduct + row.group_ins_deduct), line_size, ctx.col_x[5], ctx.col_widths[5], &font_reg);
        ctx.print_right(&format_decimal(row.other_deduct), line_size, ctx.col_x[6], ctx.col_widths[6], &font_reg);
        ctx.print_right(&format_decimal(row.taxable_income), line_size, ctx.col_x[7], ctx.col_widths[7], &font_reg);
        ctx.print_right(&format_decimal(row.cpp), line_size, ctx.col_x[8], ctx.col_widths[8], &font_reg);
        ctx.print_right(&format_decimal(row.cpp2), line_size, ctx.col_x[9], ctx.col_widths[9], &font_reg);
        ctx.print_right(&format_decimal(row.ei), line_size, ctx.col_x[10], ctx.col_widths[10], &font_reg);
        ctx.print_right(&format_decimal(row.federal_tax), line_size, ctx.col_x[11], ctx.col_widths[11], &font_reg);
        ctx.print_right(&format_decimal(row.provincial_tax), line_size, ctx.col_x[12], ctx.col_widths[12], &font_reg);
        ctx.print_right(&format_decimal(row.net_pay), line_size, ctx.col_x[13], ctx.col_widths[13], &font_reg);
        ctx.update_y(&mut doc, line_height * 1.5); // space after row
    }
    ctx.update_y(&mut doc, line_height * 1.0);

    ctx.include_table_headers = false;
    let sum_height: f64 = line_height * 10.0;
    ctx.check_new_page(&mut doc, sum_height);

    // Summary totals (simplified positions)
    let sum_left_margin = 40.0;
    let sum_col_gap = 2.0;
    let sum_col_widths = vec![
        20.0, // Gross
        20.0, // Pension
        20.0, // Union/Ins
        20.0, // Other Ded
        20.0, // Taxable
        15.0, // CPP
        15.0, // CPP2
        15.0, // EI
        20.0, // Fed Tax
        20.0, // Prov Tax
        20.0, // Net
    ];
    let mut sum_col_x = vec![sum_left_margin];
    for &width in &sum_col_widths {
        sum_col_x.push(*sum_col_x.last().unwrap() + width + sum_col_gap);
    }

    let font_bold = ctx.font_bold.clone();
    let font_reg = ctx.font_reg.clone();
    let sum_headers_size = 8.0;
    let headers = vec!["Gross", "Pension", "Union/Ins", "Other Ded", "Taxable", "CPP", "CPP2", "EI", "Fed Tax", "Prov Tax", "Net"];
    for (i, header) in headers.iter().enumerate() {
        ctx.print_right(header, sum_headers_size, sum_col_x[i], sum_col_widths[i], &font_bold);
    }
    ctx.update_y(&mut doc, line_height * 0.5);

    let start_x = sum_col_x[0];
    let end_x = sum_col_x[10] + sum_col_widths[10];
    ctx.draw_line(start_x, ctx.y, end_x, ctx.y, 0.3);
    ctx.update_y(&mut doc, line_height * 1.5);

    // Totals line
    ctx.print_right(&format_decimal(total_gross_earning), line_size, sum_col_x[0], sum_col_widths[0], &font_bold);
    ctx.print_right(&format_decimal(total_pension_deduct), line_size, sum_col_x[1], sum_col_widths[1], &font_bold);
    ctx.print_right(&format_decimal(total_union_deduct + total_group_ins_deduct), line_size, sum_col_x[2], sum_col_widths[2], &font_bold);
    ctx.print_right(&format_decimal(total_other_deduct), line_size, sum_col_x[3], sum_col_widths[3], &font_bold);
    ctx.print_right(&format_decimal(total_taxable_income), line_size, sum_col_x[4], sum_col_widths[4], &font_bold);
    ctx.print_right(&format_decimal(total_cpp), line_size, sum_col_x[5], sum_col_widths[5], &font_bold);
    ctx.print_right(&format_decimal(total_cpp2), line_size, sum_col_x[6], sum_col_widths[6], &font_bold);
    ctx.print_right(&format_decimal(total_ei), line_size, sum_col_x[7], sum_col_widths[7], &font_bold);
    ctx.print_right(&format_decimal(total_federal_tax), line_size, sum_col_x[8], sum_col_widths[8], &font_bold);
    ctx.print_right(&format_decimal(total_provincial_tax), line_size, sum_col_x[9], sum_col_widths[9], &font_bold);
    ctx.print_right(&format_decimal(total_net_pay), line_size, sum_col_x[10], sum_col_widths[10], &font_bold);
    ctx.update_y(&mut doc, line_height * 2.0);

    // Employee's Deduction
    ctx.print_right("Employee's Deduction:", line_size, 40.0, 60.0, &font_reg);
    ctx.print_right(&format_decimal(total_cpp), line_size, sum_col_x[5], sum_col_widths[5], &font_reg);
    ctx.print_right(&format_decimal(total_cpp2), line_size, sum_col_x[6], sum_col_widths[6], &font_reg);
    ctx.print_right(&format_decimal(total_ei), line_size, sum_col_x[7], sum_col_widths[7], &font_reg);
    ctx.print_right(&format_decimal(total_federal_tax + total_provincial_tax), line_size, sum_col_x[8], sum_col_widths[8], &font_reg);
    ctx.update_y(&mut doc, line_height);

    // Employer's Contribution
    let employer_ei = total_ei * Decimal::new(14, 1); // 1.4x employer contribution
    ctx.print_right("Employer's Contribution:", line_size, 40.0, 60.0, &font_reg);
    ctx.print_right(&format_decimal(total_cpp), line_size, sum_col_x[5], sum_col_widths[5], &font_reg);
    ctx.print_right(&format_decimal(total_cpp2), line_size, sum_col_x[6], sum_col_widths[6], &font_reg);
    ctx.print_right(&format_decimal(employer_ei), line_size, sum_col_x[7], sum_col_widths[7], &font_reg);
    ctx.update_y(&mut doc, line_height);

    // Payable to Government
    let payable_cpp = total_cpp * Decimal::TWO;
    let payable_cpp2 = total_cpp2 * Decimal::TWO;
    let payable_ei = total_ei + employer_ei;
    let payable_tax = total_federal_tax + total_provincial_tax;
    let payable_total = payable_cpp + payable_cpp2 + payable_ei + payable_tax;
    ctx.print_right("Payable to Government:", line_size, 40.0, 60.0, &font_reg);
    ctx.print_right(&format_decimal(payable_cpp), line_size, sum_col_x[5], sum_col_widths[5], &font_reg);
    ctx.print_right(&format_decimal(payable_cpp2), line_size, sum_col_x[6], sum_col_widths[6], &font_reg);
    ctx.print_right(&format_decimal(payable_ei), line_size, sum_col_x[7], sum_col_widths[7], &font_reg);
    ctx.print_right(&format_decimal(payable_tax), line_size, sum_col_x[8], sum_col_widths[8], &font_reg);
    ctx.print_right("Total:", line_size, sum_col_x[9], sum_col_widths[9], &font_bold);
    ctx.print_right(&format_decimal(payable_total), line_size, sum_col_x[10], sum_col_widths[10], &font_bold);

    let mut buf_writer = BufWriter::new(File::create(output_path)?);
    doc.save(&mut buf_writer)?;

    Ok(())
}

fn format_decimal(value: Decimal) -> String {
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = if parts.len() > 1 { parts[1] } else { "00" };

    let mut result = String::new();
    let chars: Vec<char> = integer_part.chars().collect();
    let len = chars.len();

    for (i, c) in chars.iter().enumerate() {
        result.push(*c);
        let position_from_right = len - i - 1;
        if position_from_right > 0 && position_from_right % 3 == 0 {
            result.push(',');
        }
    }

    format!("{}.{}", result, decimal_part)
}
