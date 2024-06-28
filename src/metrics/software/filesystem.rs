use serde::Serialize;
use sqlx::PgPool;
use std::process::Command;
// use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};
use actix_web::{web, HttpResponse, Responder};

#[derive(Serialize, Clone)]
pub struct FileSystemMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub filesystem: Option<String>,
    pub status: Option<String>,
}

impl FileSystemMetrics {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, filesystem: Option<String>, status: Option<String>) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            filesystem,
            status,
        }
    }

    pub fn update(&mut self, new_filesystem: Option<String>, new_status: Option<String>) {
        self.filesystem = new_filesystem;
        self.status = new_status;
    }
}

pub async fn get_filesystem_info_handler(pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();

    match get_filesystem_info_for_user(&pool, user_id_value).await {
        Ok(mut filesystem_metrics) => {
        let filesystems = vec!["C:", "D:"];
        for filesystem in &filesystems {
            let output = Command::new("fsutil")
                .arg("volume")
                .arg("diskfree")
                .arg(filesystem)
                .output()
                .expect("Failed to check filesystem status");
            let status = String::from_utf8_lossy(&output.stdout).to_string();
            filesystem_metrics.update(Some(filesystem.to_string()), Some(status));
            if let Err(e) = save_filesystem_metrics_to_database(&pool, &filesystem_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save Memory info");
            }  
    }
    HttpResponse::Ok().json(filesystem_metrics)

}
    Err(_) => {
    let new_filesystem_metrics = gather_filesystem_metrics(Some(user_id_value), None).await;
    if let Err(e) = save_filesystem_metrics_to_database(&pool, &new_filesystem_metrics).await {
        eprintln!("Failed to save metrics to database: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save Filesystem info");
    }
    HttpResponse::Ok().json(new_filesystem_metrics)
}
}
}

pub async fn gather_filesystem_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> FileSystemMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let filesystems = vec!["C:", "D:"];
    for filesystem in &filesystems {
        let output = Command::new("fsutil")
            .arg("volume")
            .arg("diskfree")
            .arg(filesystem)
            .output()
            .expect("Failed to check filesystem status");

        let status = String::from_utf8_lossy(&output.stdout).to_string();
    return FileSystemMetrics::new(sub_admin_metrics_id, staff_metrics_id, Some(filesystem.to_string()), Some(status))
}
}
}

pub async fn save_filesystem_metrics_to_database(pool: &PgPool, metrics: &FileSystemMetrics) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "INSERT INTO filesystem_metrics (sub_admin_metrics_id, staff_metrics_id, filesystem, status) VALUES ($1, $2, $3, $4)",
            metrics.sub_admin_metrics_id,
            metrics.staff_metrics_id,
            metrics.filesystem,
            metrics.status,
        )
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_filesystem_info(pool: &PgPool) -> Result<Vec<FileSystemMetrics>, sqlx::Error> {
    sqlx::query_as!(
        FileSystemMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, filesystem, status FROM filesystem_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_filesystem_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<FileSystemMetrics, sqlx::Error> {
    sqlx::query_as!(
        FileSystemMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, filesystem, status 
        FROM filesystem_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}
