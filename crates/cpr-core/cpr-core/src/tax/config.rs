#![allow(dead_code)]

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;

use crate::models::Province;
use crate::tax::{
    TaxYearConfig, CppConfig, Cpp2Config, EiConfig, FederalTaxConfig, ProvincialTaxConfig,
    ProvinceTaxRates, TaxBracket, Surtax, SurtaxTier,
};

/// GitHub repository configuration for remote config updates
/// 
/// For APP UPDATES: Use Tauri's built-in updater plugin instead of this module.
/// See https://tauri.app/docs/distrib/updater for Tauri app self-update setup.
/// 
/// For CONFIG UPDATES: This module handles downloading new tax config files
/// from GitHub Releases and storing them in the user's config directory.
const GITHUB_OWNER: &str = "your-github-username";
const GITHUB_REPO: &str = "canadian-payroll-system";
const GITHUB_CONFIG_PATH: &str = "config";

/// Tauri app update configuration (for app self-updates)
/// This is separate from config updates - Tauri handles app binary updates
#[cfg(feature = "app-updater")]
pub mod app_updater {
    use serde::{Deserialize, Serialize};
    
    /// Response from GitHub releases API for app updates
    #[derive(Debug, Deserialize, Serialize)]
    pub struct AppRelease {
        pub tag_name: String,
        pub name: Option<String>,
        pub body: Option<String>,
        pub html_url: String,
        pub assets: Vec<AppAsset>,
    }
    
    #[derive(Debug, Deserialize, Serialize)]
    pub struct AppAsset {
        pub name: String,
        pub browser_download_url: String,
        pub size: u64,
    }
    
    /// Check for app updates by comparing current version with latest GitHub release
    pub async fn check_app_update(
        current_version: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Option<AppRelease>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);
        
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "CanadianPayrollSystem")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(None);
        }
        
        let release: AppRelease = response.json().await?;
        
        // Compare versions (simple comparison - for production use semver crate)
        let latest_version = release.tag_name.trim_start_matches('v');
        if latest_version != current_version {
            return Ok(Some(release));
        }
        
        Ok(None)
    }
}

/// Resolve the best config path, checking in order:
/// 1. User config directory (writable, for downloaded updates)
/// 2. Bundled resources (read-only, defaults)
/// 3. Dev paths (for development)
fn resolve_config_path(year: i32) -> PathBuf {
    let filename = format!("tax_rates_{}.json", year);
    
    // 1. User config directory (highest priority - allows updates)
    if let Some(user_dir) = get_user_config_dir() {
        let user_path = user_dir.join(&filename);
        if user_path.exists() {
            return user_path;
        }
    }
    
    // 2. Bundled resources (default configs)
    let bundled_paths = [
        PathBuf::from("config").join(&filename),
        PathBuf::from("../config").join(&filename),
        PathBuf::from("../../config").join(&filename),
    ];
    
    for path in &bundled_paths {
        if path.exists() {
            return path.clone();
        }
    }
    
    // 3. Fallback to first bundled path (will fail with proper error later)
    bundled_paths[0].clone()
}

/// Get the user config directory for storing downloaded configs
fn get_user_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA").ok();
    
    #[cfg(target_os = "macos")]
    let base = std::env::var("HOME")
        .ok()
        .map(|h| format!("{}/Library/Application Support", h));
    
    #[cfg(target_os = "linux")]
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.config", h)));
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let base = None;
    
    base.map(|b| PathBuf::from(b).join("CanadianPayrollSystem"))
}

/// Load tax configuration for a given year from JSON file
/// Checks user config first, then bundled defaults
pub fn load_tax_config(year: i32) -> Result<TaxYearConfig, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config_path = resolve_config_path(year);
    load_config_from_json(&config_path.to_string_lossy())
}

/// Check if a newer config is available on GitHub Releases
/// Returns the version/year if an update is available
pub async fn check_github_update(year: i32) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "CanadianPayrollSystem")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Ok(None);
    }
    
    #[derive(Deserialize)]
    struct GitHubRelease {
        tag_name: Option<String>,
        assets: Vec<GitHubAsset>,
    }
    
    #[derive(Deserialize)]
    struct GitHubAsset {
        name: String,
        browser_download_url: String,
    }
    
    let release: GitHubRelease = response.json().await?;
    
    // Parse tag to extract year (e.g., "config-2026" -> 2026)
    if let Some(tag) = release.tag_name {
        if let Some(tag_year) = tag.strip_prefix("config-") {
            if let Ok(tag_year) = tag_year.parse::<i32>() {
                if tag_year > year {
                    // Find the matching config asset
                    let filename = format!("tax_rates_{}.json", tag_year);
                    for asset in release.assets {
                        if asset.name == filename {
                            return Ok(Some(tag_year));
                        }
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Download and save updated config from GitHub Release
pub async fn download_github_config(year: i32) -> Result<TaxYearConfig, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "CanadianPayrollSystem")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err("Failed to fetch GitHub release".into());
    }
    
    #[derive(Deserialize)]
    struct GitHubRelease {
        assets: Vec<GitHubAsset>,
    }
    
    #[derive(Deserialize)]
    struct GitHubAsset {
        name: String,
        browser_download_url: String,
    }
    
    let release: GitHubRelease = response.json().await?;
    let filename = format!("tax_rates_{}.json", year);
    
    let download_url = release.assets.iter()
        .find(|a| a.name == filename)
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| format!("Config file {} not found in release", filename))?;
    
    // Download the config file
    let config_response = client
        .get(&download_url)
        .header("User-Agent", "CanadianPayrollSystem")
        .send()
        .await?;
    
    if !config_response.status().is_success() {
        return Err("Failed to download config file".into());
    }
    
    let config_text = config_response.text().await?;
    
    // Save to user config directory
    if let Some(user_dir) = get_user_config_dir() {
        fs::create_dir_all(&user_dir)?;
        let config_path = user_dir.join(&filename);
        fs::write(&config_path, &config_text)?;
    }
    
    // Parse and return the config
    let json_config: JsonTaxConfig = serde_json::from_str(&config_text)?;
    Ok(json_config.into_tax_year_config())
}

/// Update config if a newer version is available on GitHub
/// Returns Some(year) if updated, None if already current
pub async fn update_if_available(year: i32) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(newer_year) = check_github_update(year).await? {
        download_github_config(newer_year).await?;
        return Ok(Some(newer_year));
    }
    Ok(None)
}

/// Load tax configuration with a custom base path for the config directory
fn load_tax_config_with_base(year: i32, base_path: &str) -> Result<TaxYearConfig, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let config_path = format!("{}/tax_rates_{}.json", base_path, year);
    load_config_from_json(&config_path)
}

/// Load tax configuration from JSON file
fn load_config_from_json(path: &str) -> Result<TaxYearConfig, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let json_str = fs::read_to_string(path)?;
    let json_config: JsonTaxConfig = serde_json::from_str(&json_str)?;
    Ok(json_config.into_tax_year_config())
}

// JSON deserialization structures
#[derive(Debug, Deserialize)]
struct JsonTaxConfig {
    year: i32,
    cpp: JsonCppConfig,
    cpp2: JsonCpp2Config,
    ei: JsonEiConfig,
    ei_qc: JsonEiConfig,
    qpip: JsonQpipConfig,
    federal: JsonFederalConfig,
    provincial: HashMap<String, JsonProvincialConfig>,
}

#[derive(Debug, Deserialize)]
struct JsonCppConfig {
    base_rate: f64,
    first_additional_rate: f64,
    employee_rate: f64,
    basic_exemption: f64,
    #[serde(alias = "ympe")]
    maximum_pensionable_earnings: f64,
    maximum_contribution: f64,
    maximum_base_contribution: f64,
}

#[derive(Debug, Deserialize)]
struct JsonEiConfig {
    employee_rate: f64,
    maximum_insurable_earnings: f64,
    #[serde(alias = "maximum_contribution", alias = "maximum_employee_premium")]
    maximum_employee_premium: f64,
}

#[derive(Debug, Deserialize)]
struct JsonQpipConfig {
    employee_rate: f64,
    maximum_insurable_earnings: f64,
    maximum_employee_premium: f64,
}

#[derive(Debug, Deserialize)]
struct JsonFederalConfig {
    basic_personal_amount: f64,
    canada_employment_amount: f64,
    brackets: Vec<JsonTaxBracket>,
}

#[derive(Debug, Deserialize)]
struct JsonTaxBracket {
    limit: Option<f64>,
    rate: f64,
    #[serde(default)]
    constant: f64,
}

#[derive(Debug, Deserialize)]
struct JsonProvincialConfig {
    name: String,
    basic_personal_amount: f64,
    #[serde(default)]
    canada_employment_amount: Option<f64>,
    brackets: Vec<JsonTaxBracket>,
    #[serde(default)]
    surtax_thresholds: Vec<JsonSurtax>,
    #[serde(default)]
    qpip_rate: Option<f64>,
    /// Alberta-specific K5P tax reduction threshold (only for AB)
    #[serde(default)]
    k5p_threshold: Option<f64>,
    /// Alberta-specific K5P tax reduction rate (only for AB)
    #[serde(default)]
    k5p_rate: Option<f64>,
    /// S2 amount for provincial tax reduction (2026 - ON=300, BC=575)
    #[serde(default)]
    s2_amount: Option<f64>,
    /// Index rate for personal amounts (year-over-year increase)
    #[serde(default)]
    index_rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct JsonSurtax {
    threshold: f64,
    rate: f64,
}

#[derive(Debug, Deserialize)]
struct JsonCpp2Config {
    rate: f64,
    max_earnings: f64,
    max_contribution: f64,
}

impl JsonTaxConfig {
    fn into_tax_year_config(self) -> TaxYearConfig {
        let provincial = Self::provincial_into_config(self.provincial);
        
        TaxYearConfig {
            year: self.year,
            cpp: self.cpp.into_cpp_config(),
            cpp2: self.cpp2.into_cpp2_config(),
            ei: self.ei.into_ei_config(&self.ei_qc),
            federal: self.federal.into_federal_config(),
            provincial,
        }
    }
    
    fn provincial_into_config(json_provincial: HashMap<String, JsonProvincialConfig>) -> ProvincialTaxConfig {
        let mut province_configs = HashMap::new();
        
        for (province_code, config) in json_provincial {
            // Match province code string to Province enum
            let province = match province_code.as_str() {
                "ON" => Some(Province::ON),
                "QC" => Some(Province::QC),
                "BC" => Some(Province::BC),
                "AB" => Some(Province::AB),
                "SK" => Some(Province::SK),
                "MB" => Some(Province::MB),
                "NB" => Some(Province::NB),
                "NS" => Some(Province::NS),
                "PE" => Some(Province::PE),
                "NL" => Some(Province::NL),
                "YT" => Some(Province::YT),
                "NT" => Some(Province::NT),
                "NU" => Some(Province::NU),
                _ => None,
            };
            
            if let Some(prov) = province {
                province_configs.insert(prov, config.into_province_rates());
            }
        }
        
        ProvincialTaxConfig { province_configs }
    }
}

impl JsonCpp2Config {
    fn into_cpp2_config(self) -> Cpp2Config {
        Cpp2Config {
            rate: Decimal::from_f64_retain(self.rate).unwrap(),
            max_earnings: Decimal::from_f64_retain(self.max_earnings).unwrap(),
            max_contribution: Decimal::from_f64_retain(self.max_contribution).unwrap(),
        }
    }
}

impl JsonCppConfig {
    fn into_cpp_config(self) -> CppConfig {
        CppConfig {
            basic_exemption: Decimal::from_f64_retain(self.basic_exemption).unwrap(),
            max_pensionable_earnings: Decimal::from_f64_retain(self.maximum_pensionable_earnings).unwrap(),
            base_rate: Decimal::from_f64_retain(self.base_rate).unwrap(),
            first_additional_rate: Decimal::from_f64_retain(self.first_additional_rate).unwrap(),
            employee_rate: Decimal::from_f64_retain(self.employee_rate).unwrap(),
            max_contribution: Decimal::from_f64_retain(self.maximum_contribution).unwrap(),
            max_base_contribution: Decimal::from_f64_retain(self.maximum_base_contribution).unwrap(),
        }
    }
}

impl JsonEiConfig {
    fn into_ei_config(self, qc_config: &JsonEiConfig) -> EiConfig {
        EiConfig {
            max_insurable_earnings: Decimal::from_f64_retain(self.maximum_insurable_earnings).unwrap(),
            employee_rate: Decimal::from_f64_retain(self.employee_rate).unwrap(),
            qc_employee_rate: Decimal::from_f64_retain(qc_config.employee_rate).unwrap(),
            max_contribution: Decimal::from_f64_retain(self.maximum_employee_premium).unwrap(),
            qc_max_contribution: Decimal::from_f64_retain(qc_config.maximum_employee_premium).unwrap(),
        }
    }
}

impl JsonFederalConfig {
    fn into_federal_config(self) -> FederalTaxConfig {
        let mut brackets = Vec::new();
        let mut lower_limit = Decimal::ZERO;
        
        for bracket in self.brackets {
            let upper_limit = bracket.limit.map(|l| Decimal::from_f64_retain(l).unwrap());
            let rate = Decimal::from_f64_retain(bracket.rate).unwrap();
            let constant = Decimal::from_f64_retain(bracket.constant).unwrap();
            
            brackets.push(TaxBracket {
                lower_limit,
                upper_limit,
                rate,
                constant,
            });
            
            if let Some(limit) = upper_limit {
                lower_limit = limit;
            }
        }
        
        FederalTaxConfig {
            basic_personal_amount: Decimal::from_f64_retain(self.basic_personal_amount).unwrap(),
            canada_employment_amount: Decimal::from_f64_retain(self.canada_employment_amount).unwrap(),
            brackets,
        }
    }
}

impl JsonProvincialConfig {
    fn into_province_rates(self) -> ProvinceTaxRates {
        let mut brackets = Vec::new();
        let mut lower_limit = Decimal::ZERO;
        
        for bracket in self.brackets {
            let upper_limit = bracket.limit.map(|l| Decimal::from_f64_retain(l).unwrap());
            let rate = Decimal::from_f64_retain(bracket.rate).unwrap();
            let constant = Decimal::from_f64_retain(bracket.constant).unwrap();
            
            brackets.push(TaxBracket {
                lower_limit,
                upper_limit,
                rate,
                constant,
            });
            
            if let Some(limit) = upper_limit {
                lower_limit = limit;
            }
        }
        
        // Handle surtax - take the first threshold for backwards compatibility
        let surtax = self.surtax_thresholds.first().map(|s| Surtax {
            threshold: Decimal::from_f64_retain(s.threshold).unwrap(),
            rate: Decimal::from_f64_retain(s.rate).unwrap(),
        });
        
        // Load all surtax tiers for multi-tier surtax (e.g., Ontario)
        let surtax_tiers: Vec<SurtaxTier> = self.surtax_thresholds.iter().map(|s| SurtaxTier {
            threshold: Decimal::from_f64_retain(s.threshold).unwrap(),
            rate: Decimal::from_f64_retain(s.rate).unwrap(),
        }).collect();
        
        ProvinceTaxRates {
            basic_personal_amount: Decimal::from_f64_retain(self.basic_personal_amount).unwrap(),
            canada_employment_amount: self.canada_employment_amount
                .map(|v| Decimal::from_f64_retain(v).unwrap())
                .unwrap_or(Decimal::ZERO),
            brackets,
            surtax,
            surtax_tiers,
            k5p_threshold: self.k5p_threshold.map(|v| Decimal::from_f64_retain(v).unwrap()),
            k5p_rate: self.k5p_rate.map(|v| Decimal::from_f64_retain(v).unwrap()),
            s2_amount: self.s2_amount
                .map(|v| Decimal::from_f64_retain(v).unwrap())
                .unwrap_or(Decimal::ZERO),
        }
    }
}
