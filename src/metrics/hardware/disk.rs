use sysinfo::Disks;
use serde::Serialize;
use sqlx::PgPool;
// use std::sync::Arc;
use tokio::time::{interval, Duration};
use actix_web::{web, HttpResponse, Responder};


#[derive(Serialize, Clone)]
pub struct DiskMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub total_space: Option<f64>,
    pub available_space: Option<f64>,
}

impl DiskMetrics {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, total_space: Option<f64>, available_space: Option<f64>) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            total_space,
            available_space,
        }
    }

    pub fn update(&mut self, new_total_space: Option<f64>, new_available_space: Option<f64>) {
        self.total_space = new_total_space;
        self.available_space = new_available_space;
    }
}

pub async fn get_disk_info_handler(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner(); 
    match get_cpu_info_for_user(&pool, user_id_value).await {
        Ok(mut disk_metrics) => {
        let disks = Disks::new_with_refreshed_list();
        let total_space: Option<f64> = disks.iter().map(|d| Some(d.total_space() as f64)).sum::<Option<f64>>().map(|total| total / 1_048_576.0);
        let available_space: Option<f64> = disks.iter().map(|d| Some(d.available_space() as f64)).sum::<Option<f64>>().map(|available| available / 1_048_576.0);


        disk_metrics.update(total_space, available_space);
        if let Err(e) = save_disk_metrics_to_database(&pool, &disk_metrics).await {
            eprintln!("Failed to save metrics to database: {}", e);
            return HttpResponse::InternalServerError().body("Failed to save CPU info");
        }   
        HttpResponse::Ok().json(disk_metrics)
    }
    Err(_) => {
        let new_disk_metrics = gather_disk_metrics(Some(user_id_value), None).await;
        if let Err(e) = save_disk_metrics_to_database(&pool, &new_disk_metrics).await {
            eprintln!("Failed to save metrics to database: {}", e);
            return HttpResponse::InternalServerError().body("Failed to save CPU info");
        }
        HttpResponse::Ok().json(new_disk_metrics)
    }
}
}

pub async fn gather_disk_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> DiskMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let disks = Disks::new_with_refreshed_list();
    let total_space: Option<f64> = disks.iter().map(|d| Some(d.total_space() as f64)).sum::<Option<f64>>().map(|total| total / 1_048_576.0);
    let available_space: Option<f64> = disks.iter().map(|d| Some(d.available_space() as f64)).sum::<Option<f64>>().map(|available| available / 1_048_576.0);
    return DiskMetrics::new(sub_admin_metrics_id, staff_metrics_id, total_space, available_space)
}
}
pub async fn save_disk_metrics_to_database(pool: &PgPool, metrics: &DiskMetrics) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "INSERT INTO disk_metrics (sub_admin_metrics_id, staff_metrics_id, total_space, available_space) VALUES ($1, $2, $3, $4)",
            metrics.sub_admin_metrics_id,
            metrics.staff_metrics_id,
            metrics.total_space,
            metrics.available_space,
        )
        .execute(pool)
        .await?;
    Ok(())
    }
   

pub async fn get_disk_info(pool: &PgPool) -> Result<Vec<DiskMetrics>, sqlx::Error> {
    sqlx::query_as!(
        DiskMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, total_space, available_space FROM disk_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_cpu_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<DiskMetrics, sqlx::Error> {
    sqlx::query_as!(
        DiskMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, total_space, available_space 
        FROM disk_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}
