use actix_web::{web, HttpResponse, Responder};
use sysinfo::System;
use serde::Serialize;
use sqlx::PgPool;
use chrono::Utc;
use tokio::time::{interval, Duration};


#[derive(Serialize, Clone)]
pub struct SystemInfo {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

impl SystemInfo {
    pub fn new(
        sub_admin_metrics_id: Option<i32>,
        staff_metrics_id: Option<i32>,
        name: Option<String>,
        hostname: Option<String>,
        os_version: Option<String>,
        kernel_version: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            name,
            hostname,
            os_version,
            kernel_version,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    pub fn update(
        &mut self,
        new_name: Option<String>,
        new_hostname: Option<String>,
        new_os_version: Option<String>,
        new_kernel_version: Option<String>,
    ) {
        if new_name != self.name
            || new_hostname != self.hostname
            || new_os_version != self.os_version
            || new_kernel_version != self.kernel_version
        {
            self.name = new_name;
            self.hostname = new_hostname;
            self.os_version = new_os_version;
            self.kernel_version = new_kernel_version;
            self.updated_at = Some(Utc::now());
        }
    }
}

pub async fn get_system_info_handler(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();

    match get_system_info_for_user(&pool, user_id_value).await {
        Ok(mut system_info) => {
            let mut sys = System::new_all();
            sys.refresh_all();

            let name = Some(System::name().unwrap_or_default());
            let hostname = Some(System::host_name().unwrap_or_default());
            let os_version = Some(System::os_version().unwrap_or_default());
            let kernel_version = Some(System::kernel_version().unwrap_or_default());

            system_info.update(name, hostname, os_version, kernel_version);
            if let Err(e) = save_systeminfo_metrics_to_database(&pool, &system_info).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save system info");
            }
            HttpResponse::Ok().json(system_info)
        }
        Err(_) => {
            let new_system_info = gather_system_info(Some(user_id_value), None).await;
            if let Err(e) = save_systeminfo_metrics_to_database(&pool, &new_system_info).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save system info");
            }
            HttpResponse::Ok().json(new_system_info)
        }
    }
}

pub async fn gather_system_info(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> SystemInfo {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let mut sys = System::new_all();
    sys.refresh_all();

    let name = Some(System::name().unwrap_or_default());
    let hostname = Some(System::host_name().unwrap_or_default());
    let os_version = Some(System::os_version().unwrap_or_default());
    let kernel_version = Some(System::kernel_version().unwrap_or_default());

    return SystemInfo::new(
        sub_admin_metrics_id,
        staff_metrics_id,
        name,
        hostname,
        os_version,
        kernel_version,
    )
}
}
pub async fn save_systeminfo_metrics_to_database(
    pool: &PgPool,
    metrics: &SystemInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query!(
        "INSERT INTO systeminfo_metrics (sub_admin_metrics_id, staff_metrics_id, name, hostname, os_version, kernel_version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        metrics.sub_admin_metrics_id,
        metrics.staff_metrics_id,
        metrics.name,
        metrics.hostname,
        metrics.os_version,
        metrics.kernel_version,
        metrics.created_at,
        metrics.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_system_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<SystemInfo, sqlx::Error> {
    sqlx::query_as!(
        SystemInfo,
        "SELECT sub_admin_metrics_id, staff_metrics_id, name, hostname, os_version, kernel_version, created_at, updated_at 
        FROM systeminfo_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_system_info(pool: &PgPool) -> Result<Vec<SystemInfo>, sqlx::Error> {
    sqlx::query_as!(
        SystemInfo,
        "SELECT sub_admin_metrics_id, staff_metrics_id, name, hostname, os_version, kernel_version, created_at, updated_at FROM systeminfo_metrics"
    )
    .fetch_all(pool)
    .await
}
