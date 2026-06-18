# digiCaPr - Digi Canadian Payroll App

A comprehensive payroll management system for Canadian businesses, built with Rust and Tauri for cross-platform desktop deployment.

## 🚀 Features

- **Tax Calculations**: Accurate Canadian tax calculations following CRA T4127 formulas
- **Multi-Province Support**: Handles federal and provincial tax rates for all Canadian provinces
- **Database Management**: Secure SQLite database with optional password protection
- **Payroll Processing**: Full payroll cycle management including calculations, remittances, and reporting
- **Reporting**: Generate T4 tax forms and remittance reports
- **Developer Tools**: Advanced developer mode for direct database manipulation

## 📋 System Requirements

- **Operating System**: Windows 11 or later
- **Rust**: Stable version (1.70+)
- **Node.js**: 18.0 or higher
- **SQLite**: 3.0 or higher

## 🛠️ Installation

### Prerequisites

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Node.js from [nodejs.org](https://nodejs.org/)

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/cpr.git
   cd cpr
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Build the project:
   ```bash
   npm run build
   ```

4. Run the application:
   ```bash
   npm run dev
   ```

## 🔧 Configuration

### Database Setup

The CPR system uses SQLite for data storage. Database files can be password-protected for security.

### Tax Rates

Tax rates are loaded from JSON configuration files in the `config/` directory. The system supports:
- Federal tax rates
- Provincial tax rates for all Canadian provinces
- CPP (Canada Pension Plan) rates
- EI (Employment Insurance) rates

## 📊 Usage

### Getting Started

1. **Create a Company**: Set up your company information in the settings
2. **Add Employees**: Enter employee details and tax information
3. **Process Payroll**: Run payroll calculations for each pay period
4. **Generate Reports**: Create tax forms and remittance reports

### Key Features

- **Payroll Processing**: Calculate gross pay, deductions, and net pay
- **Tax Calculations**: Automatic federal and provincial tax calculations
- **Remittance Management**: Track and generate remittance reports
- **Tax Form Generation**: Create T4 slips and remittance reports
- **History Tracking**: Maintain complete payroll history

## 🔧 Development

### Project Structure

```
cpr/
├── crates/                 # Rust libraries
│   ├── cpr-core/          # Core business logic
│   ├── cpr-db/            # Database layer
│   └── cpr-reports/       # Report generation
├── src/                   # Frontend (Vue.js)
├── docs/                  # Documentation
├── config/                # Tax rate configurations
└── tests/                 # Test files
```

### Building

```bash
# Build the Rust backend
cargo build --release

# Build the frontend
npm run build

# Run tests
cargo test
npm test
```

## 📝 Documentation

- [Developer Documentation](DEVELOPMENT.md) - Architecture and implementation details
- [Tax Rate Configuration](config/README.md) - How to update tax rates
- [T4127 Formulas](docs/README.md) - CRA payroll deduction formulas

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🚨 Security

- Database files can be password-protected
- All tax calculations follow CRA guidelines
- Secure password hashing using Argon2

## 📞 Support

For issues and questions:
- Create an issue on GitHub
- Check the documentation in the `docs/` directory
- Review the test files in `tests/` for examples

---

**Note**: This software is intended for educational and development purposes. Always verify tax calculations with official CRA guidelines.
