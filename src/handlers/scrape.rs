use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ScrapeResponse {
    success: bool,
    data: Option<ScrapeData>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct ScrapeData {
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    city: Option<String>,
    state: Option<String>,
    industry: Option<String>,
}

#[derive(Deserialize)]
pub struct ScrapeRequest {
    url: String,
}

const INDUSTRY_MAP: &[(&str, &str)] = &[
    ("Restaurant", "Restaurant"),
    ("FoodEstablishment", "Restaurant"),
    ("CafeOrCoffeeShop", "Restaurant"),
    ("BarOrPub", "Restaurant"),
    ("AutoDealer", "Automotive"),
    ("AutoRepair", "Automotive"),
    ("Attorney", "Legal"),
    ("LegalService", "Legal"),
    ("MedicalBusiness", "Healthcare"),
    ("Dentist", "Healthcare"),
    ("Physician", "Healthcare"),
    ("Hospital", "Healthcare"),
    ("Pharmacy", "Healthcare"),
    ("VeterinaryCare", "Healthcare"),
    ("RealEstateAgent", "Real Estate"),
    ("InsuranceAgency", "Insurance"),
    ("FinancialService", "Finance"),
    ("AccountingService", "Finance"),
    ("EducationalOrganization", "Education"),
    ("School", "Education"),
    ("Store", "Retail"),
    ("ClothingStore", "Retail"),
    ("HardwareStore", "Retail"),
    ("ElectronicsStore", "Retail"),
    ("GroceryStore", "Retail"),
    ("HomeAndConstructionBusiness", "Construction"),
    ("Plumber", "Construction"),
    ("Electrician", "Construction"),
    ("RoofingContractor", "Construction"),
    ("HVACBusiness", "Construction"),
    ("LodgingBusiness", "Hospitality"),
    ("Hotel", "Hospitality"),
    ("SportsActivityLocation", "Fitness"),
    ("HealthAndBeautyBusiness", "Beauty"),
    ("HairSalon", "Beauty"),
    ("DaySpa", "Beauty"),
    ("TravelAgency", "Travel"),
    ("ProfessionalService", "Professional Services"),
];

const INDUSTRY_KEYWORDS: &[(&str, &str)] = &[
    ("restaurant", "Restaurant"),
    ("plumbing", "Construction"),
    ("hvac", "Construction"),
    ("roofing", "Construction"),
    ("dental", "Healthcare"),
    ("medical", "Healthcare"),
    ("clinic", "Healthcare"),
    ("law firm", "Legal"),
    ("attorney", "Legal"),
    ("real estate", "Real Estate"),
    ("realty", "Real Estate"),
    ("insurance", "Insurance"),
    ("salon", "Beauty"),
    ("spa", "Beauty"),
    ("fitness", "Fitness"),
    ("gym", "Fitness"),
    ("auto", "Automotive"),
    ("car dealer", "Automotive"),
    ("hotel", "Hospitality"),
    ("motel", "Hospitality"),
    ("accounting", "Finance"),
    ("consulting", "Professional Services"),
    ("technology", "Technology"),
    ("software", "Technology"),
    ("marketing", "Marketing"),
];

pub async fn scrape(Json(req): Json<ScrapeRequest>) -> impl IntoResponse {
    let url = req.url.trim();

    if url.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(ScrapeResponse {
            success: false,
            data: None,
            error: Some("Please enter a website URL first.".to_string()),
        })).into_response();
    }

    let mut url_to_fetch = url.to_string();
    if !url_to_fetch.starts_with("http://") && !url_to_fetch.starts_with("https://") {
        url_to_fetch = format!("https://{}", url_to_fetch);
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ScrapeResponse {
                success: false,
                data: None,
                error: Some("Failed to create HTTP client".to_string()),
            })).into_response();
        }
    };

    let response = match client.get(&url_to_fetch)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ScrapeResponse {
                success: false,
                data: None,
                error: Some(format!("Could not fetch the website: {}", e)),
            })).into_response();
        }
    };

    if !response.status().is_success() {
        return (StatusCode::BAD_REQUEST, Json(ScrapeResponse {
            success: false,
            data: None,
            error: Some(format!("Website returned status: {}", response.status())),
        })).into_response();
    }

    let html = match response.text().await {
        Ok(text) => text,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(ScrapeResponse {
                success: false,
                data: None,
                error: Some("Failed to read website content".to_string()),
            })).into_response();
        }
    };

    let mut data = ScrapeData {
        name: None,
        email: None,
        phone: None,
        city: None,
        state: None,
        industry: None,
    };

    if let Some(json_ld) = extract_json_ld(&html) {
        if let Some(name) = json_ld.get("name").and_then(|v| v.as_str()) {
            data.name = Some(name.to_string());
        }
        if let Some(email) = json_ld.get("email").and_then(|v| v.as_str()) {
            data.email = Some(email.to_string());
        }
        if let Some(tel) = json_ld.get("telephone").and_then(|v| v.as_str()) {
            data.phone = Some(tel.to_string());
        }
        if let Some(addr) = json_ld.get("address").and_then(|v| v.as_object()) {
            if let Some(locality) = addr.get("addressLocality").and_then(|v| v.as_str()) {
                data.city = Some(locality.to_string());
            }
            if let Some(region) = addr.get("addressRegion").and_then(|v| v.as_str()) {
                data.state = Some(region.to_string());
            }
        }
        if let Some(type_val) = json_ld.get("@type").and_then(|v| v.as_str()) {
            for (schema_type, industry) in INDUSTRY_MAP {
                if type_val.eq_ignore_ascii_case(schema_type) {
                    data.industry = Some(industry.to_string());
                    break;
                }
            }
        }
    }

    if data.email.is_none() {
        data.email = extract_email(&html);
    }
    if data.phone.is_none() {
        data.phone = extract_phone(&html);
    }
    if data.city.is_none() || data.state.is_none() {
        if let Some((city, state)) = extract_address(&html) {
            if data.city.is_none() {
                data.city = city;
            }
            if data.state.is_none() {
                data.state = state;
            }
        }
    }
    if data.industry.is_none() {
        data.industry = extract_industry(&html);
    }
    if data.name.is_none() {
        data.name = extract_name(&html);
    }

    (StatusCode::OK, Json(ScrapeResponse {
        success: true,
        data: Some(data),
        error: None,
    })).into_response()
}

fn extract_json_ld(html: &str) -> Option<serde_json::Value> {
    let re = regex::Regex::new(r#"<script[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>"#).ok()?;
    let org_types = ["Organization", "LocalBusiness", "Corporation", "Store", "Restaurant", 
        "MedicalBusiness", "LegalService", "FinancialService", "RealEstateAgent", "AutoDealer",
        "AutoRepair", "EducationalOrganization", "ProfessionalService", "HomeAndConstructionBusiness"];

    for cap in re.captures_iter(html) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(cap.get(1)?.as_str()) {
            if let Some(items) = json.get("@graph").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(t) = item.get("@type").and_then(|v| v.as_str()) {
                        if org_types.iter().any(|o| o.eq_ignore_ascii_case(t)) {
                            return Some(item.clone());
                        }
                    }
                }
            } else if let Some(t) = json.get("@type").and_then(|v| v.as_str()) {
                if org_types.iter().any(|o| o.eq_ignore_ascii_case(t)) {
                    return Some(json);
                }
            }
        }
    }
    None
}

fn extract_email(html: &str) -> Option<String> {
    let re = regex::Regex::new(r#"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}"#).ok()?;
    let clean = regex::Regex::new(r#"<(script|style)[^>]*>.*?</\1>"#).ok()
        .map(|r| r.replace_all(html, " ").into_owned())
        .unwrap_or_else(|| html.to_string());
    
    for cap in re.find_iter(&clean) {
        let email = cap.as_str();
        let skip_domains = ["example.com", "sentry.io", "w3.org"];
        if !skip_domains.iter().any(|d| email.contains(d)) {
            return Some(email.to_string());
        }
    }
    None
}

fn extract_phone(html: &str) -> Option<String> {
    let clean = regex::Regex::new(r#"<(script|style)[^>]*>.*?</\1>"#).ok()
        .map(|r| r.replace_all(html, " ").into_owned())
        .unwrap_or_else(|| html.to_string());
    let clean = regex::Regex::new(r#"<[^>]+>"#).ok()
        .map(|r| r.replace_all(&clean, " ").into_owned())
        .unwrap_or_else(|| clean.clone());

    let patterns = [
        r"\(?\d{3}\)?[\s.\-]?\d{3}[\s.\-]?\d{4}",
        r"1[\s.\-]?\(?\d{3}\)?[\s.\-]?\d{3}[\s.\-]?\d{4}",
        r"\+1[\s.\-]?\(?\d{3}\)?[\s.\-]?\d{3}[\s.\-]?\d{4}",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(m) = re.find(&clean) {
                return Some(m.as_str().to_string());
            }
        }
    }
    None
}

fn extract_address(html: &str) -> Option<(Option<String>, Option<String>)> {
    let clean = regex::Regex::new(r#"<(script|style)[^>]*>.*?</\1>"#).ok()
        .map(|r| r.replace_all(html, " ").into_owned())
        .unwrap_or_else(|| html.to_string());
    let clean = regex::Regex::new(r#"<[^>]+>"#).ok()
        .map(|r| r.replace_all(&clean, " ").into_owned())
        .unwrap_or_else(|| clean.clone());

    let states = "AL|AK|AZ|AR|CA|CO|CT|DE|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY|DC";
    
    let pattern = format!(r#"([A-Z][a-zA-Z\s.\-]{{1,30}}),\s*({})[\s,]+\d{{5}}"#, states);
    if let Ok(re) = regex::Regex::new(&pattern) {
        if let Some(cap) = re.captures(&clean) {
            return Some((Some(cap.get(1)?.as_str().to_string()), Some(cap.get(2)?.as_str().to_string())));
        }
    }

    let pattern2 = format!(r#"([A-Z][a-zA-Z\s.\-]{{1,30}}),\s*({})"#, states);
    if let Ok(re) = regex::Regex::new(&pattern2) {
        if let Some(cap) = re.captures(&clean) {
            return Some((Some(cap.get(1)?.as_str().to_string()), Some(cap.get(2)?.as_str().to_string())));
        }
    }

    None
}

fn extract_industry(html: &str) -> Option<String> {
    let clean = regex::Regex::new(r#"<(script|style)[^>]*>.*?</\1>"#).ok()
        .map(|r| r.replace_all(html, " ").into_owned())
        .unwrap_or_else(|| html.to_string());
    let clean = regex::Regex::new(r#"<[^>]+>"#).ok()
        .map(|r| r.replace_all(&clean, " ").into_owned())
        .unwrap_or_else(|| clean.clone());
    let clean_lower = clean.to_lowercase();

    for (keyword, industry) in INDUSTRY_KEYWORDS {
        if clean_lower.contains(keyword) {
            return Some(industry.to_string());
        }
    }
    None
}

fn extract_name(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
    let title = re.captures(html)?.get(1)?.as_str().to_string();
    let name = title.split(|c| c == '|' || c == '-' || c == '–' || c == '—')
        .next()
        .unwrap_or(&title)
        .trim()
        .to_string();
    if name.is_empty() { None } else { Some(name) }
}
