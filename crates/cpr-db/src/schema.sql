-- =============================================
-- CPR Database Schema - Initial Version (embedded in crates/cpr-db/src/schema.rs via include_str!)
-- =============================================
--
-- Usage: Copy-paste into SQLite to recreate schema.
-- Note: Run PRAGMA foreign_keys = ON;
--
-- UPDATE POLICY: When schema changes (new migration), create schema_vN.sql
-- Running schema::initialize_database() (called on every DB open) automatically adds missing indexes.
--

PRAGMA foreign_keys = ON;

-- Schema versioning tables (created in init_schema_version)
CREATE TABLE IF NOT EXISTS schema_version (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    version INTEGER NOT NULL,
    applied_at TEXT NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS migration_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version INTEGER NOT NULL,
    description TEXT NOT NULL,
    applied_at TEXT NOT NULL,
    execution_time_ms INTEGER,
    success INTEGER NOT NULL DEFAULT 1
);

-- Registry key-value store for application settings
-- Similar to Windows Registry, stores hierarchical key-value pairs
-- Key paths follow format: "category/subcategory/name" (e.g., "company/info/name", "security/password/hash")
CREATE TABLE IF NOT EXISTS registry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_path TEXT NOT NULL UNIQUE,  -- e.g., "company/name", "security/password"
    value_type TEXT NOT NULL CHECK(value_type IN ('String', 'Integer', 'Boolean', 'Json')),
    value_string TEXT,              -- For String, Boolean, Json types
    value_integer INTEGER,          -- For Integer type
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_registry_key_path ON registry(key_path);

-- Database configuration for password protection and recovery
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    password_hash TEXT,           -- Argon2 hashed main password
    recovery_key_hash TEXT,       -- Argon2 hashed master recovery key
    password_hint TEXT,           -- Hint for remembering password
    backup_email TEXT,            -- User backup email for recovery notifications
    support_email TEXT DEFAULT 'support@cpr.com',  -- Master support email for admin recovery
    is_locked INTEGER NOT NULL DEFAULT 0 CHECK(is_locked IN (0,1)),  -- 1 if password required to open DB
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Company: Single record for the company (id=1)
CREATE TABLE IF NOT EXISTS company (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL CHECK(name <> ''),
    business_number TEXT,  -- Business Number (BN)
    address TEXT,
    province TEXT NOT NULL CHECK(province IN ('ON','QC','BC','AB','SK','MB','NS','NB','PE','NL','NT','YT','NU')),
    created_at TEXT NOT NULL
);

-- Employee: Core employee record with pay/tax info
CREATE TABLE IF NOT EXISTS employee (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
    employee_number TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    notes TEXT,  -- General employee notes: accommodations, preferences, important dates, HR notes, performance summaries
    sin TEXT NOT NULL UNIQUE,  -- Social Insurance Number
    date_of_birth TEXT NOT NULL DEFAULT '1990-01-01',
    address_street TEXT,
    address_city TEXT,
    address_province TEXT NOT NULL CHECK(address_province IN ('ON','QC','BC','AB','SK','MB','NS','NB','PE','NL','NT','YT','NU')),
    address_postal_code TEXT,
    hire_date TEXT NOT NULL,
    hire_province TEXT NOT NULL CHECK(hire_province IN ('ON','QC','BC','AB','SK','MB','NS','NB','PE','NL','NT','YT','NU')),  -- Province for tax calculations
    termination_date TEXT,
    pay_type TEXT NOT NULL CHECK(pay_type IN ('Hourly', 'Weekly', 'Monthly', 'Annual')),
    pay_rate INTEGER NOT NULL,  -- Store as cents (e.g., $25.50 = 2550)
    vacation_pay_rate INTEGER NOT NULL DEFAULT 400,  -- Store as basis points (e.g., 4% = 400)
    vacation_balance INTEGER NOT NULL DEFAULT 0,  -- Store as cents (fast lookup for vacation balance)
    vacation_balance_days INTEGER NOT NULL DEFAULT 0,  -- Store as thousandths of days (for non-hourly employees)
    overtime_multiplier INTEGER NOT NULL DEFAULT 150,  -- Store as basis points (e.g., 1.5x = 150)
    additional_tax_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    ei_exempt INTEGER NOT NULL DEFAULT 0 CHECK(ei_exempt IN (0,1)),  -- Whether employee is exempt from EI deductions
    cpp_exempt INTEGER NOT NULL DEFAULT 0 CHECK(cpp_exempt IN (0,1)),  -- Whether employee is exempt from CPP deductions
    dental_benefit INTEGER NOT NULL DEFAULT 1 CHECK(dental_benefit IN (1,2,3)),  -- Employer-offered dental benefit (T4 Box 45): 1=No, 2=Basic, 3=Comprehensive
    created_at TEXT NOT NULL
);

-- Indexes for employee
CREATE INDEX IF NOT EXISTS idx_employee_active ON employee(is_active);
CREATE INDEX IF NOT EXISTS idx_employee_names ON employee(first_name, last_name);
CREATE INDEX IF NOT EXISTS idx_employee_hire_date ON employee(hire_date);
CREATE INDEX IF NOT EXISTS idx_employee_termination_date ON employee(termination_date);

-- Personal amount indexing
CREATE TABLE IF NOT EXISTS personal_amount (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    province TEXT NOT NULL CHECK(province IN ('ON','QC','BC','AB','SK','MB','NS','NB','PE','NL','NT','YT','NU')),
    year INTEGER NOT NULL,
    federal_amount INTEGER NOT NULL,  -- Store as cents
    provincial_amount INTEGER NOT NULL,  -- Store as cents
    indexed_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE,
    UNIQUE(employee_id, province, year)
);

-- Pay rate log: Tracks pay rate changes
CREATE TABLE IF NOT EXISTS pay_rate_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    pay_rate INTEGER NOT NULL,  -- Store as cents
    pay_type TEXT NOT NULL,
    effective_date TEXT NOT NULL,
    end_date TEXT,
    reason TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_pay_rate_employee ON pay_rate_log(employee_id, effective_date);
CREATE INDEX IF NOT EXISTS idx_pay_rate_effective ON pay_rate_log(effective_date);

-- Employment log: Hire/term events
CREATE TABLE IF NOT EXISTS employment_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    hire_date TEXT NOT NULL,
    termination_date TEXT,
    termination_reason TEXT,
    rehire_eligible INTEGER NOT NULL DEFAULT 1,
    notes TEXT,  -- Employment event notes: termination reasons details, rehire conditions, special circumstances, feedback
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_employment_employee ON employment_log(employee_id, hire_date);
CREATE INDEX IF NOT EXISTS idx_employment_dates ON employment_log(hire_date, termination_date);


-- Payroll tables
CREATE TABLE IF NOT EXISTS payroll (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL UNIQUE,
    pay_period_start_date TEXT NOT NULL,
    pay_period_end_date TEXT NOT NULL,
    pay_date TEXT NOT NULL,
    pay_period_number INTEGER NOT NULL DEFAULT 0,
    total_pay_periods INTEGER NOT NULL DEFAULT 26,
    regular_hours INTEGER,  -- Store as thousandths of hours (e.g., 40.5 = 40500)
    overtime_hours INTEGER DEFAULT 0,  -- Store as thousandths of hours
    gross_pay INTEGER NOT NULL,  -- Store as cents
    additional_earnings INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    insured_earning INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    additional_tax_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    cpp_deduction INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    cpp2_deduction INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    ei_deduction INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    federal_tax INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    provincial_tax INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    additional_deductions INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    net_pay INTEGER NOT NULL,  -- Store as cents
    federal_personal_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    provincial_personal_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    province TEXT NOT NULL DEFAULT 'ON' CHECK(province IN ('ON','QC','BC','AB','SK','MB','NS','NB','PE','NL','NT','YT','NU')),
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);

-- Additional earnings per payroll
CREATE TABLE IF NOT EXISTS payroll_earning (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payroll_id INTEGER NOT NULL,
    earning_type TEXT NOT NULL,  -- vacation, commission, allowance, bonus, holiday, benefit, etc.
    amount INTEGER NOT NULL,  -- Store as cents
    hours INTEGER,  -- optional hours for this earning type (store as thousandths)
    is_periodic INTEGER NOT NULL DEFAULT 1 CHECK(is_periodic IN (0,1)),  -- 1=periodic (I), 0=non-periodic (B/I1)
    created_at TEXT NOT NULL,
    FOREIGN KEY (payroll_id) REFERENCES payroll(id) ON DELETE CASCADE,
    UNIQUE(payroll_id, earning_type)
);

-- Additional deductions per payroll
CREATE TABLE IF NOT EXISTS payroll_deduction (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payroll_id INTEGER NOT NULL,
    deduction_type TEXT NOT NULL,
    amount INTEGER NOT NULL,  -- Store as cents
    created_at TEXT NOT NULL,
    FOREIGN KEY (payroll_id) REFERENCES payroll(id) ON DELETE CASCADE,
    UNIQUE(payroll_id, deduction_type)
);


-- History records
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    pay_period_start_date TEXT NOT NULL,
    pay_period_end_date TEXT NOT NULL,
    pay_date TEXT NOT NULL,
    pay_period_number INTEGER NOT NULL DEFAULT 0,
    total_pay_periods INTEGER NOT NULL DEFAULT 26,
    regular_hours INTEGER,  -- Store as thousandths of hours (e.g., 40.5 = 40500)
    overtime_hours INTEGER DEFAULT 0,  -- Store as thousandths of hours
    gross_pay INTEGER NOT NULL,  -- Store as cents
    additional_earnings INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    insured_earning INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    additional_tax_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    cpp_deduction INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    cpp2_deduction INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    ei_deduction INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    federal_tax INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    provincial_tax INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    additional_deductions INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    net_pay INTEGER NOT NULL,  -- Store as cents
    federal_personal_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    provincial_personal_amount INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    province TEXT NOT NULL DEFAULT 'ON' CHECK(province IN ('ON','QC','BC','AB','SK','MB','NS','NB','PE','NL','NT','YT','NU')),
    remittance_id INTEGER REFERENCES remittance(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_history_employee_pay_date ON history(employee_id, pay_date);
CREATE INDEX IF NOT EXISTS idx_history_dates ON history(pay_period_start_date, pay_period_end_date);
CREATE INDEX IF NOT EXISTS idx_history_remittance_id ON history(remittance_id);

-- Additional earnings per history
CREATE TABLE IF NOT EXISTS history_earning (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payroll_id INTEGER NOT NULL,
    earning_type TEXT NOT NULL,  -- vacation, commission, allowance, bonus, holiday, benefit, etc.
    amount INTEGER NOT NULL,  -- Store as cents
    hours INTEGER,  -- optional hours for this earning type (store as thousandths)
    is_periodic INTEGER NOT NULL DEFAULT 1 CHECK(is_periodic IN (0,1)),  -- 1=periodic (I), 0=non-periodic (B/I1)
    created_at TEXT NOT NULL,
    FOREIGN KEY (payroll_id) REFERENCES history(id) ON DELETE CASCADE,
    UNIQUE(payroll_id, earning_type)
);

-- Additional deductions per history
CREATE TABLE IF NOT EXISTS history_deduction (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payroll_id INTEGER NOT NULL,
    deduction_type TEXT NOT NULL,
    amount INTEGER NOT NULL,  -- Store as cents
    created_at TEXT NOT NULL,
    FOREIGN KEY (payroll_id) REFERENCES history(id) ON DELETE CASCADE,
    UNIQUE(payroll_id, deduction_type)
);


-- Vacation accrual tracking (transaction history)
CREATE TABLE IF NOT EXISTS vacation_accrual (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    accrual_date TEXT NOT NULL,
    payroll_id INTEGER REFERENCES payroll(id) ON DELETE SET NULL,  -- Original payroll that triggered this
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('accrue', 'payout', 'adjust', 'timeoff')),
    -- accrue: automatic accrual from payroll
    -- payout: vacation paid out (from payroll vacation earning)
    -- adjust: manual adjustment (correction, policy, forfeited, etc.)
    -- timeoff: unpaid time off taken (reduces balance without payout)
    amount INTEGER NOT NULL,  -- Store as cents (+ to add, - to deduct)
    amount_days INTEGER NOT NULL DEFAULT 0,  -- Store as thousandths of days (+ to add, - to deduct)
    balance_after INTEGER NOT NULL,  -- Running dollar balance after this transaction
    balance_after_days INTEGER NOT NULL DEFAULT 0,  -- Running day balance after this transaction
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_vacation_employee ON vacation_accrual(employee_id);
CREATE INDEX IF NOT EXISTS idx_vacation_accrual_date ON vacation_accrual(accrual_date);
CREATE INDEX IF NOT EXISTS idx_vacation_accrual_payroll ON vacation_accrual(payroll_id);

-- Vacation time off tracking
CREATE TABLE IF NOT EXISTS vacation_time_off (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    vacation_accrual_id INTEGER REFERENCES vacation_accrual(id) ON DELETE SET NULL,  -- Links to accrual transaction if paid
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    pay_type TEXT NOT NULL CHECK(pay_type IN ('paid', 'unpaid')),
    estimated_payout INTEGER NOT NULL DEFAULT 0,  -- Store as cents; readonly estimate
    payout_amount INTEGER NOT NULL DEFAULT 0,          -- Store as cents; editable; for unpaid = 0
    days_taken INTEGER NOT NULL DEFAULT 0,  -- Store as thousandths of days (weekdays consumed, for non-hourly)
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_vacation_time_off_employee ON vacation_time_off(employee_id);
CREATE INDEX IF NOT EXISTS idx_vacation_time_off_dates ON vacation_time_off(start_date, end_date);


-- Remittance summaries
CREATE TABLE IF NOT EXISTS remittance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period_start_date TEXT NOT NULL,
    period_end_date TEXT NOT NULL,
    total_employees INTEGER NOT NULL DEFAULT 0,  -- Number of employees
    total_earnings INTEGER NOT NULL DEFAULT 0,  -- Gross earnings as cents
    total_cpp INTEGER NOT NULL,  -- Store as cents
    total_cpp2 INTEGER NOT NULL DEFAULT 0,  -- CPP2 (second ceiling) as cents
    total_ei INTEGER NOT NULL,  -- Store as cents
    total_federal_tax INTEGER NOT NULL,  -- Store as cents
    total_provincial_tax INTEGER NOT NULL,  -- Store as cents
    grand_total INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    cra_report_reference TEXT,
    generated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_remittance_dates ON remittance(period_start_date, period_end_date);


-- ============================================================================
-- T4 Flexible Schema (New Design)
-- ============================================================================
--
-- Design: Each T4 box value consists of a calculated value and an adjustment field.
-- This allows adding/removing box values without structural modifications.
-- Similar to payroll_earning pattern with key-value storage.
--
-- Status flow: draft -> calculated -> filed -> locked
-- - draft: Initial state, can add adjustments
-- - calculated: Values computed from payroll history, can add adjustments
-- - filed: Submitted to CRA, cannot be modified unless unlocked
-- - locked: Archived, read-only, requires unlock to amend
--
-- For amendments: Create new slip_version with incremented version number.
-- Previous version remains locked for audit trail.
-- ============================================================================

-- T4 slip (replaces t4_calculated + t4_adjustments)
CREATE TABLE IF NOT EXISTS t4_slip (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    year INTEGER NOT NULL,
    slip_version INTEGER NOT NULL DEFAULT 1,  -- Version for recalculations
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'calculated', 'filed', 'locked')),
    net_pay INTEGER NOT NULL DEFAULT 0,  -- Sum of net_pay from payroll history (ground truth, in cents)
    filed_at TEXT,  -- When the T4 was filed
    filed_by TEXT,  -- Who filed it
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE,
    UNIQUE(employee_id, year, slip_version)
);
CREATE INDEX IF NOT EXISTS idx_t4_slip_employee_year ON t4_slip(employee_id, year);
CREATE INDEX IF NOT EXISTS idx_t4_slip_status ON t4_slip(status);

-- T4 box value (flexible key-value storage for box values)
-- Each box has calculated_value + adjustment_value
CREATE TABLE IF NOT EXISTS t4_box_value (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    t4_slip_id INTEGER NOT NULL,
    box_type TEXT NOT NULL,  -- "box_14", "box_16", "box_24", etc. (validated in code)
    calculated_value INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    adjustment_value INTEGER NOT NULL DEFAULT 0,  -- Store as cents
    FOREIGN KEY (t4_slip_id) REFERENCES t4_slip(id) ON DELETE CASCADE,
    UNIQUE(t4_slip_id, box_type)
);
CREATE INDEX IF NOT EXISTS idx_t4_box_value_slip ON t4_box_value(t4_slip_id);

-- Employee autofill values for additional earnings and deductions
-- These are default values that auto-populate when creating payroll
CREATE TABLE IF NOT EXISTS employee_autofill (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    autofill_type TEXT NOT NULL CHECK(autofill_type IN ('earning', 'deduction')),
    type_name TEXT NOT NULL,  -- earning: bonus, commission, allowance, benefit, holiday, vacation, overtime, other
                              -- deduction: rpp, union_dues, charitable, garnishment, other
    amount INTEGER NOT NULL,  -- Store as cents
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
    created_at TEXT NOT NULL,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE,
    UNIQUE(employee_id, autofill_type, type_name)
);
CREATE INDEX IF NOT EXISTS idx_employee_autofill_employee ON employee_autofill(employee_id);
CREATE INDEX IF NOT EXISTS idx_employee_autofill_active ON employee_autofill(employee_id, is_active);

