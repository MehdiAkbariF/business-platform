use axum::{
    extract::State,
    http::header,
    response::IntoResponse,
};
use crate::state::AppState;

pub async fn get_prometheus_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let pool_size = state.db.pool().size();
    let num_idle = state.db.pool().num_idle();

    // Fetch operational metrics cleanly through Application/Repository ports without direct SQL in handlers
    let metrics = state.admin_repo.get_dashboard_metrics().await.ok();
    let total_biz = metrics.as_ref().map(|m| m.total_published_businesses).unwrap_or(0);
    let pending_biz = metrics.as_ref().map(|m| m.pending_businesses).unwrap_or(0);
    let active_users = metrics.as_ref().map(|m| m.total_active_users).unwrap_or(0);
    let active_campaigns = metrics.as_ref().map(|m| m.active_campaigns).unwrap_or(0);

    let prometheus_format = format!(
        "# HELP db_connections_total Total database pool connections\n\
         # TYPE db_connections_total gauge\n\
         db_connections_total {}\n\
         # HELP db_connections_idle Idle database pool connections\n\
         # TYPE db_connections_idle gauge\n\
         db_connections_idle {}\n\
         # HELP platform_businesses_published Total published businesses\n\
         # TYPE platform_businesses_published gauge\n\
         platform_businesses_published {}\n\
         # HELP platform_businesses_pending Total pending review businesses\n\
         # TYPE platform_businesses_pending gauge\n\
         platform_businesses_pending {}\n\
         # HELP platform_active_users Total active users\n\
         # TYPE platform_active_users gauge\n\
         platform_active_users {}\n\
         # HELP platform_active_campaigns Total active ad campaigns\n\
         # TYPE platform_active_campaigns gauge\n\
         platform_active_campaigns {}\n",
        pool_size, num_idle, total_biz, pending_biz, active_users, active_campaigns
    );

    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")], prometheus_format)
}