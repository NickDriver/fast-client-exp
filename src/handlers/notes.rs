use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::customer::Customer;
use crate::models::customer_note::CustomerNote;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateNoteForm {
    pub body: String,
}

pub async fn create(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Form(form): Form<CreateNoteForm>,
) -> impl IntoResponse {
    let _ = Customer::find(&state.pool, customer_id).await;

    match sqlx::query_as::<_, CustomerNote>(
        "INSERT INTO customer_notes (customer_id, body) VALUES ($1, $2) RETURNING *",
    )
    .bind(customer_id)
    .bind(&form.body)
    .fetch_one(&state.pool)
    .await
    {
        Ok(_) => {
            let notes = CustomerNote::by_customer(&state.pool, customer_id)
                .await
                .unwrap_or_default();
            let mut html = String::new();
            for note in &notes {
                html.push_str(&format!(
                    r#"<div class="py-2 border-b border-gray-200">
                        <p class="text-sm text-gray-800">{}</p>
                        <p class="text-xs text-gray-500">{}</p>
                    </div>"#,
                    note.body,
                    note.created_at.format("%Y-%m-%d %H:%M")
                ));
            }
            Html(html).into_response()
        }
        Err(e) => Html(format!("<div class='p-4 bg-red-100 text-red-800'>Error: {}</div>", e)).into_response(),
    }
}
