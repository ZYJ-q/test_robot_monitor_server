use std::collections::HashMap;
use actix_web::{middleware, web, App, HttpServer};


use super::database;
use super::http::handlers;



pub async fn server(ip: String, config_db: HashMap<String, String>) -> std::io::Result<()> {
    log::info!("starting HTTP server at http://{}:8082", &ip);

    let pool = database::create_pool(config_db);

    let server = HttpServer::new(move || {
        App::new()
            // enable logger
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            // <- limit size of the payload (global configuration)
            .service(web::resource("/signIn").route(web::post().to(handlers::sign_in)))
            .service(web::resource("/signOut").route(web::post().to(handlers::sign_out)))
            .service(web::resource("/account").route(web::post().to(handlers::account)))
            .service(web::resource("/trades").route(web::post().to(handlers::trade)))
            .service(web::resource("/position").route(web::post().to(handlers::positions)))
            .service(web::resource("/net_worth").route(web::post().to(handlers::net_worth)))
            .service(web::resource("/income").route(web::post().to(handlers::incomes)))
            .service(web::resource("/equity").route(web::post().to(handlers::pnl)))
            .service(web::resource("/newPrice").route(web::post().to(handlers::is_price)))
            .service(web::resource("/incomes").route(web::post().to(handlers::history_incomes)))
            .service(web::resource("/open_orders").route(web::post().to(handlers::open_orders)))
            .service(web::resource("/accounts").route(web::post().to(handlers::assets)))
            .service(web::resource("/date_history").route(web::post().to(handlers::date_trade)))
            .service(web::resource("/products").route(web::post().to(handlers::get_products_data)))
            .service(web::resource("/alarm_orders").route(web::post().to(handlers::get_open_orders_data)))
            .service(web::resource("/delect_orders").route(web::post().to(handlers::delect_open_orders_data)))
            .service(web::resource("/add_orders").route(web::post().to(handlers::add_open_orders_data)))
            .service(web::resource("/get_positions").route(web::post().to(handlers::get_positions_data)))
            .service(web::resource("/delect_positions").route(web::post().to(handlers::delect_positions_data)))
            .service(web::resource("/add_positions").route(web::post().to(handlers::add_positions_data)))
            .service(web::resource("/update_positions").route(web::post().to(handlers::update_positions_data)))
            .service(web::resource("/update_ori_balance").route(web::post().to(handlers::update_ori_balance_data)))
            .service(web::resource("/get_accounts").route(web::post().to(handlers::get_account)))
            .service(web::resource("/update_accounts_alarm").route(web::post().to(handlers::update_accounts_alarm)))
            .service(web::resource("/delete_accounts").route(web::post().to(handlers::delete_accounts_data)))
            .service(web::resource("/add_accounts").route(web::post().to(handlers::add_accounts_data)))
            .service(web::resource("/select_id").route(web::post().to(handlers::select_tra_id)))
            .service(web::resource("/get_net_worths").route(web::post().to(handlers::get_net_worths_data)))
            .service(web::resource("/get_equitys").route(web::post().to(handlers::get_equitys_data)))
            .service(web::resource("/get_single_account").route(web::post().to(handlers::single_account)))
            .service(web::resource("/get_bybit_account").route(web::post().to(handlers::bybit_account)))
            .service(web::resource("/get_bybit_equity").route(web::post().to(handlers::get_bybit_equity)))
            .service(web::resource("/get_bian_equity").route(web::post().to(handlers::get_bian_equity)))
            .service(web::resource("/data_bybit_history").route(web::post().to(handlers::date_bybit_trade)))
            .service(web::resource("/bybit_trades").route(web::post().to(handlers::bybit_trade)))
            .service(web::resource("/bybit_futures_position").route(web::post().to(handlers::futures_bybit_positions)))
            .service(web::resource("/bybit_spot_position").route(web::post().to(handlers::spot_bybit_positions)))
            .service(web::resource("/bybit_futures_open_order").route(web::post().to(handlers::futures_bybit_open_orders)))
            .service(web::resource("/bybit_spot_open_order").route(web::post().to(handlers::spot_bybit_open_orders)))
            .service(web::resource("/bybit_usdc_open_order").route(web::post().to(handlers::spot_bybit_usdc_open_orders)))
            .service(web::resource("/bybit_assets").route(web::post().to(handlers::bybit_assets)))
            .service(web::resource("/bybit_incomes").route(web::post().to(handlers::bybit_incomes)))
            .service(web::resource("/get_bybit_single_account").route(web::post().to(handlers::single_bybit_account)))
            .service(web::resource("/total_bybit_equity").route(web::post().to(handlers::get_total_bybit_equity)))
            .service(web::resource("/total_bian_equity").route(web::post().to(handlers::get_total_bian_equity)))
            .service(web::resource("/clear_equity").route(web::post().to(handlers::clear_equity)))
            .service(web::resource("/get_papi_account").route(web::post().to(handlers::papi_account)))
            .service(web::resource("/get_papi_single_account").route(web::post().to(handlers::single_papi_account)))
            .service(web::resource("/get_papi_positions").route(web::post().to(handlers::papi_positions)))
            .service(web::resource("/get_papi_open_orders").route(web::post().to(handlers::papi_open_orders)))
            .service(web::resource("/get_papi_klines").route(web::post().to(handlers::papi_klines)))
            .service(web::resource("/get_papi_assets").route(web::post().to(handlers::papi_assets)))
            .service(web::resource("/get_papi_incomes").route(web::post().to(handlers::papi_income)))
            .service(web::resource("/get_papi_equity").route(web::post().to(handlers::get_total_papi_bian_equity)))
    })
    .bind((ip.as_str(), 8082))?
    .run();

    return server.await;
}