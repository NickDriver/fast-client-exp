use axum::{
    response::IntoResponse,
};

pub async fn import_page() -> impl IntoResponse {
    let content = r#"
    <main class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div class="mb-4">
            <a href="/customers" class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300 text-sm">&larr; Back to Customers</a>
        </div>
        <div class="card p-6">
            <h3 class="text-lg font-medium text-gray-900 dark:text-warm-100 mb-4">Import Customers from CSV</h3>
            <form method="post" action="/customers/import" enctype="multipart/form-data" class="space-y-6">
                <div>
                    <label class="form-label">CSV File</label>
                    <input type="file" name="csv_file" accept=".csv" required class="form-input">
                </div>
                <div class="bg-gray-50 dark:bg-warm-900 p-4 rounded-md">
                    <h4 class="text-sm font-medium text-gray-700 dark:text-warm-200 mb-2">Required Columns:</h4>
                    <p class="text-sm text-gray-500 dark:text-warm-400">name, phone, city, state</p>
                    <p class="text-sm text-gray-500 dark:text-warm-400 mt-1">Optional: email, website, industry, status</p>
                </div>
                <div class="flex justify-end gap-3">
                    <a href="/customers" class="btn btn-secondary">Cancel</a>
                    <button type="submit" class="btn btn-primary">Import Customers</button>
                </div>
            </form>
        </div>
    </main>"#;

    let nav = r#"<nav class="bg-white shadow-sm border-b dark:bg-warm-900 dark:border-warm-700">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex h-16 items-center justify-between">
            <div class="flex items-center gap-4">
                <h1 class="text-xl font-bold text-indigo-600 dark:text-indigo-400">FastClient</h1>
                <a href="/customers" class="text-gray-600 hover:text-gray-900 dark:text-warm-300 dark:hover:text-warm-100 text-sm">Customers</a>
            </div>
            <a href="/logout" class="text-gray-600 hover:text-gray-900 dark:text-warm-300 dark:hover:text-warm-100 text-sm">Logout</a>
        </div>
    </nav>"#;

    axum::response::Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Import Customers - FastClient</title>
    <link rel="stylesheet" href="/assets/css/app.css">
    <script src="/assets/js/htmx.min.js"></script>
    <script>
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {{
            document.documentElement.classList.add('dark');
        }}
    </script>
</head>
<body class="bg-gray-50 min-h-screen dark:bg-warm-950">
    {}
    {}
    <script src="/assets/js/app.js"></script>
</body>
</html>"#,
        nav, content
    ))
}
