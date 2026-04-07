use crate::models::Customer;
use crate::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Extension,
};
use uuid::Uuid;

pub async fn dashboard(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> impl IntoResponse {
    let customers = Customer::all(&state.pool).await.unwrap_or_default();
    
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

    let recent: Vec<&Customer> = customers.iter().rev().take(5).collect();

    let mut table_rows = String::new();
    for c in recent {
        let status_badge = match c.status.as_str() {
            "new" => r#"<span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300">New</span>"#,
            "contacted" => r#"<span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300">Contacted</span>"#,
            "callback" => r#"<span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300">Callback</span>"#,
            "follow_up" => r#"<span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-purple-100 text-purple-800 dark:bg-purple-900/40 dark:text-purple-300">Follow Up</span>"#,
            _ => r#"<span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800 dark:bg-gray-900/40 dark:text-gray-300">Unknown</span>"#,
        };
        table_rows.push_str(&format!(
            r#"<tr class="hover:bg-gray-50 dark:hover:bg-warm-700/50">
                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-warm-100">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">{}</td>
                <td class="px-6 py-4 whitespace-nowrap">{}</td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-warm-400">{}</td>
            </tr>"#,
            c.name,
            c.email.as_deref().unwrap_or("-"),
            c.phone.as_deref().unwrap_or("-"),
            status_badge,
            c.created_at.format("%Y-%m-%d")
        ));
    }

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en" class="h-full">
<head>
    <meta charset="UTF-8">
    <title>Dashboard - FastClient</title>
    <link rel="stylesheet" href="/assets/css/app.css">
    <script src="/assets/js/htmx.min.js"></script>
    <script src="/assets/js/htmx-ext-preload.js"></script>
    <script>
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {{
            document.documentElement.classList.add('dark');
        }}
    </script>
</head>
<body class="h-full dark:bg-warm-950" hx-ext="preload">
    <div class="min-h-full flex">
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
                <a href="/" class="sidebar-link sidebar-link-active">
                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                    </svg>
                    Dashboard
                </a>

                <a href="/customers" class="sidebar-link">
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

            <main class="py-6 px-4 sm:px-6 lg:px-8">
                <div class="space-y-6">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-warm-100">Dashboard</h1>
                        <p class="mt-1 text-sm text-gray-500 dark:text-warm-400">Welcome back!</p>
                    </div>

                    <!-- Stats cards -->
                    <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-5">
                        <a href="/customers" class="card card-status p-4">
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
                                    <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{total}</p>
                                </div>
                            </div>
                        </a>

                        <a href="/customers?status=new" class="card card-status p-4">
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
                                    <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{new_count}</p>
                                </div>
                            </div>
                        </a>

                        <a href="/customers?status=contacted" class="card card-status p-4">
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
                                    <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{contacted_count}</p>
                                </div>
                            </div>
                        </a>

                        <a href="/customers?status=callback" class="card card-status p-4">
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
                                    <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{callback_count}</p>
                                </div>
                            </div>
                        </a>

                        <a href="/customers?status=follow_up" class="card card-status p-4">
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
                                    <p class="text-xl font-semibold text-gray-900 dark:text-warm-100">{follow_up_count}</p>
                                </div>
                            </div>
                        </a>
                    </div>

                    <!-- Quick actions -->
                    <div class="card p-6">
                        <h2 class="text-lg font-medium text-gray-900 dark:text-warm-100 mb-4">Quick Actions</h2>
                        <div class="flex flex-wrap gap-3">
                            <a href="/customers/new" class="btn btn-primary">
                                <svg class="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                                </svg>
                                Add Customer
                            </a>
                            <a href="/customers" class="btn btn-secondary">View All Customers</a>
                        </div>
                    </div>

                    <!-- Recent Customers -->
                    <div class="card overflow-hidden">
                        <div class="px-4 py-5 sm:p-6">
                            <h3 class="text-lg font-medium text-gray-900 dark:text-warm-100 mb-4">Recent Customers</h3>
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-warm-700">
                                <thead class="bg-gray-50 dark:bg-warm-900">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Name</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Email</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Phone</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Status</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase dark:text-warm-400">Created</th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white dark:bg-warm-800 divide-y divide-gray-200 dark:divide-warm-700">
                                    {table_rows}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    </div>
    <script src="/assets/js/app.js"></script>
</body>
</html>"#,
        total = total, new_count = new_count, contacted_count = contacted_count, callback_count = callback_count, follow_up_count = follow_up_count, table_rows = table_rows
    ))
}
