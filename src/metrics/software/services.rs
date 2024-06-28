use serde::Serialize;
use sqlx::PgPool;
use std::process::Command;
// use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};
use actix_web::{web, HttpResponse, Responder};


#[derive(Serialize, Clone)]
pub struct ServiceStatus {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub service_name: Option<String>,
    pub status: Option<String>,
}

impl ServiceStatus {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, service_name: Option<String>, status: Option<String>) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            service_name,
            status,
        }
    }

    pub fn update(&mut self, new_status: Option<String>) {
        self.status = new_status;
    }
}

pub async fn gather_services_status_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> ServiceStatus {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let services = vec!["wuauserv", "WinDefend"];
    for service in &services {
        let status = match fetch_service_status(service).await {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Failed to fetch service status: {}", e);
                continue;
            }
        };

    return ServiceStatus::new(sub_admin_metrics_id, staff_metrics_id, Some(service.to_string()), status)
}
}
}

pub async fn get_services_status_info_handler(pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();
    match get_services_status_info_for_user(&pool, user_id_value).await{
    Ok(mut services_metrics) => {
    let services = vec!["wuauserv", "WinDefend"];
    for service in &services {
        let status = match fetch_service_status(service).await {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Failed to fetch service status: {}", e);
                continue;
            }
        };
            if let Err(e) = save_service_status_to_database(&pool, &services_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save Memory info");
            }  
    }
    HttpResponse::Ok().json(services_metrics)
}
    Err(_) => {
    let new_services_metrics = gather_services_status_metrics(Some(user_id_value), None).await;
    if let Err(e) = save_service_status_to_database(&pool, &new_services_metrics).await {
        eprintln!("Failed to save metrics to database: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save Service Status info");
    }
    HttpResponse::Ok().json(new_services_metrics)
}
}
}

async fn fetch_service_status(service_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = Command::new("sc")
        .arg("query")
        .arg(service_name)
        .output()
        .expect("Failed to check service status");
    let status = String::from_utf8(output.stdout).ok();
    Ok(status)
}

pub async fn save_service_status_to_database(pool: &PgPool, metrics: &ServiceStatus) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "INSERT INTO service_status_metrics (sub_admin_metrics_id, staff_metrics_id, service_name, status) VALUES ($1, $2, $3, $4)
             ON CONFLICT (sub_admin_metrics_id, service_name) DO UPDATE
             SET status = EXCLUDED.status",
            metrics.sub_admin_metrics_id,
            metrics.staff_metrics_id,
            metrics.service_name,
            metrics.status,
        )
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_service_status_info(pool: &PgPool) -> Result<Vec<ServiceStatus>, sqlx::Error> {
 sqlx::query_as!(
        ServiceStatus,
        "SELECT sub_admin_metrics_id, staff_metrics_id, service_name, status FROM service_status_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_services_status_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<ServiceStatus, sqlx::Error> {
    sqlx::query_as!(
        ServiceStatus,
        "SELECT sub_admin_metrics_id, staff_metrics_id, service_name, status 
        FROM service_status_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}