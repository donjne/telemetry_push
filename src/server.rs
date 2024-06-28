use actix_web::{web, App, HttpServer};
use crate::functionalities::maintenance::count_ongoing_maintenance_requests;
use crate::metrics::hardware::{
    cpu::get_cpu_info_handler,
    memory::get_memory_info_handler,
    disk::get_disk_info_handler,
    network::get_network_info_handler,
    aboutsys::get_system_info_handler,
};
use crate::metrics::software::{
    ip_location::get_ip_location_info_handler,
    uptime::get_uptime_info_handler,
    services::get_services_status_info_handler,
    filesystem::get_filesystem_info_handler,
    process::get_process_info_handler
};
use crate::user::users::{self, get_all_staffs_by_company, get_all_sub_admins, count_staffs_by_company, count_sub_admins};
use crate::user::login::login;
use sqlx::PgPool;
use crate::auth::middleware::AuthMiddleware;
use crate::functionalities::{
maintenance::{create_maintenance_request, get_user_maintenance_requests, get_user_specific_maintenance_request, update_maintenance_request, delete_maintenance_request, get_ongoing_maintenance_count}, 
assign::{create_system_assignment, get_system_assignment, update_system_assignment, delete_system_assignment, get_system_assignment_count}};

pub async fn run_server(pool: PgPool) {
        HttpServer::new(move|| {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/login", web::post().to(login))
            .route("/seeallsubadmin", web::get().to(get_all_sub_admins))
            .route("/countallsubadmin", web::get().to(count_sub_admins))
            .route("/seeallmystaffs", web::get().to(get_all_staffs_by_company))
            .route("/countallmystaffs", web::get().to(count_staffs_by_company))
            .route("/countongoingmaintenancereq", web::get().to(get_ongoing_maintenance_count))
            .route("/systeminfo/{user_id}", web::get().to(get_system_info_handler))
            .route("/cpu", web::get().to(get_cpu_info_handler))
            .route("/disk", web::get().to(get_disk_info_handler))
            .route("/memory", web::get().to(get_memory_info_handler))
            .route("/network", web::get().to(get_network_info_handler))
            .route("/filesystem", web::get().to(get_filesystem_info_handler))
            .route("/iplocation", web::get().to(get_ip_location_info_handler))
            .route("/process", web::get().to(get_process_info_handler))
            .route("/services", web::get().to(get_services_status_info_handler))
            .route("/uptime", web::get().to(get_uptime_info_handler))
            .route("/createsub", web::post().to(users::createsub))
            .route("/createsuper", web::post().to(users::createsuper))
            .route("/createstaff", web::post().to(users::createstaff))
            .route("/createtechnician", web::post().to(users::createtechnician))
            .route("/createreq", web::post().to(create_maintenance_request))
            .route("/maintenance/user/{reported_by_id}", web::get().to(get_user_maintenance_requests))
            .route("/maintenance/user/{reported_by_id}/{maintenance_id}", web::get().to(get_user_specific_maintenance_request))
            .route("/maintenance/user/{reported_by_id}/{maintenance_id}", web::patch().to(update_maintenance_request))
            .route("/maintenance/user/{reported_by_id}/{maintenance_id}", web::delete().to(delete_maintenance_request))
            .route("/ongoing_maintenance/{reported_by_id}", web::get().to(get_ongoing_maintenance_count))
            .route("/systemassign", web::post().to(create_system_assignment))
            .route("/systemassign/{new_system_id}", web::get().to(get_system_assignment))
            .route("/systemassign/{new_system_id}", web::patch().to(update_system_assignment))
            .route("/systemassign/{new_system_id}", web::delete().to(delete_system_assignment))
            .route("/systemassigncount/{new_system_id}", web::delete().to(get_system_assignment_count))
            .wrap(AuthMiddleware)
    })
    .bind("127.0.0.1:8080")
    .expect("Can not bind to port 8080")
    .run().await.unwrap();

}
