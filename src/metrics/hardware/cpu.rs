use actix_web::{web, HttpResponse, Responder};
use sysinfo::System;
use serde::Serialize;
use sqlx::PgPool;
use chrono::prelude::*;
use tokio::time::{interval, Duration};

#[derive(Serialize, Clone)]
pub struct CpuMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub last_refresh: Option<NaiveDateTime>,
    pub cpu_info: Option<String>,
    pub usage_summary: Option<String>,
}

impl CpuMetrics {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, cpu_info: Option<String>, usage_summary: Option<String>) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.naive_utc();
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            last_refresh: Some(timestamp),
            cpu_info,
            usage_summary,
        }
    }

    pub fn update(&mut self, new_cpu_info: Option<String>, new_usage_summary: Option<String>) {
        self.cpu_info = new_cpu_info;
        self.usage_summary = new_usage_summary;
        self.last_refresh = Some(Utc::now().naive_utc());
    }
}

pub async fn get_cpu_info_handler(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();

    match get_cpu_info_for_user(&pool, user_id_value).await {
        Ok(mut cpu_metrics) => {
            let mut sys = System::new_all();
            sys.refresh_cpu();

            let cpu_info = sys.cpus().first().map(|cpu| {
                format!("Name: {}, Vendor ID: {}, Brand: {}", cpu.name(), cpu.vendor_id(), cpu.brand())
            });

            let usage_summary = format!(
                "Average: {}, Max: {}, Min: {}",
                calculate_average_cpu_usage(0), // Placeholder value
                calculate_max_cpu_usage(0),     // Placeholder value
                calculate_min_cpu_usage(0)      // Placeholder value
            );

            cpu_metrics.update(cpu_info, Some(usage_summary));
            if let Err(e) = save_cpu_metrics_to_database(&pool, &cpu_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save CPU info");
            }
            HttpResponse::Ok().json(cpu_metrics)
        }
        Err(_) => {
            let new_cpu_metrics = gather_cpu_metrics(Some(user_id_value), None).await;
            if let Err(e) = save_cpu_metrics_to_database(&pool, &new_cpu_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save CPU info");
            }
            HttpResponse::Ok().json(new_cpu_metrics)
        }
    }
}

pub async fn gather_cpu_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> CpuMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let mut sys = System::new_all();
    sys.refresh_cpu();

    let cpu_info = sys.cpus().first().map(|cpu| {
        format!("Name: {}, Vendor ID: {}, Brand: {}", cpu.name(), cpu.vendor_id(), cpu.brand())
    });

    let usage_summary = format!(
        "Average: {}, Max: {}, Min: {}",
        calculate_average_cpu_usage(0), // Placeholder value
        calculate_max_cpu_usage(0),     // Placeholder value
        calculate_min_cpu_usage(0)      // Placeholder value
    );

    return CpuMetrics::new(
        sub_admin_metrics_id,
        staff_metrics_id,
        cpu_info,
        Some(usage_summary),
    )
}
}
pub async fn save_cpu_metrics_to_database(
    pool: &PgPool,
    metrics: &CpuMetrics,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query!(
        "INSERT INTO cpu_metrics (sub_admin_metrics_id, staff_metrics_id, last_refresh, cpu_info, usage_summary) VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (sub_admin_metrics_id) DO UPDATE SET 
            last_refresh = EXCLUDED.last_refresh, 
            cpu_info = EXCLUDED.cpu_info, 
            usage_summary = EXCLUDED.usage_summary",
        metrics.sub_admin_metrics_id,
        metrics.staff_metrics_id,
        metrics.last_refresh,
        metrics.cpu_info,
        metrics.usage_summary
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_cpu_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<CpuMetrics, sqlx::Error> {
    sqlx::query_as!(
        CpuMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, last_refresh, cpu_info, usage_summary 
        FROM cpu_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}

fn calculate_average_cpu_usage(_sub_admin_metrics_id: i32) -> String {
    // Implement your logic to calculate average CPU usage based on sub_admin_metrics_id
    // For illustration, returning a placeholder
    "Average CPU usage".to_string()
}

fn calculate_max_cpu_usage(_sub_admin_metrics_id: i32) -> String {
    // Implement your logic to calculate max CPU usage based on sub_admin_metrics_id
    // For illustration, returning a placeholder
    "Max CPU usage".to_string()
}

fn calculate_min_cpu_usage(_sub_admin_metrics_id: i32) -> String {
    // Implement your logic to calculate min CPU usage based on sub_admin_metrics_id
    // For illustration, returning a placeholder
    "Min CPU usage".to_string()
}

pub async fn get_cpu_info(pool: &PgPool) -> Result<Vec<CpuMetrics>, sqlx::Error> {
    sqlx::query_as!(
        CpuMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, last_refresh, cpu_info, usage_summary FROM cpu_metrics"
    )
    .fetch_all(pool)
    .await
}
