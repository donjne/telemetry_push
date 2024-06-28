use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use sysinfo::System;
use tokio::time::{interval, Duration};

#[derive(Serialize, Clone)]
pub struct ProcessMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub pid: Option<i32>,
    pub name: Option<String>,
    pub exe: Option<String>,
    pub cpu_usage: Option<f64>,
    pub memory: Option<f64>,
}

impl ProcessMetrics {
    pub fn new(
        sub_admin_metrics_id: Option<i32>,
        staff_metrics_id: Option<i32>,
        pid: Option<i32>,
        name: Option<String>,
        exe: Option<String>,
        cpu_usage: Option<f64>,
        memory: Option<f64>,
    ) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            pid,
            name,
            exe,
            cpu_usage,
            memory,
        }
    }

    pub fn update(
        &mut self,
        new_pid: Option<i32>,
        new_name: Option<String>,
        new_exe: Option<String>,
        new_cpu_usage: Option<f64>,
        new_memory: Option<f64>,
    ) {
        self.pid = new_pid;
        self.name = new_name;
        self.exe = new_exe;
        self.cpu_usage = new_cpu_usage;
        self.memory = new_memory;
    }
}

pub async fn gather_process_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> ProcessMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour

    loop {
        interval.tick().await;

        let mut sys = System::new_all();
        sys.refresh_all();

        let process_metrics = sys.processes().iter().map(|(&pid, process)| {
            ProcessMetrics::new(
                sub_admin_metrics_id,
                staff_metrics_id,
                Some(pid.as_u32() as i32),
                Some(process.name().to_string()),
                process.exe().map(|path| path.to_string_lossy().to_string()),
                Some(process.cpu_usage() as f64),
                Some(process.memory() as f64 / 1024.0 / 1024.0),
            )
        }).collect::<Vec<ProcessMetrics>>();

        if let Some(metric) = process_metrics.into_iter().next() {
            return metric;
        }
    }
}

pub async fn get_process_info_handler(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();

    match get_process_info_for_user(&pool, user_id_value).await {
        Ok(process_metrics) => HttpResponse::Ok().json(process_metrics),
        Err(_) => {
            let new_process_metrics = gather_process_metrics(Some(user_id_value), None).await;
            if let Err(e) = save_process_metrics_to_database(&pool, &new_process_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save process info");
            }
            HttpResponse::Ok().json(new_process_metrics)
        }
    }
}

pub async fn save_process_metrics_to_database(pool: &PgPool, metrics: &ProcessMetrics) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query!(
        "INSERT INTO process_metrics (sub_admin_metrics_id, staff_metrics_id, pid, name, exe, cpu_usage, memory) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        metrics.sub_admin_metrics_id,
        metrics.staff_metrics_id,
        metrics.pid,
        metrics.name,
        metrics.exe,
        metrics.cpu_usage,
        metrics.memory,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_process_info(pool: &PgPool) -> Result<Vec<ProcessMetrics>, sqlx::Error> {
    sqlx::query_as!(
        ProcessMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, pid, name, exe, cpu_usage, memory FROM process_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_process_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<ProcessMetrics, sqlx::Error> {
    sqlx::query_as!(
        ProcessMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, pid, name, exe, cpu_usage, memory 
        FROM process_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}
