# Tax Rate Configuration Files

This directory contains JSON configuration files for Canadian tax rates by year.

## Overview

The Digi Canadian Payroll App loads tax rates from JSON files at runtime. This allows for easy updates when CRA publishes new rates for upcoming tax years without needing to recompile the application.

## File Naming Convention

Tax rate files should be named: `tax_rates_{YEAR}.json`

Example: `tax_rates_2024.json`, `tax_rates_2025.json`

## Configuration Loading

When the application starts, it first attempts to load the JSON file for the requested year from the `config/` directory.

## JSON Schema

The configuration file must follow this structure:

```json
{
  "year": 2024,
  "effective_date": "2024-01-01",
  "cpp": {
    "employee_rate": 0.0595,
    "employer_rate": 0.0595,
    "basic_exemption": 3500.00,
    "maximum_pensionable_earnings": 68500.00,
    "maximum_contribution": 3867.50
  },
  "ei": {
    "employee_rate": 0.0166,
    "employer_rate": 0.02324,
    "maximum_insurable_earnings": 63200.00,
    "maximum_contribution": 1049.12
  },
  "federal": {
    "basic_personal_amount": 15705.00,
    "brackets": [
      {
        "limit": 55867.00,
        "rate": 0.15
      },
      {
        "limit": null,
        "rate": 0.33
      }
    ]
  },
  "provincial": {
    "ON": {
      "name": "Ontario",
      "basic_personal_amount": 11865.00,
      "brackets": [
        {
          "limit": 51446.00,
          "rate": 0.0505
        }
      ],
      "surtax_thresholds": [
        {
          "threshold": 5315.00,
          "rate": 0.20
        }
      ]
    }
  }
}
```

## Configuration Sections

### CPP (Canada Pension Plan)
- `employee_rate`: CPP contribution rate for employees
- `employer_rate`: CPP contribution rate for employers (usually matches employee rate)
- `basic_exemption`: Annual basic exemption amount
- `maximum_pensionable_earnings`: Maximum pensionable earnings for the year
- `maximum_contribution`: Maximum CPP contribution for the year

### EI (Employment Insurance)
- `employee_rate`: EI premium rate for employees
- `employer_rate`: EI premium rate for employers (1.4× employee rate)
- `maximum_insurable_earnings`: Maximum insurable earnings for the year
- `maximum_contribution`: Maximum EI premium for the year

### Federal Tax
- `basic_personal_amount`: Federal basic personal amount (non-refundable tax credit)
- `brackets`: Array of tax brackets
  - `limit`: Upper income limit for this bracket (null for top bracket)
  - `rate`: Tax rate as a decimal (e.g., 0.15 = 15%)

### Provincial Tax
Each province is identified by its two-letter code (ON, QC, BC, AB, SK, MB, NB, NS, PE, NL, YT, NT, NU).

For each province:
- `name`: Full province name (for reference only)
- `basic_personal_amount`: Provincial basic personal amount
- `brackets`: Array of tax brackets (same structure as federal)
- `surtax_thresholds`: (Optional) Array of surtax thresholds
  - `threshold`: Basic provincial tax amount at which surtax applies
  - `rate`: Surtax rate
- `qpip_rate`: (Optional, QC only) Quebec Parental Insurance Plan rate

## Updating for a New Tax Year

1. Visit the CRA website for official tax rates:
   - https://www.canada.ca/en/revenue-agency/services/forms-publications/payroll/t4127-payroll-deductions-formulas.html
   - https://www.canada.ca/en/revenue-agency/services/forms-publications/payroll/t4127-payroll-deductions-formulas/t4127-jan.html

2. Copy the previous year's JSON file:
   ```bash
   cp config/tax_rates_2024.json config/tax_rates_2025.json
   ```

3. Update the following values in the new file:
   - `year` field
   - `effective_date`
   - CPP rates and limits
   - EI rates and limits
   - Federal tax brackets and basic personal amount
   - Provincial tax brackets and basic personal amounts for all provinces

4. Validate the JSON file:
   ```bash
   cargo test --package cpr-core
   ```

5. Test the application with the new configuration

## Testing

After creating or updating a configuration file, run the test suite to ensure it loads correctly:

```bash
cargo test --package cpr-core -- --nocapture
```

## Notes

- All rates should be expressed as decimals (e.g., 5.95% = 0.0595)
- All monetary amounts should be in Canadian dollars
- Tax brackets array should be ordered from lowest to highest income
- The last bracket should have `"limit": null` to indicate it applies to all income above the previous bracket
- Quebec has unique rates for EI due to QPIP (Quebec Parental Insurance Plan)
