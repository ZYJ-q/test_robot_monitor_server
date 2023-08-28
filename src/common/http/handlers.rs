use actix_web::{error, web, Error, HttpResponse};
use chrono::Utc;
use futures_util::StreamExt as _;
use mysql::Pool;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use super::{database, SignIn, SignInRes, SignOut, InvitationRes, AccShareList,SelectTraderMess, DelAccGroup, CheckAdmins, CheckAccounts, SelectTraders,AccShareTra, UpdateEquitys, AccGroupShare, DeleteShareList, DeleteShareAcc, DeleteShareAccGroup, IsAccTra, AddShareList, DetailGroup, IsAccGroup, DeleteTradeSlackNotice, Group, AddGroupTra, AddAccGroup, DeleteAccountTra, AddAccountGroup, AddTradeSlackNotice, AddTradeNotice, SelectAllInvitation, SelectInvitation, InsertAccounts, SelectWeixin,CreateInvitation, SelectNewOrders, UpdateBorrow, UpdateCurreny, Klines, SelectAccounts, InsertAccount, Account, actions, Trade, Posr, NetWorthRe, IncomesRe, Equity, DateTrade, DelectOrders, AddOrders, AddPositions, UpdatePositions,AccountEquity, UpdateOriBalance, UpdateAlarms, AddAccounts, SelectId, SelectAccount};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status: u32,
    data: T,
}

pub async fn sign_in(
    mut payload: web::Payload,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SignIn>(&body)?;

    let query = database::check_account(db_pool.clone(), &obj.name, &obj.password);
    match query {
        Ok(data) => match data {
            Some(response) => {
                let rand_string: String = thread_rng()
                    .sample_iter(Alphanumeric)
                    .take(15)
                    .map(char::from)
                    .collect();
                match database::add_active(
                    db_pool,
                    response.acc_id,
                    &rand_string,
                    &response.acc_name,
                ) {
                    Ok(pros) => {
                        return Ok(HttpResponse::Ok().json(Response {
                            status: 200,
                            data: SignInRes {
                                name: response.acc_name,
                                account: response.acc_id,
                                admin: response.admin,
                                products: pros,
                                token: rand_string,
                            },
                        }));
                    }
                    Err(e) => {
                        return Err(error::ErrorNotFound(e));
                    }
                }
            }
            None => {
                return Err(error::ErrorNotFound("account not exist"));
            }
        },
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 生成邀请码
pub async fn create_invitation(
    mut payload: web::Payload,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<CreateInvitation>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let query = database::check_account_admin(db_pool.clone(), &obj.name);
    match query {
        Ok(data) => match data {
            Some(response) => {
                let rand_string: String = thread_rng()
                    .sample_iter(Alphanumeric)
                    .take(5)
                    .map(char::from)
                    .collect();
                match database::add_invitation(
                    db_pool,
                    &rand_string,
                    &response.acc_name
                ) {
                    Ok(pros) => {
                        return Ok(HttpResponse::Ok().json(Response {
                            status: 200,
                            data: InvitationRes {
                                name: response.acc_name,
                                invitation: pros,
                            },
                        }));
                    }
                    Err(e) => {
                        return Err(error::ErrorNotFound(e));
                    }
                }
            }
            None => {
                return Err(error::ErrorNotFound("account not exist"));
            }
        },
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn sign_out(
    mut payload: web::Payload,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SignOut>(&body)?;

    match database::remove_active(db_pool.clone(), &obj.name, &obj.token) {
        Ok(()) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: format!("succeed"),
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取账户列表的权益杠杆率数据
pub async fn account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_traders(db_pool.clone()) {
        Ok(traders) => {
            let acct_re = actions::get_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取账户列表papi
pub async fn papi_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_traders(db_pool.clone()) {
        Ok(traders) => {
            let acct_re = actions::get_papi_account_(traders).await;
            // println!("{:?}", acct_re);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


// 获取账户列表和账户的唯一标识符
pub async fn account_list(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data =  database::get_account_list(db_pool.clone()); 
        match data {
            Ok(traders) => {
                // println!("{:?}", acct_re);
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
    
}

// 获取单个Biance账户的papi详情数据
pub async fn single_papi_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_one_traders(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_papi_account_(traders).await;
            let dw = database::insert_trader_mess(db_pool.clone(), acct_re.clone());
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


// 获取该账户是否监控该数据
pub async fn alarm_account_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectTraders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_account_data(db_pool.clone(), &obj.account_id); 
    match data {
        Ok(traders) => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: traders,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


// 获取该账户是否监控该数据
pub async fn insert_trader(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<InsertAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::insert_traders(db_pool.clone(), &obj.tra_venue, &obj.tra_currency, &obj.ori_balance, &obj.api_key, &obj.secret_key, &obj.r#type, &obj.name, &obj.alarm, &obj.threshold, &obj.thres_amount, &obj.borrow_currency); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}



// 获取该账户是否监控该数据
pub async fn insert_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccounts>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::select_accounts(db_pool.clone(), &obj.name, &obj.account_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


// 判断用户是否是该账户和账户组的创始者
pub async fn is_check_admins(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<CheckAdmins>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::is_admins(db_pool.clone(), &obj.account_id, &obj.tra_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

//删除账户组
pub async fn delete_acc_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelAccGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_acc_group(db_pool.clone(), &obj.group_id, &obj.account_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

// 移除账户组
pub async fn remove_acc_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelAccGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::remove_acc_group(db_pool.clone(), &obj.group_id, &obj.account_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

pub async fn check_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<CheckAccounts>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::check_trader(db_pool.clone(), &obj.api_key, &obj.secret_key); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


pub async fn insert_accounts(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<InsertAccounts>(&body)?;

    let data = database::insert_account(db_pool.clone(), &obj.user_name, &obj.password); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


pub async fn select_invitations(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectInvitation>(&body)?;

    let data =  database::check_invitation(db_pool.clone(), &obj.code);
        match data {
            Ok(traders) => {
                // println!("{:#?}", traders);
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
    
}

pub async fn select_all_invitations(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAllInvitation>(&body)?;
    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data =  database::check_all_invitation(db_pool.clone(), &obj.user);
        match data {
            Ok(traders) => {
                // println!("{:#?}", traders);
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
    
}


pub async fn insert_weixins(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectWeixin>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::insert_weixins(db_pool.clone(), &obj.wx_name, &obj.wx_hook, &obj.name); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}




pub async fn papi_positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_papi_history_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn papi_assets(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_papi_account_asset(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn papi_income(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_papi_account_income(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn papi_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_papi_history_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn papi_klines(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Klines>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_traders(db_pool.clone()) {
        Ok(traders) => {
            let acct_re = actions::get_papi_kilines(traders, &obj.symbol).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取账户列表的权益杠杆率数据
pub async fn bybit_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_traders(db_pool.clone()) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_account_(traders).await;
            // println!("{:?}", acct_re);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

// 获取单个账户的详情数据
pub async fn single_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data =  database::get_one_traders_message(db_pool.clone(), &obj.tra_id);
        match data {
            Ok(traders) => {
                // println!("{:#?}", traders);
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 获取单个bybit账户的详情数据
pub async fn single_bybit_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_one_traders(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_account_(traders).await;

            let dw = database::insert_trader_mess(db_pool.clone(), acct_re.clone());
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}



pub async fn single_bian_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectAccount>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_one_traders(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_single_account(traders).await;
            let dw = database::insert_trader_mess(db_pool.clone(), acct_re.clone());
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}




// 获取所有账户列表（显示为机器人列表）
pub async fn get_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectTraders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_all_traders(db_pool.clone(), &obj.account_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 获取所有账户列表（显示为机器人列表）
pub async fn get_total_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectTraders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_total_traders(db_pool.clone(), &obj.account_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


pub async fn get_account_message(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectTraders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_all_traders_message(db_pool.clone(), &obj.account_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


pub async fn is_account_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<IsAccGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::is_acc_group(db_pool.clone(), obj.account_id, obj.group_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

pub async fn share_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccShareTra>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::share_account(db_pool.clone(), &obj.account_id, &obj.tra_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


pub async fn share_group_account(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccGroupShare>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::share_group_account(db_pool.clone(), &obj.account_id, &obj.group_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

pub async fn share_group_account_tra(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccGroupShare>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::share_group_account_tra(db_pool.clone(), &obj.account_id, &obj.group_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

pub async fn add_shara_account_list(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddShareList>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_share_list(db_pool.clone(), &obj.from_id, &obj.to_id, &obj.tra_id, &obj.tra_name); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


// 删除账户分享
pub async fn del_shara_acc(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DeleteShareAcc>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_acc_share_list(db_pool.clone(), &obj.to_id, &obj.tra_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


// 删除账户分享记录
pub async fn del_shara_list(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DeleteShareList>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_share_list(db_pool.clone(), &obj.sh_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


// 删除账户组分享
pub async fn del_shara_acc_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DeleteShareAccGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_acc_group_share_list(db_pool.clone(), &obj.to_id, &obj.group_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}

// 获取分享列表
pub async fn get_share_list(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccShareList>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_account_share_list(db_pool.clone(), &obj.from_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


pub async fn is_account_tra(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<IsAccTra>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::is_acc_tra(db_pool.clone(), obj.account_id, obj.tra_id); 
    match data {
        true => {
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));
        },
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        },
    }
}


// 获取权益数据（显示资金曲线）
pub async fn get_bybit_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_bybit_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 清除数据
pub async fn clear_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Account>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::clear_data(db_pool.clone());
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}




// 获取bian权益数据（显示资金曲线）
pub async fn get_bian_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_bian_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}



// 获取后续的bybit权益数据（显示资金曲线）
pub async fn get_total_bybit_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_total_bybit_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


// 获取bian权益数据（显示资金曲线）

pub async fn get_total_bian_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let now = Utc::now();
    let date = format!("{}", now.format("%Y/%m/%d %H:%M:%S"));
    println!("开始时间{}", date);
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_total_bian_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
                
            }
            Err(e) => {
                println!("错误返回代码{}", e);
                return Err(error::ErrorInternalServerError(e));
            }
            
        }

    
}


// 获取bian权益数据（显示资金曲线）
pub async fn get_total_papi_bian_equity(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AccountEquity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_total_papi_bian_equity(db_pool.clone(), &obj.name);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}

pub async fn positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_history_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn futures_bybit_positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn spot_bybit_positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_spot_position(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_history_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn futures_bybit_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_futures_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}

pub async fn spot_bybit_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_spot_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn spot_bybit_usdc_open_orders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_usdc_open_order(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn assets(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_history_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}



pub async fn bybit_assets(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_history_account(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}



pub async fn incomes(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acc_income_re = actions::get_history_income(traders).await;
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acc_income_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn bybit_incomes(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    match database::get_trader_positions(db_pool.clone(), &obj.tra_id) {
        Ok(traders) => {
            let acct_re = actions::get_bybit_history_income(traders).await;
            // println!("{:#?}", traders);
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: acct_re,
            }));
        }
        Err(e) => {
            return Err(error::ErrorInternalServerError(e));
        }
    }
}


pub async fn trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_trades(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 获取bybit账户订单详细
pub async fn bybit_trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_bybit_trades(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


pub async fn history_incomes(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_incomes(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_income) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_income,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 查找账户的通知方式
pub async fn get_trader_notices(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::trader_notice_way(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_income) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_income,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 添加账户的通知方式
pub async fn add_wx_trader_notices(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddTradeNotice>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::insert_traders_wx_notices(db_pool.clone(), &obj.tra_id, &obj.wx_hook, &obj.wx_name);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}


pub async fn check_weixin_ways(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddTradeNotice>(&body)?;
    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data =  database::check_traders_wx_notices(db_pool.clone(), &obj.tra_id, &obj.wx_hook, &obj.wx_name);
        match data {
            Ok(traders) => {
                // println!("{:#?}", traders);
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
    
}


// 添加账户的通知方式
pub async fn add_slack_trader_notices(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddTradeSlackNotice>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::insert_traders_slack_notices(db_pool.clone(), &obj.tra_id, &obj.slack_hook, &obj.slack_name);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}


// 删除账户的通知方式
pub async fn delete_slack_trader_notices(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DeleteTradeSlackNotice>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_slack_trader_notices(db_pool.clone(), &obj.tra_id, &obj.hook);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}

// 添加账号组名字
pub async fn add_account_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddAccountGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_account_group(db_pool.clone(), &obj.name);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}


// 添加账号组配置tra_id
pub async fn add_group_tra(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddGroupTra>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::insert_group_tra(db_pool.clone(), &obj.name, obj.tra_id);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}


// 删除账号权限里面的配置
pub async fn delete_acc_tra(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DeleteAccountTra>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_account_tra(db_pool.clone(), &obj.account_id, obj.tra_id);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}


// 添加账号组
pub async fn add_acc_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddAccGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::insert_acc_group(db_pool.clone(), &obj.name, &obj.account_id);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}

pub async fn get_account_group_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Group>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_account_group_tra(db_pool.clone(), obj.account_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


pub async fn get_account_group(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Group>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_account_group(db_pool.clone(), obj.account_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}



pub async fn get_account_detail_group_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DetailGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_detail_account_group_tra(db_pool.clone(), obj.group_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


pub async fn get_account_group_tra(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DetailGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_account_group_tras(db_pool.clone(), obj.group_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}



pub async fn get_only_acc_traders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Trade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let date =  database::get_only_traders(db_pool.clone(), &obj.tra_id);
        match date {
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}


pub async fn get_account_detail_group_equitys(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DetailGroup>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

     let data =  database::get_detail_account_group_equity(db_pool.clone(), obj.group_id);
        
           match data { 
            Ok(traders) => {
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
}

// 删除账户的通知方式
pub async fn delete_wx_trader_notices(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DeleteTradeSlackNotice>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delete_wx_trader_notices(db_pool.clone(), &obj.tra_id, &obj.hook);
    match data {
        true => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data,
            }));    
        }
        false => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 404,
                data,
            }).into());
        }
        
    }
}


pub async fn check_slack_ways(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddTradeSlackNotice>(&body)?;
    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data =  database::check_traders_slack_notices(db_pool.clone(), &obj.tra_id, &obj.slack_hook, &obj.slack_name);
        match data {
            Ok(traders) => {
                // println!("{:#?}", traders);
                return Ok(HttpResponse::Ok().json(Response {
                    status: 200,
                    data: traders,
                }));
            }
            Err(e) => {
                return Err(error::ErrorInternalServerError(e));
            }
            
        }
    
}





// 获取账户权益
pub async fn pnl(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_equity(db_pool.clone());
    match data {
        Ok(histor_equity) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_equity,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 获取账户交易额
pub async fn is_price(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_trade_price(db_pool.clone());
    match data {
        Ok(histor_price) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_price,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取仓位数据
pub async fn position(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Posr>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_positions(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(histor_positions) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_positions,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 获取权益数据
pub async fn net_worth(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<NetWorthRe>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_history_networth(db_pool.clone());
    match data {
        Ok(histor_net_worths) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_net_worths,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 根据日期来获取账户的成交记录
pub async fn date_trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DateTrade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_history_trades(db_pool.clone(), &obj.start_time, &obj.end_time, &obj.tra_id);
    match data {
        Ok(histor_date_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_date_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 根据日期来获取bybit账户的成交记录
pub async fn date_bybit_trade(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DateTrade>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_bybit_history_trades(db_pool.clone(), &obj.start_time, &obj.end_time, &obj.tra_id);
    match data {
        Ok(histor_date_trade) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: histor_date_trade,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取所有的产品
pub async fn get_products_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_all_products(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 获取当前所有监控的挂单账户
pub async fn get_open_orders_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_alarm_open_orders(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 删除监控的挂单账户
pub async fn delect_open_orders_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delect_orders(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 添加监控的挂单账户
pub async fn add_open_orders_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_orders(db_pool.clone(), &obj.name, &obj.api_key, &obj.secret_key);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 获取当前所有监控的净头寸账户
pub async fn get_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_alarm_positions(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 删除监控的净头寸账户
pub async fn delect_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delect_positions(db_pool.clone(), &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 添加监控的净头寸账户
pub async fn add_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddPositions>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_positions(db_pool.clone(), &obj.name, &obj.api_key, &obj.secret_key, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 更新监控的净头寸账户阈值
pub async fn update_positions_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdatePositions>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_positions(db_pool.clone(), &obj.name, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 更新权益监控和权益监控中的阈值
pub async fn update_equitys_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateEquitys>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_equitys(db_pool.clone(), &obj.name, &obj.equitys);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 更新监控的净头寸
pub async fn update_positions(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdatePositions>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_threshold(db_pool.clone(), &obj.name, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 更新账户份额
pub async fn update_ori_balance_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateOriBalance>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_ori_balance(db_pool.clone(), &obj.tra_id, &obj.ori_balance);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 更新账户是否进行监控
pub async fn update_accounts_alarm(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateAlarms>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_alarms(db_pool.clone(), &obj.name, &obj.alarm);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

pub async fn update_curreny(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateCurreny>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_currency(db_pool.clone(), &obj.name, &obj.currency);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

pub async fn update_borrow(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<UpdateBorrow>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::update_borrow(db_pool.clone(), &obj.name, &obj.borrow);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 删除账号
pub async fn delete_accounts_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::delect_accounts(db_pool.clone(), &obj.tra_id, &obj.account_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 移除账号
pub async fn remove_accounts_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<DelectOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::remove_accounts(db_pool.clone(), &obj.tra_id, &obj.account_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 查找今天的订单明细
pub async fn select_new_traders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectNewOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_new_trades(db_pool.clone(), &obj.start_time, &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 查找今天的订单明细
pub async fn select_new_bybit_traders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectNewOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_new_bybit_trades(db_pool.clone(), &obj.start_time, &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}

// 查找今天bybit的订单明细
pub async fn select_bybit_new_traders(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectNewOrders>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_date_bybit_new_trades(db_pool.clone(), &obj.start_time, &obj.tra_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}



// 添加账号
pub async fn add_accounts_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<AddAccounts>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::add_accounts(db_pool.clone(), &obj.name, &obj.api_key, &obj.secret_key, &obj.alarm, &obj.threshold);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


// 查找tra_id，并添加到test_prod_tra中
pub async fn select_tra_id(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SelectId>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::select_id(db_pool.clone(), &obj.name, &obj.prod_id);
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


//获取所有的净值数据
pub async fn get_net_worths_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_net_worths(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}


//获取所有的权益数据
pub async fn get_equitys_data(mut payload: web::Payload, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Equity>(&body)?;

    match database::is_active(db_pool.clone(), &obj.token) {
        true => {}
        false => {
            return Err(error::ErrorNotFound("account not active"));
        }
    }

    let data = database::get_equitys(db_pool.clone());
    match data {
        Ok(all_products) => {
            return Ok(HttpResponse::Ok().json(Response {
                status: 200,
                data: all_products,
            }));    
        }
        Err(e) => {
            return Err(error::ErrorNotFound(e));
        }
        
    }
}