use crate::models::{Customer, CreateCustomer, UpdateCustomer};
use crate::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

#[derive(Deserialize)]
pub struct FilterQuery {
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct StatusForm {
    status: String,
}

fn status_class(status: &str) -> &'static str {
    match status {
        "new" => "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300",
        "contacted" => "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300",
        "callback" => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300",
        "follow_up" => "bg-purple-100 text-purple-800 dark:bg-purple-900/40 dark:text-purple-300",
        _ => "bg-gray-100 text-gray-800 dark:bg-gray-900/40 dark:text-gray-300",
    }
}

fn status_badge(customer: &Customer) -> String {
    let label = match customer.status.as_str() {
        "new" => "New",
        "contacted" => "Contacted",
        "callback" => "Callback",
        "follow_up" => "Follow Up",
        _ => "Unknown",
    };
    format!(
        r#"<span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}">{}</span>"#,
        status_class(&customer.status),
        label
    )
}

fn base_html(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - FastClient</title>
    <link rel="stylesheet" href="/assets/css/app.css">
    <script src="/assets/js/htmx.min.js"></script>
    <script src="/assets/js/htmx-ext-preload.js"></script>
    <script>
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {{
            document.documentElement.classList.add('dark');
        }}
    </script>
</head>
<body class="bg-gray-50 min-h-screen dark:bg-warm-950" hx-ext="preload">
    {}
    <script src="/assets/js/app.js"></script>
</body>
</html>"#,
        title, content
    )
}

fn sidebar_layout(current_path: &str, content: &str) -> String {
    let dashboard_class = if current_path == "/" || current_path == "/dashboard" {
        "sidebar-link sidebar-link-active"
    } else {
        "sidebar-link"
    };
    let customers_class = if current_path.starts_with("/customers") {
        "sidebar-link sidebar-link-active"
    } else {
        "sidebar-link"
    };
    
    format!(
        r#"<div class="min-h-full flex">
        <!-- Sidebar overlay for mobile -->
        <div id="sidebar-overlay" class="fixed inset-0 bg-gray-600 bg-opacity-75 dark:bg-warm-950 dark:bg-opacity-75 z-20 hidden lg:hidden"></div>

        <!-- Sidebar -->
        <aside id="sidebar" class="fixed inset-y-0 left-0 z-30 w-64 transform -translate-x-full transition-transform duration-300 ease-in-out lg:translate-x-0 lg:flex lg:flex-col bg-white border-r border-gray-200 dark:bg-warm-900 dark:border-warm-700">
            <!-- Logo -->
            <div class="flex h-16 shrink-0 items-center px-6 border-b border-gray-200 dark:border-warm-700">
                <a href="/" class="text-xl font-bold text-gray-900 dark:text-warm-100">
                    FastClient
                </a>
            </div>

            <!-- Navigation -->
            <nav class="flex-1 space-y-1 px-3 py-4">
                <a href="/" class="{}">
                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                    </svg>
                    Dashboard
                </a>

                <a href="/customers" class="{}">
                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
                    </svg>
                    Customers
                </a>
            </nav>

            <!-- User section -->
            <div class="border-t border-gray-200 dark:border-warm-700 p-4">
                <div class="flex items-center gap-3">
                    <div class="h-9 w-9 rounded-full bg-gray-200 dark:bg-warm-700 flex items-center justify-center">
                        <span class="text-sm font-medium text-gray-600 dark:text-warm-300">U</span>
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-sm font-medium text-gray-900 dark:text-warm-100 truncate">User</p>
                    </div>
                    <a href="/logout" class="text-gray-400 hover:text-gray-600 dark:text-warm-400 dark:hover:text-warm-200" title="Logout">
                        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15M12 9l-3 3m0 0l3 3m-3-3h12.75" />
                        </svg>
                    </a>
                </div>
            </div>
        </aside>

        <!-- Main content -->
        <div class="flex-1 lg:pl-64">
            <!-- Top bar for mobile -->
            <div class="sticky top-0 z-10 flex h-16 shrink-0 items-center gap-x-4 border-b border-gray-200 dark:border-warm-700 bg-white dark:bg-warm-900 px-4 shadow-sm lg:hidden">
                <button type="button" id="sidebar-toggle" class="-m-2.5 p-2.5 text-gray-700 dark:text-warm-300">
                    <span class="sr-only">Open sidebar</span>
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                    </svg>
                </button>
                <div class="flex-1 text-sm font-semibold text-gray-900 dark:text-warm-100">FastClient</div>
            </div>

            {}
        </div>
    </div>"#,
        dashboard_class, customers_class, content
    )
}

pub async fn index(
    State(state): State<AppState>,
    Query(search_q): Query<SearchQuery>,
    Query(filter_q): Query<FilterQuery>,
) -> impl IntoResponse {
    let customers = if let Some(ref q) = search_q.q {
        if !q.is_empty() {
            Customer::search(&state.pool, q).await.unwrap_or_default()
        } else if let Some(ref status) = filter_q.status {
            Customer::filter_by_status(&state.pool, status).await.unwrap_or_default()
        } else {
            Customer::all(&state.pool).await.unwrap_or_default()
        }
    } else if let Some(ref status) = filter_q.status {
        Customer::filter_by_status(&state.pool, status).await.unwrap_or_default()
    } else {
        Customer::all(&state.pool).await.unwrap_or_default()
    };

    let status_counts = Customer::count_by_status(&state.pool).await.unwrap_or_default();
    let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for sc in &status_counts {
        counts.insert(sc.status.clone(), sc.count);
    }
    let total = counts.values().sum::<i64>() as usize;
    let new_count = counts.get("new").copied().unwrap_or(0);
    let contacted_count = counts.get("contacted").copied().unwrap_or(0);
    let callback_count = counts.get("callback").copied().unwrap_or(0);
    let follow_up_count = counts.get("follow_up").copied().unwrap_or(0);

    let current_status = filter_q.status.as_deref().unwrap_or("");

    let mut rows = String::new();
    for c in &customers {
        rows.push_str(&format!(
            r#"<tr class="hover:bg-gray-50 dark:hover:bg-warm-800/50">
                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-warm-100">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">{}</td>
                <td class="px-6 py-4 whitespace-nowrap">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">
                    <a href="/customers/{}" class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300">View</a>
                </td>
            </tr>"#,
            c.name,
            c.email.as_deref().unwrap_or("-"),
            c.phone.as_deref().unwrap_or("-"),
            status_badge(c),
            c.created_at.format("%Y-%m-%d"),
            c.id
        ));
    }

    let search_value = search_q.q.as_deref().unwrap_or("");

    let filter_options = {
        let mut opts = String::new();
        let options = [("", "All Statuses"), ("new", "New"), ("contacted", "Contacted"), ("callback", "Callback"), ("follow_up", "Follow Up")];
        for (value, label) in options {
            let selected = if value == current_status { " selected" } else { "" };
            opts.push_str(&format!(r#"<option value="{}"{}>{}</option>"#, value, selected, label));
        }
        opts
    };

    let total_selected = if current_status.is_empty() { "ring-2 ring-blue-500" } else { "" };
    let new_selected = if current_status == "new" { "ring-2 ring-green-500" } else { "" };
    let contacted_selected = if current_status == "contacted" { "ring-2 ring-blue-500" } else { "" };
    let callback_selected = if current_status == "callback" { "ring-2 ring-yellow-500" } else { "" };
    let follow_up_selected = if current_status == "follow_up" { "ring-2 ring-purple-500" } else { "" };

    let content = format!(
        r#"<main class="py-6 px-4 sm:px-6 lg:px-8">
        <div class="space-y-6">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-warm-100">Customers</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-warm-400">{} total customers</p>
                </div>
                <div class="flex gap-2">
                    <a href="/customers/import" class="btn btn-secondary">Import CSV</a>
                    <a href="/customers/new" class="btn btn-primary">Add Customer</a>
                </div>
            </div>

            <!-- Stats cards -->
            <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-5">
                <a href="/customers" class="card card-status p-4 {}">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="h-10 w-10 rounded-lg bg-blue-100 dark:bg-blue-900/40 flex items-center justify-center">
                                <svg class="h-5 w-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-3">
                            <p class="text-xs font-medium text-gray-500 dark:text-warm-400">Total</p>
                            <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{}</p>
                        </div>
                    </div>
                </a>

                <a href="/customers?status=new" class="card card-status p-4 {}">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="h-10 w-10 rounded-lg bg-green-100 dark:bg-green-900/40 flex items-center justify-center">
                                <svg class="h-5 w-5 text-green-600 dark:text-green-400" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v6m3-3H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-3">
                            <p class="text-xs font-medium text-gray-500 dark:text-warm-400">New</p>
                            <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{}</p>
                        </div>
                    </div>
                </a>

                <a href="/customers?status=contacted" class="card card-status p-4 {}">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="h-10 w-10 rounded-lg bg-blue-100 dark:bg-blue-900/40 flex items-center justify-center">
                                <svg class="h-5 w-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 6.75c0 8.284 6.716 15 15 15h2.25a2.25 2.25 0 002.25-2.25v-1.372c0-.516-.351-.966-.852-1.091l-4.423-1.106c-.44-.11-.902.055-1.173.417l-.97 1.293c-.282.376-.769.542-1.21.38a12.035 12.035 0 01-7.143-7.143c-.162-.441.004-.928.38-1.21l1.293-.97c.363-.271.527-.734.417-1.173L6.963 3.102a1.125 1.125 0 00-1.091-.852H4.5A2.25 2.25 0 002.25 4.5v2.25z" />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-3">
                            <p class="text-xs font-medium text-gray-500 dark:text-warm-400">Contacted</p>
                            <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{}</p>
                        </div>
                    </div>
                </a>

                <a href="/customers?status=callback" class="card card-status p-4 {}">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="h-10 w-10 rounded-lg bg-yellow-100 dark:bg-yellow-900/40 flex items-center justify-center">
                                <svg class="h-5 w-5 text-yellow-600 dark:text-yellow-400" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-3">
                            <p class="text-xs font-medium text-gray-500 dark:text-warm-400">Callback</p>
                            <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{}</p>
                        </div>
                    </div>
                </a>

                <a href="/customers?status=follow_up" class="card card-status p-4 {}">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="h-10 w-10 rounded-lg bg-purple-100 dark:bg-purple-900/40 flex items-center justify-center">
                                <svg class="h-5 w-5 text-purple-600 dark:text-purple-400" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 12c0-1.232-.046-2.453-.138-3.662a4.006 4.006 0 00-3.7-3.7 48.678 48.678 0 00-7.324 0 4.006 4.006 0 00-3.7 3.7c-.017.22-.032.441-.046.662M19.5 12l3-3m-3 3l-3-3m-12 3c0 1.232.046 2.453.138 3.662a4.006 4.006 0 003.7 3.7 48.656 48.656 0 007.324 0 4.006 4.006 0 003.7-3.7c.017-.22.032-.441.046-.662M4.5 12l3 3m-3-3l-3 3" />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-3">
                            <p class="text-xs font-medium text-gray-500 dark:text-warm-400">Follow Up</p>
                            <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{}</p>
                        </div>
                    </div>
                </a>
            </div>

            <!-- Filters -->
            <div class="card p-4">
                <form class="flex flex-col sm:flex-row gap-4">
                    <div class="flex-1">
                        <input type="search" name="search" placeholder="Search customers..." value="{}" class="form-input">
                    </div>
                    <div class="sm:w-48">
                        <select name="status" class="form-input" onchange="this.form.submit()">
                            <option value="">All Statuses</option>
                            {}
                        </select>
                    </div>
                </form>
            </div>

            <!-- Customer list -->
            <div class="card overflow-hidden">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-warm-700">
                    <thead class="bg-gray-50 dark:bg-warm-900">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Name</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Email</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Phone</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Status</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Created</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="bg-white dark:bg-warm-800 divide-y divide-gray-200 dark:divide-warm-700">
                        {}
                    </tbody>
                </table>
            </div>
        </div>
    </main>"#,
        total, total_selected, total, new_selected, new_count, contacted_selected, contacted_count, callback_selected, callback_count, follow_up_selected, follow_up_count, search_value, filter_options, rows
    );

    axum::response::Html(base_html("Customers", &sidebar_layout("/customers", &content)))
}

pub async fn create_page() -> impl IntoResponse {
    let content = r#"
    <main class="py-6 px-4 sm:px-6 lg:px-8">
        <div class="card p-6">
            <h3 class="text-lg font-medium text-gray-900 dark:text-warm-100 mb-4">Add New Customer</h3>
            <form method="post" action="/customers" class="space-y-6">
                <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
                    <div class="sm:col-span-2">
                        <label class="form-label">Name *</label>
                        <input type="text" name="name" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Email</label>
                        <input type="email" name="email" class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Phone *</label>
                        <input type="tel" name="phone" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Website</label>
                        <input type="url" name="website" class="form-input">
                    </div>
                    <div>
                        <label class="form-label">City *</label>
                        <input type="text" name="city" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">State *</label>
                        <input type="text" name="state" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Industry</label>
                        <input type="text" name="industry" class="form-input">
                    </div>
                </div>
                <div class="flex justify-end gap-3">
                    <a href="/customers" class="btn btn-secondary">Cancel</a>
                    <button type="submit" class="btn btn-primary">Create Customer</button>
                </div>
            </form>
        </div>
    </main>"#;

    axum::response::Html(base_html("Add Customer", &sidebar_layout("/customers", content)))
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<CreateCustomer>,
) -> impl IntoResponse {
    match Customer::create(&state.pool, &form).await {
        Ok(_) => axum::response::Redirect::to("/customers").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let customer = match Customer::find(&state.pool, id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            let content = r#"<main class="py-6 px-4 sm:px-6 lg:px-8">
                <div class="card p-6 text-center">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-warm-100 mb-2">Customer Not Found</h2>
                    <p class="text-gray-500 dark:text-warm-400 mb-4">The customer you're looking for doesn't exist.</p>
                    <a href="/customers" class="btn btn-primary">Back to Customers</a>
                </div>
            </main>"#;
            return axum::response::Html(base_html("Not Found", &sidebar_layout("/customers", content)));
        }
        Err(e) => {
            return axum::response::Html(format!("<div class='p-4 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'>Database error: {}</div>", e));
        }
    };

    let notes = crate::models::CustomerNote::by_customer(&state.pool, id).await.unwrap_or_default();

    let mut notes_html = String::new();
    for note in &notes {
        notes_html.push_str(&format!(
            r#"<div class="py-2 border-b border-gray-200 dark:border-warm-700 last:border-0">
                <p class="text-sm text-gray-800 dark:text-warm-200">{}</p>
                <p class="text-xs text-gray-500 dark:text-warm-500">{}</p>
            </div>"#,
            note.body,
            note.created_at.format("%Y-%m-%d %H:%M")
        ));
    }

    let review_badge = if customer.needs_review {
        let reason = customer.review_reason.as_deref().unwrap_or("No reason provided");
        format!(r#"<div class="mt-2 p-2 bg-yellow-50 dark:bg-yellow-900/30 border border-yellow-200 dark:border-yellow-800 rounded text-xs text-yellow-800 dark:text-yellow-200">
            <strong>Needs Review:</strong> {}
            <form method="post" action="/customers/{}/clear-review" class="inline ml-2">
                <button type="submit" class="underline hover:no-underline">Dismiss</button>
            </form>
        </div>"#, reason, customer.id)
    } else {
        String::new()
    };

    let content = format!(
        r#"<main class="py-6 px-4 sm:px-6 lg:px-8">
        <div class="mb-4">
            <a href="/customers" class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300 text-sm">&larr; Back to Customers</a>
        </div>
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-2">
                <div class="card p-6">
                    <div class="flex justify-between items-start mb-4">
                        <h3 class="text-lg font-medium text-gray-900 dark:text-warm-100">{}</h3>
                        <a href="/customers/{}/edit" class="btn btn-secondary text-sm">Edit</a>
                    </div>
                    {}
                    <dl class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4">
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">Email</dt>
                            <dd class="text-sm text-gray-900 dark:text-warm-200">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">Phone</dt>
                            <dd class="text-sm text-gray-900 dark:text-warm-200">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">Website</dt>
                            <dd class="text-sm text-gray-900 dark:text-warm-200">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">City</dt>
                            <dd class="text-sm text-gray-900 dark:text-warm-200">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">State</dt>
                            <dd class="text-sm text-gray-900 dark:text-warm-200">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">Industry</dt>
                            <dd class="text-sm text-gray-900 dark:text-warm-200">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500 dark:text-warm-500">Status</dt>
                            <dd class="mt-1" id="status-badge-{}">{}</dd>
                        </div>
                    </dl>
                </div>
            </div>
            <div>
                <div class="card p-6">
                    <h3 class="text-lg font-medium text-gray-900 dark:text-warm-100 mb-4">Notes</h3>
                    <div id="notes-list">{}</div>
                    <form method="post" action="/customers/{}/notes" class="mt-4 space-y-3">
                        <textarea name="body" rows="3" required placeholder="Add a note..." class="form-input"></textarea>
                        <button type="submit" class="btn btn-primary w-full">Add Note</button>
                    </form>
                </div>
            </div>
        </div>
    </main>"#,
        customer.name,
        customer.id,
        review_badge,
        customer.email.as_deref().unwrap_or("-"),
        customer.phone.as_deref().unwrap_or("-"),
        customer.website.as_deref().unwrap_or("-"),
        customer.city.as_deref().unwrap_or("-"),
        customer.state.as_deref().unwrap_or("-"),
        customer.industry.as_deref().unwrap_or("-"),
        customer.id,
        status_badge(&customer),
        notes_html,
        id
    );

    axum::response::Html(base_html(&customer.name, &sidebar_layout("/customers", &content)))
}

pub async fn edit_page(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let customer = match Customer::find(&state.pool, id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            let content = r#"<main class="py-6 px-4 sm:px-6 lg:px-8">
                <div class="card p-6 text-center">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-warm-100 mb-2">Customer Not Found</h2>
                    <p class="text-gray-500 dark:text-warm-400 mb-4">The customer you're looking for doesn't exist.</p>
                    <a href="/customers" class="btn btn-primary">Back to Customers</a>
                </div>
            </main>"#;
            return axum::response::Html(base_html("Not Found", &sidebar_layout("/customers", content)));
        }
        Err(e) => {
            return axum::response::Html(format!("<div class='p-4 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'>Database error: {}</div>", e));
        }
    };

    let status_options = {
        let mut opts = String::new();
        let options = [("new", "New"), ("contacted", "Contacted"), ("callback", "Callback"), ("follow_up", "Follow Up")];
        for (value, label) in options {
            let selected = if value == customer.status { " selected" } else { "" };
            opts.push_str(&format!(r#"<option value="{}"{}>{}</option>"#, value, selected, label));
        }
        opts
    };

    let content = format!(
        r#"<main class="py-6 px-4 sm:px-6 lg:px-8">
        <div class="mb-4">
            <a href="/customers/{}" class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300 text-sm">&larr; Back to Customer</a>
        </div>
        <div class="card p-6">
            <h3 class="text-lg font-medium text-gray-900 dark:text-warm-100 mb-4">Edit Customer</h3>
            <form method="post" action="/customers/{}" class="space-y-6">
                <input type="hidden" name="_method" value="PUT">
                <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
                    <div class="sm:col-span-2">
                        <label class="form-label">Name *</label>
                        <input type="text" name="name" value="{}" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Email</label>
                        <input type="email" name="email" value="{}" class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Phone *</label>
                        <input type="tel" name="phone" value="{}" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Website</label>
                        <input type="url" name="website" value="{}" class="form-input">
                    </div>
                    <div>
                        <label class="form-label">City *</label>
                        <input type="text" name="city" value="{}" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">State *</label>
                        <input type="text" name="state" value="{}" required class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Industry</label>
                        <input type="text" name="industry" value="{}" class="form-input">
                    </div>
                    <div>
                        <label class="form-label">Status</label>
                        <select name="status" class="form-input">
                            {}
                        </select>
                    </div>
                </div>
                <div class="flex justify-end gap-3">
                    <a href="/customers/{}" class="btn btn-secondary">Cancel</a>
                    <button type="submit" class="btn btn-primary">Update Customer</button>
                </div>
            </form>
        </div>
    </main>"#,
        customer.id,
        customer.id,
        customer.name,
        customer.email.as_deref().unwrap_or(""),
        customer.phone.as_deref().unwrap_or(""),
        customer.website.as_deref().unwrap_or(""),
        customer.city.as_deref().unwrap_or(""),
        customer.state.as_deref().unwrap_or(""),
        customer.industry.as_deref().unwrap_or(""),
        status_options,
        customer.id
    );

    axum::response::Html(base_html("Edit Customer", &sidebar_layout("/customers", &content)))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<UpdateCustomer>,
) -> impl IntoResponse {
    let customer = match Customer::find(&state.pool, id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            let content = r#"<main class="py-6 px-4 sm:px-6 lg:px-8">
                <div class="card p-6 text-center">
                    <h2 class="text-xl font-bold text-gray-900 dark:text-warm-100 mb-2">Customer Not Found</h2>
                    <p class="text-gray-500 dark:text-warm-400 mb-4">The customer you're looking for doesn't exist.</p>
                    <a href="/customers" class="btn btn-primary">Back to Customers</a>
                </div>
            </main>"#;
            return axum::response::Html(base_html("Not Found", &sidebar_layout("/customers", content))).into_response();
        }
        Err(e) => {
            return axum::response::Html(format!("<div class='p-4 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'>Database error: {}</div>", e)).into_response();
        }
    };

    match customer.update(&state.pool, &form).await {
        Ok(_) => {
            return axum::response::Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0;url=/customers/{}"></head>
<body><a href="/customers/{}">Redirect</a></body>
</html>"#,
                id, id
            )).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let customer = match Customer::find(&state.pool, id).await {
        Ok(Some(c)) => c,
        _ => return axum::response::Html("<div>Customer not found</div>").into_response(),
    };
    let _ = customer.delete(&state.pool).await;
    axum::response::Html(format!(
        r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0;url=/customers"></head>
<body><a href="/customers">Redirect</a></body>
</html>"#
    )).into_response()
}

pub async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<StatusForm>,
) -> impl IntoResponse {
    let customer = match Customer::update_status(&state.pool, id, &form.status).await {
        Ok(c) => c,
        Err(e) => return axum::response::Html(format!("<div class='p-4 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'>Error: {}</div>", e)).into_response(),
    };

    let badge = status_badge(&customer);
    let mut headers = HeaderMap::new();
    headers.insert("HX-Trigger", HeaderValue::from_static("statusChanged"));
    headers.insert("Content-Type", "text/html".parse().unwrap());

    (headers, axum::response::Html(badge)).into_response()
}

pub async fn clear_review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let customer = match Customer::find(&state.pool, id).await {
        Ok(Some(c)) => c,
        _ => return axum::response::Html("<div>Customer not found</div>").into_response(),
    };

    match customer.clear_review_flag(&state.pool).await {
        Ok(_) => {
            return axum::response::Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0;url=/customers/{}"></head>
<body><a href="/customers/{}">Redirect</a></body>
</html>"#,
                id, id
            )).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }
}
