use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use csv::WriterBuilder;
use serde::Serialize;

use crate::models::{Customer, CreateCustomer};
use crate::AppState;

#[derive(Serialize)]
struct CsvCustomer {
    name: String,
    email: String,
    phone: String,
    website: String,
    city: String,
    state: String,
    industry: String,
    status: String,
}

pub async fn export(State(state): State<AppState>) -> impl IntoResponse {
    let customers = match Customer::all(&state.pool).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let csv_customers: Vec<CsvCustomer> = customers
        .into_iter()
        .map(|c| CsvCustomer {
            name: c.name,
            email: c.email.unwrap_or_default(),
            phone: c.phone.unwrap_or_default(),
            website: c.website.unwrap_or_default(),
            city: c.city.unwrap_or_default(),
            state: c.state.unwrap_or_default(),
            industry: c.industry.unwrap_or_default(),
            status: c.status,
        })
        .collect();

    let mut wtr = WriterBuilder::new().from_writer(vec![]);
    for c in &csv_customers {
        let _ = wtr.serialize(c);
    }
    let data = match wtr.into_inner() {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/csv".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"customers.csv\"".parse().unwrap(),
    );

    (headers, Body::from(data)).into_response()
}

pub async fn import(
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    let pool = &state.pool;
    let mut csv_content: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("csv_file") {
            let filename = field.file_name().map(|s| s.to_string()).unwrap_or_default();
            if !filename.ends_with(".csv") {
                let mut headers = HeaderMap::new();
                headers.insert("Location", "/customers/import".parse().unwrap());
                return (StatusCode::SEE_OTHER, headers, "Invalid file type. Please upload a CSV file.".to_string()).into_response();
            }
            csv_content = Some(field.text().await.unwrap_or_default());
            break;
        }
    }

    let csv_content = match csv_content {
        Some(c) => c,
        None => {
            let mut headers = HeaderMap::new();
            headers.insert("Location", "/customers/import".parse().unwrap());
            return (StatusCode::SEE_OTHER, headers, "No file uploaded".to_string()).into_response();
        }
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content.as_bytes());

    let headers = match reader.headers() {
        Ok(h) => h.clone(),
        Err(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("Location", "/customers/import".parse().unwrap());
            return (StatusCode::SEE_OTHER, headers, "Invalid CSV format".to_string()).into_response();
        }
    };

    let header_map: std::collections::HashMap<_, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.to_lowercase(), i))
        .collect();

    let required = ["name", "phone", "city", "state"];
    for req in &required {
        if !header_map.contains_key(*req) {
            let mut headers = HeaderMap::new();
            headers.insert("Location", "/customers/import".parse().unwrap());
            return (StatusCode::SEE_OTHER, headers, format!("Missing required column: {}", req)).into_response();
        }
    }

    let name_idx = header_map["name"];
    let phone_idx = header_map["phone"];
    let city_idx = header_map["city"];
    let state_idx = header_map["state"];
    let email_idx = header_map.get("email").copied();
    let website_idx = header_map.get("website").copied();
    let industry_idx = header_map.get("industry").copied();
    let status_idx = header_map.get("status").copied();

    let mut imported = 0;
    let mut errors: Vec<String> = Vec::new();

    for (row_num, result) in reader.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(_) => {
                errors.push(format!("Row {}: Invalid format", row_num + 2));
                continue;
            }
        };

        let name = record.get(name_idx).unwrap_or("").trim();
        let phone = record.get(phone_idx).unwrap_or("").trim();
        let city = record.get(city_idx).unwrap_or("").trim();
        let state_field = record.get(state_idx).unwrap_or("").trim();

        if name.is_empty() || phone.is_empty() || city.is_empty() || state_field.is_empty() {
            errors.push(format!("Row {}: Missing required fields", row_num + 2));
            continue;
        }

        let email: Option<String> = email_idx.and_then(|i| {
            record.get(i).map(|s| {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }).flatten()
        });
        let website: Option<String> = website_idx.and_then(|i| {
            record.get(i).map(|s| {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }).flatten()
        });
        let industry: Option<String> = industry_idx.and_then(|i| {
            record.get(i).map(|s| {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }).flatten()
        });
        let status_val: String = status_idx.and_then(|i| {
            record.get(i).map(|s| s.trim().to_string())
        }).unwrap_or_else(|| "new".to_string());

        let status = if ["new", "contacted", "callback", "follow_up"].contains(&status_val.as_str()) {
            status_val
        } else {
            "new".to_string()
        };

        let needs_review;
        let review_reason;
        if let Some(ref email_val) = email {
            if !email_val.is_empty() {
                if let Ok(Some(_)) = Customer::find_by_email(&pool, email_val).await {
                    needs_review = true;
                    review_reason = Some(format!("Duplicate email: {}", email_val));
                } else {
                    needs_review = false;
                    review_reason = None;
                }
            } else {
                needs_review = false;
                review_reason = None;
            }
        } else {
            needs_review = false;
            review_reason = None;
        }

        let create_data = CreateCustomer {
            name: name.to_string(),
            email,
            phone: Some(phone.to_string()),
            website,
            city: Some(city.to_string()),
            state: Some(state_field.to_string()),
            industry,
        };

        match Customer::create(&pool, &create_data).await {
            Ok(mut customer) => {
                if needs_review {
                    if let Ok(updated) = customer.update(&pool, &crate::models::UpdateCustomer {
                        status: Some(status),
                        ..Default::default()
                    }).await {
                        customer = updated;
                    }
                }
                imported += 1;
            }
            Err(e) => {
                errors.push(format!("Row {}: {}", row_num + 2, e));
            }
        }
    }

    let message = if imported > 0 {
        format!("Successfully imported {} customer(s).", imported)
    } else {
        "No customers were imported.".to_string()
    };

    let mut headers = HeaderMap::new();
    headers.insert("Location", "/customers".parse().unwrap());

    (StatusCode::SEE_OTHER, headers, message).into_response()
}

trait FilterEmpty {
    fn filter_empty(self) -> Option<String>;
}

impl FilterEmpty for String {
    fn filter_empty(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}
