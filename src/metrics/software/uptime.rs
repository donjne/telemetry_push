use serde::Serialize;
use std::error::Error;
// use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};
use sysinfo::System;
use sqlx::PgPool;
use chrono::Utc;
use actix_web::{web, HttpResponse, Responder};

#[derive(Serialize, Clone, Copy)]
pub struct UptimeMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub uptime: Option<f64>,
    pub downtime: Option<f64>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

impl UptimeMetrics {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, uptime: Option<f64>, downtime: Option<f64>) -> Self {
        let now = Utc::now();
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            uptime,
            downtime,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    pub fn update(&mut self, new_uptime: Option<f64>, new_downtime: Option<f64>) {
        if let Some(new_uptime) = new_uptime {
            if let Some(uptime) = self.uptime {
                if new_uptime > uptime {
                    self.downtime = Some(self.downtime.unwrap_or(0.0) + new_uptime - uptime);
                }
            }
            self.uptime = Some(new_uptime);
        }
        if let Some(new_downtime) = new_downtime {
            self.downtime = Some(new_downtime);
        }
        self.updated_at = Some(Utc::now());
    }
}

pub async fn get_uptime_info_handler(pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();

    match get_uptime_info_for_user(&pool, user_id_value).await {
        Ok(mut uptime_metrics) => {
            let mut system = System::new_all();
    system.refresh_all();
    let uptime_hours = Some(System::uptime() as f64 / 3600.0);
    let downtime_hours = uptime_hours.map(|u| u - 1.0);
            uptime_metrics.update(uptime_hours, downtime_hours);
            if let Err(e) = save_uptime_metrics_to_database(&pool, &uptime_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save Memory info");
            }  
    HttpResponse::Ok().json(uptime_metrics)

    }
    Err(_) => {
    let new_uptime_metrics = gather_uptime_metrics(Some(user_id_value), None).await;
    if let Err(e) = save_uptime_metrics_to_database(&pool, &new_uptime_metrics).await {
        eprintln!("Failed to save metrics to database: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save Uptime info");
    }
    HttpResponse::Ok().json(new_uptime_metrics)
}
}
}

pub async fn gather_uptime_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> UptimeMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let mut system = System::new_all();
    system.refresh_all();
    let uptime_hours = Some(System::uptime() as f64 / 3600.0);
    let downtime_hours = uptime_hours.map(|u| u - 1.0);
    return UptimeMetrics::new(sub_admin_metrics_id, staff_metrics_id, uptime_hours, downtime_hours)
}
}


pub async fn save_uptime_metrics_to_database(pool: &PgPool, metrics: &UptimeMetrics) -> Result<(), Box<dyn Error>> {
        sqlx::query!(
            "INSERT INTO uptime_metrics (sub_admin_metrics_id, staff_metrics_id, uptime, downtime, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
            metrics.sub_admin_metrics_id,
            metrics.staff_metrics_id,
            metrics.uptime,
            metrics.downtime,
            metrics.created_at,
            metrics.updated_at,
        )
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_uptime_info(pool: &PgPool) -> Result<Vec<UptimeMetrics>, sqlx::Error> {
    sqlx::query_as!(
        UptimeMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, uptime, downtime, created_at, updated_at FROM uptime_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_uptime_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<UptimeMetrics, sqlx::Error> {
    sqlx::query_as!(
        UptimeMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, uptime, downtime, created_at, updated_at 
        FROM uptime_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}