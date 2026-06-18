use crate::currency::convert_cents;
use crate::DbResult;
use cpr_core::models::payroll::YtdTotals;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

pub struct YtdRepository {
    conn: Arc<Mutex<Connection>>,
}

impl YtdRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get YTD totals calculated from history, history_earning, and history_deduction
    pub fn get_or_create(&self, employee_id: i64, year: i32) -> DbResult<YtdTotals> {
        self.get(employee_id, year).map(|opt| opt.unwrap_or_else(|| YtdTotals::new(employee_id, year)))
    }

    /// Get YTD totals calculated from history
    pub fn get(&self, employee_id: i64, year: i32) -> DbResult<Option<YtdTotals>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT
                COALESCE(CAST(SUM(gross_pay + additional_earnings) AS INTEGER), 0) as ytd_gross,
                COALESCE(CAST(SUM(cpp_deduction) AS INTEGER), 0) as ytd_cpp,
                COALESCE(CAST(SUM(cpp2_deduction) AS INTEGER), 0) as ytd_cpp2,
                COALESCE(CAST(SUM(ei_deduction) AS INTEGER), 0) as ytd_ei,
                COALESCE(CAST(SUM(federal_tax) AS INTEGER), 0) as ytd_federal,
                COALESCE(CAST(SUM(provincial_tax) AS INTEGER), 0) as ytd_provincial,
                COALESCE(CAST(SUM(net_pay) AS INTEGER), 0) as ytd_net
             FROM history
             WHERE employee_id = ?1
               AND strftime('%Y', pay_date) = ?2",
        )?;

        let mut totals = stmt.query_row(params![employee_id, year.to_string()], |row| {
            let gross_cents: i64 = row.get(0)?;
            let cpp_cents: i64 = row.get(1)?;
            let cpp2_cents: i64 = row.get(2)?;
            let ei_cents: i64 = row.get(3)?;
            let federal_cents: i64 = row.get(4)?;
            let provincial_cents: i64 = row.get(5)?;
            let net_cents: i64 = row.get(6)?;

            Ok(YtdTotals {
                employee_id,
                year,
                gross_pay: convert_cents(gross_cents, "ytd_gross_pay")?,
                cpp: convert_cents(cpp_cents, "ytd_cpp")?,
                cpp2: convert_cents(cpp2_cents, "ytd_cpp2")?,
                ei: convert_cents(ei_cents, "ytd_ei")?,
                federal_tax: convert_cents(federal_cents, "ytd_federal_tax")?,
                provincial_tax: convert_cents(provincial_cents, "ytd_provincial_tax")?,
                net_pay: convert_cents(net_cents, "ytd_net_pay")?,
                rpp_contributions: rust_decimal::Decimal::ZERO,
                pension_adjustment: rust_decimal::Decimal::ZERO,
            })
        })?;

        // Sum PensionRrsp (Box 20 - RPP contributions) from history_deduction
        let rpp_cents: i64 = conn.query_row(
            "SELECT COALESCE(SUM(d.amount), 0)
             FROM history_deduction d
             JOIN history h ON d.payroll_id = h.id
             WHERE h.employee_id = ?1
               AND strftime('%Y', h.pay_date) = ?2
               AND d.deduction_type = 'pension_rrsp'",
            params![employee_id, year.to_string()],
            |row| row.get(0),
        )?;

        // Sum UnionDues (Box 52 - Pension adjustment) from history_deduction
        let pension_cents: i64 = conn.query_row(
            "SELECT COALESCE(SUM(d.amount), 0)
             FROM history_deduction d
             JOIN history h ON d.payroll_id = h.id
             WHERE h.employee_id = ?1
               AND strftime('%Y', h.pay_date) = ?2
               AND d.deduction_type = 'union_dues'",
            params![employee_id, year.to_string()],
            |row| row.get(0),
        )?;

        totals.rpp_contributions = convert_cents(rpp_cents, "rpp_contributions")?;
        totals.pension_adjustment = convert_cents(pension_cents, "pension_adjustment")?;

        // If all values are zero, treat it as no data
        if totals.gross_pay.is_zero()
            && totals.cpp.is_zero()
            && totals.ei.is_zero()
            && totals.federal_tax.is_zero()
            && totals.provincial_tax.is_zero()
            && totals.net_pay.is_zero()
        {
            Ok(None)
        } else {
            Ok(Some(totals))
        }
    }

    /// Get all employees with YTD records for a given year (calculated from history)
    pub fn list_by_year(&self, year: i32) -> DbResult<Vec<YtdTotals>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT DISTINCT employee_id FROM history
             WHERE strftime('%Y', pay_date) = ?1
             ORDER BY employee_id",
        )?;

        let employee_ids: Vec<i64> = stmt.query_map([year.to_string()], |row| row.get(0))?.collect::<Result<_, _>>()?;

        drop(stmt);
        drop(conn);

        let mut ytds = Vec::new();
        for employee_id in employee_ids {
            if let Some(ytd) = self.get(employee_id, year)? {
                ytds.push(ytd);
            }
        }

        Ok(ytds)
    }
}
