use std::collections::HashMap;

use actix_web::web;
use mysql::prelude::*;
use mysql::*;

// use crate::common;

// use super::AlarmUnit;
use super::db_data::{Account, Active, AccountData, Product, Trader, GroupTra, NewTrade, GroupEquity, TraderMessage, AccountGroup, BybitNewTrade, ClearData, NoticesData, InvitationData, Trade, Position, NetWorth, Equity, NewPrice, HistoryIncomes, OpenOrders, PositionsAlarm, BybitTrade, NetWorths, Equitys, BybitEquity, BianEquity};
use super::http_data::{SignInProRes, CreateInvitationProRes, GroupAccountProRes, GroupEquitysProRes};

pub fn create_pool(config_db: HashMap<String, String>) -> Pool {
    let user = config_db.get("user").unwrap();
    let password = config_db.get("password").unwrap();
    let host = config_db.get("host").unwrap();
    let port = config_db.get("port").unwrap();
    let database = config_db.get("database").unwrap();
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        user, password, host, port, database
    );
    let pool = Pool::new(url).unwrap();
    return pool;
}

pub fn check_account(pool: web::Data<Pool>, name: &str, password: &str) -> Result<Option<Account>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from accounts where acc_name = :name and acc_password = :password",
            params! {
                "name" => name,
                "password" => password
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(acc_id, acc_name, acc_password, admin)| Account {
                    acc_id,
                    acc_name,
                    acc_password,
                    admin
                })
            },
        );

    return res;
}


pub fn check_account_admin(pool: web::Data<Pool>, name: &str) -> Result<Option<Account>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from accounts where acc_name = :name and admin = :admin",
            params! {
                "name" => name,
                "admin" => "true",
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(acc_id, acc_name, acc_password, admin)| Account {
                    acc_id,
                    acc_name,
                    acc_password,
                    admin
                })
            },
        );

    // println!("检查账户是否是管理员{:?}", res);

    return res;
}

// 查看是否有此邀请码
pub fn check_invitation(pool: web::Data<Pool>, code: &str) -> Result<Option<InvitationData>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from invitation where code = :code",
            params! {
                "code" => code,
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(code, user, max, status, id)| InvitationData {
                    code,
                    user,
                    max,
                    status,
                    id
                })
            },
        );
        

    return res;
}


pub fn check_all_invitation(pool: web::Data<Pool>, user: &str) -> Result<Vec<InvitationData>> {
    let mut invitation: Vec<InvitationData> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from invitation where user = :user",
            params! {
                "user" => user,
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(code, user, max, status, id)| InvitationData {
                    code,
                    user,
                    max,
                    status,
                    id
                })
            },
        );


        match res {
            Ok(trader) => match trader {
                Some(tra) => {
                    invitation.push(tra);
                }
                None => {
                    return Ok(invitation);
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
        

    return Ok(invitation);
}

// 新增用户
pub fn insert_account(pool: web::Data<Pool>, acc_name: &str, acc_password: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"insert into accounts (acc_name, acc_password, admin) values (:acc_name, :acc_password, :admin)",
        params! {
            "acc_name" => acc_name,
            "acc_password" => acc_password,
            "admin" => "false"
        },
    );
    match res {
        Ok(c) => {
            return true;
        },
        Err(e) => {
            return false;
        }
    };
}


pub fn add_active(
    pool: web::Data<Pool>,
    account_id: u64,
    token: &str,
    name: &str,
) -> Result<Vec<SignInProRes>> {
    let mut conn = pool.get_conn().unwrap();
    let mut re: Vec<SignInProRes> = Vec::new();
    let res = conn
        .exec_first(
            r"select * from active where name = :name",
            params! {
                "name" => name
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(acc_id, token, name)| Active {
                    acc_id,
                    token,
                    name,
                })
            },
        );
    match res {
        Ok(resq) => match resq {
            Some(active) => {
                conn.exec_drop(
                    r"delete from active where name = :name",
                    params! {
                        "name" => active.name
                    },
                )
                .unwrap();
            }
            None => {}
        },
        Err(_) => {}
    }

    let res = conn.exec_drop(
        r"INSERT INTO active (acc_id, token, name) VALUES (:acc_id, :token, :name)",
        params! {
            "acc_id" => account_id,
            "token" => token,
            "name" => name,
        },
    );
    match res {
        Ok(()) => match get_products(pool, account_id) {
            Ok(res) => match res {
                Some(data) => {
                    for item in data {
                        re.push(SignInProRes {
                            name: String::from(item.prod_name),
                            id: item.prod_id.to_string(),
                        });
                    }
                    return Ok(re);
                }
                None => {
                    return Ok(re);
                }
            },
            Err(e) => {
                return Err(e);
            }
        },
        Err(e) => {
            return Err(e);
        }
    }
}


pub fn add_invitation(
    pool: web::Data<Pool>,
    code: &str,
    name: &str,
) -> Result<Vec<CreateInvitationProRes>> {
    let mut conn = pool.get_conn().unwrap();
    let mut re: Vec<CreateInvitationProRes> = Vec::new();

    let res = conn.exec_drop(
        r"INSERT INTO invitation (code, user, max, status) VALUES (:code, :user, :max, :status)",
        params! {
            "code" => code,
            "user" => name,
            "max" => "10",
            "status" => "success"
        },
    );
    match res {
        Ok(()) => match get_invitation(pool, name) {
            Ok(res) => {
                    for item in res {
                        re.push(CreateInvitationProRes {
                            code: item.code,
                            max: item.max,
                            status: item.status
                        });
                    }
                    return Ok(re);
            },
            Err(e) => {
                return Err(e);
            }
        },
        Err(e) => {
            return Err(e);
        }
    }
}


pub fn is_active(pool: web::Data<Pool>, token: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"select * from actives where token = :token",
        params! {
            "token" => token,
        },
    );
    match res {
        Ok(()) => {
            return true;
        }
        Err(_) => {
            return false;
        }
    }
}

pub fn remove_active(pool: web::Data<Pool>, name: &str, token: &str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"delete from actives where token = :token and name = :name",
        params! {
            "token" => token,
            "name" => name
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}



pub fn get_products(pool: web::Data<Pool>, account_id: u64) -> Result<Option<Vec<Product>>> {
    let mut products: Vec<Product> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select prod_id from test_acc_prod where acc_id = :acc_id",
        params! {
            "acc_id" => account_id
        },
    );
    match res {
        Ok(prod_ids) => {
            for prod_id in prod_ids {
                let mut conn = pool.get_conn().unwrap();
                let prod = conn
                    .exec_first(
                        r"select * from test_products where prod_id = :prod_id",
                        params! {
                            "prod_id" => prod_id
                        },
                    )
                    .map(
                        // Unpack Result
                        |row| {
                            row.map(|(prod_id, prod_name, weixin_id, prog_id)| Product {
                                prod_id,
                                prod_name,
                                weixin_id,
                                prog_id,
                            })
                        },
                    );
                match prod {
                    Ok(produc) => match produc {
                        Some(product) => {
                            products.push(product);
                        }
                        None => {
                            continue;
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            return Ok(Some(products));
        }
        Err(e) => return Err(e),
    }
}

// 获取账户列表
pub fn get_traders(pool: web::Data<Pool>) -> Result<HashMap<String, Trader>> {
    let mut traders: HashMap<String, Trader> = HashMap::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from trader",
        |(tra_id,
            tra_venue,
            tra_currency,
            api_key,
            secret_key,
            r#type,
            name,
            alarm,
            threshold, borrow, amount, wx_hook)| Trader {
                tra_id,
                tra_venue,
                tra_currency,
                api_key,
                secret_key,
                r#type,
                name,
                alarm,
                threshold,
                borrow,
                amount,
                wx_hook
            }
    ).unwrap();

    for i in res {
        let name = i.name.as_str();
        traders.insert(String::from(name), i);
    }
    
    return Ok(traders);
}

// 创建邀请码
// pub fn create_invitation(pool: web::Data<Pool>) -> bool {
//     let mut conn = pool.get_conn().unwrap();
// }
// 查看邀请码
pub fn get_invitation(pool: web::Data<Pool>, name: &str) -> Result<Vec<InvitationData>> {
    let mut products: Vec<InvitationData> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let prod = conn
                    .exec_first(
                        r"select * from invitation where user = :user",
                        params! {
                            "user" => name
                        },
                    )
                    .map(
                        // Unpack Result
                        |row| {
                            row.map(|(
                                code,
                                user,
                                max,
                                status,
                                id
                            )| InvitationData {
                                code,
                                user,
                                max,
                                status,
                                id
                            })
                        },
                    );
                match prod {
                    Ok(produc) => match produc {
                        Some(product) => {
                            products.push(product);
                            return  Ok(products);
                        }
                        None => {
                            return Ok(products);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
    // let res = conn.query_map(
    //             value,
    //             |(
    //                 code,
    //                 user,
    //                 max,
    //                 status,
    //                 id
    //             )| InvitationData {
    //                 code,
    //                 user,
    //                 max,
    //                 status,
    //                 id
    //             }
    //             ).unwrap();
    // return Ok(res);

}

// 获取账户列表
pub fn get_account_list(pool: web::Data<Pool>) -> Result<Vec<Trader>> {
    // let mut traders: HashMap<String, Trader> = HashMap::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from trader",
        |(tra_id,
            tra_venue,
            tra_currency,
            api_key,
            secret_key,
            r#type,
            name,
            alarm,
            threshold, borrow, amount, wx_hook)| Trader {
                tra_id,
                tra_venue,
                tra_currency,
                api_key,
                secret_key,
                r#type,
                name,
                alarm,
                threshold,
                borrow,
                amount,
                wx_hook
            }
    ).unwrap();

    // for i in res {
    //     let name = i.name.as_str();
    //     traders.insert(String::from(name), i);
    // }
    
    return Ok(res);
}

// 查看账户是否被监控数据
pub fn get_account_data(pool: web::Data<Pool>, account_id: &u64) -> Result<Vec<AccountData>> {
    let mut conn = pool.get_conn().unwrap();
    let value = &format!("select * from test_acc_tra where acc_id = {}", account_id);
    let res = conn.query_map(
                value,
                |(
                    ap_id,
                    acc_id,
                    tra_id
                )| AccountData {
                    ap_id,
                    acc_id,
                    tra_id
                }
                ).unwrap();
    return Ok(res);

}

// 查看账户是否被监控数据
pub fn insert_traders(pool: web::Data<Pool>,tra_venue: &str, tra_currency: &str, ori_balance:&str, api_key: &str, secret_key:&str, r#type: &str, name: &str, alarm: &str, threshold:&str, thres_amount: &str, borrow_currency: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"insert into trader (tra_venue, tra_currency, api_key, secret_key, type, name, alarm, threshold, borrow, amount, wx_hook) values (:tra_venue, :tra_currency, :api_key, :secret_key, :type, :name, :alarm, :threshold, :borrow, :amount, :wx_hook)",
        params! {
            "tra_venue" => tra_venue,
            "tra_currency" => tra_currency,
            "api_key" => api_key,
            "secret_key" =>  secret_key,
            "type" => r#type,
            "name" =>  name,
            "alarm" => alarm,
            "threshold" => threshold,
            "borrow" => borrow_currency,
            "amount" => thres_amount,
            "wx_hook" => ori_balance,
        },
    );
    match res {
        Ok(c) => {
            return true;
        },
        Err(e) => {
            return false;
        }
    };
}


pub fn insert_weixins(pool: web::Data<Pool>, wx_name: &str, wx_hook: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"insert into weixins (wx_name, wx_hook) values (:wx_name, :wx_hook)",
        params! {
            "wx_name" => wx_name,
            "wx_hook" => wx_hook
        },
    );
    match res {
        Ok(c) => {
            return true;
        },
        Err(e) => {
            return false;
        }
    };
}




// 获取所有的账户列表
pub fn get_all_traders_message(pool: web::Data<Pool>, account_id: &u64) -> Result<Option<Vec<TraderMessage>>> {
    let mut products: Vec<TraderMessage> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select tra_id from test_acc_tra where acc_id = :acc_id",
        params! {
            "acc_id" => account_id
        },
    );
    match res {
        Ok(tra_ids) => {
            for tra_id in tra_ids {
                let mut conn = pool.get_conn().unwrap();
                let prod = conn
                    .exec_first(
                        r"select * from trader_message where tra_id = :tra_id order by id desc limit 1",
                        params! {
                            "tra_id" => tra_id
                        },
                    )
                    .map(
                        // Unpack Result
                        |row| {
                            row.map(|(id,
                                tra_id,
                                name,
                                equity,
                                leverage,
                                position,
                                open_order_amt,
                                avaliable_balance,
                                tra_venue,
                                r#type, total_balance,)| TraderMessage {
                                
                                    id,
                                    tra_id,
                                    name,
                                    equity,
                                    leverage,
                                    position,
                                    open_order_amt,
                                    avaliable_balance,
                                    tra_venue,
                                    r#type,
                                    total_balance,
                               
                            })
                        },
                    );
                match prod {
                    Ok(produc) => match produc {
                        Some(product) => {
                            products.push(product);
                        }
                        None => {
                            continue;
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            return Ok(Some(products));
        }
        Err(e) => return Err(e),
    }
}


pub fn get_all_traders(pool: web::Data<Pool>, account_id: &u64) -> Result<Option<Vec<Trader>>> {
    let mut products: Vec<Trader> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select tra_id from test_acc_tra where acc_id = :acc_id",
        params! {
            "acc_id" => account_id
        },
    );
    match res {
        Ok(tra_ids) => {
            for tra_id in tra_ids {
                let mut conn = pool.get_conn().unwrap();
                let prod = conn
                    .exec_first(
                        r"select * from trader where tra_id = :tra_id",
                        params! {
                            "tra_id" => tra_id
                        },
                    )
                    .map(
                        // Unpack Result
                        |row| {
                            row.map(|(
                                tra_id,
                                tra_venue,
                                tra_currency,
                                api_key,
                                secret_key,
                                r#type,
                                name,
                                alarm,
                                threshold,
                                borrow,
                                amount,
                                wx_hook,
                            )| Trader {
                                tra_id,
                                tra_venue,
                                tra_currency,
                                api_key,
                                secret_key,
                                r#type,
                                name,
                                alarm,
                                threshold,
                                borrow,
                                amount,
                                wx_hook,
                            })
                        },
                    );
                match prod {
                    Ok(produc) => match produc {
                        Some(product) => {
                            products.push(product);
                        }
                        None => {
                            continue;
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            return Ok(Some(products));
        }
        Err(e) => return Err(e),
    }
}

pub fn select_accounts(pool: web::Data<Pool>, name: &str, account_id: &u64) -> bool {
    let mut products: Vec<Trader> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from trader where name = :name",
            params! {
                "name" => name
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(tra_id,
                    tra_venue,
                    tra_currency,
                    api_key,
                    secret_key,
                    r#type,
                    name,
                    alarm,
                    threshold,
                    borrow,
                    amount, wx_hook)| Trader {
                    tra_id,
                    tra_venue,
                    tra_currency,
                    api_key,
                    secret_key,
                    r#type,
                    name,
                    alarm,
                    threshold,
                    borrow,
                    amount,
                    wx_hook
                })
            },
        );
    match res {
        Ok(resq) => match resq {
            Some(active) => {
                let tra = conn.exec_drop(
                    r"insert into test_acc_tra (acc_id, tra_id) values (:acc_id, :tra_id)",
                    params! {
                        "acc_id" => account_id,
                        "tra_id" => active.tra_id
                    },
                );
                match tra {
                    Ok(c) => {
                        return true;
                    },
                    Err(e) => {
                        return false;
                    }
                };
            }
            None => {
                return  false;

            }
        },
        Err(_) => {
            return false;
        }
    }
}

// 获取账户的通知方式
pub fn trader_notice_way(pool: web::Data<Pool>, tra_id: &str) -> Result<Vec<NoticesData>> {
    let mut conn = pool.get_conn().unwrap();
    let value = &format!("select * from notices where tra_id = {}", tra_id);
    let res = conn
        .query_map(
            value,
            |(id, tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name)|{
                NoticesData { id, tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name }
            },
        ).unwrap();
        

    return Ok(res);
}


pub fn get_account_group(pool: web::Data<Pool>, account_id: u64) -> Result<Option<Vec<AccountGroup>>> {
    let mut products: Vec<AccountGroup> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select group_id from acc_group where acc_id = :acc_id",
        params! {
            "acc_id" => account_id
        },
    );
    match res {
        Ok(prod_ids) => {
            for prod_id in prod_ids {
                let mut conn = pool.get_conn().unwrap();
                let prod = conn
                    .exec_first(
                        r"select * from account_group where group_id = :group_id",
                        params! {
                            "group_id" => prod_id
                        },
                    )
                    .map(
                        // Unpack Result
                        |row| {
                            row.map(|(group_id, name)| AccountGroup {
                                group_id,
                                name
                            })
                        },
                    );
                match prod {
                    Ok(produc) => match produc {
                        Some(product) => {
                            products.push(product);
                        }
                        None => {
                            continue;
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            return Ok(Some(products));
        }
        Err(e) => return Err(e),
    }
}


pub fn get_account_group_tra(
    pool: web::Data<Pool>,
    account_id: u64
) -> Result<Vec<GroupAccountProRes>> {
    let mut conn = pool.get_conn().unwrap();
    let mut re: Vec<GroupAccountProRes> = Vec::new();
    match get_account_group(pool, account_id) {
        Ok(res) => match res {
            Some(data) => {
                for item in data {
                    let names = &item.name;
                    let value = &format!("select * from group_tra where group_id = {}", item.group_id);
                    let tra_data = conn.query_map(
                        value, 
                        |(id, group_id, tra_id)| { GroupTra{id, group_id, tra_id}} 
                    ).unwrap();

                    for tra_id in tra_data{
                        let account_data = conn.exec_first(
                            r"select * from trader_message where tra_id = :tra_id order by id desc limit 1", 
                            params! {
                                "tra_id" => tra_id.tra_id
                            }
                        )
                        .map(
                            |row| {
                                row.map(|(id,
                                    tra_id,
                                    name,
                                    equity,
                                    leverage,
                                    position,
                                    open_order_amt,
                                    avaliable_balance,
                                    tra_venue,
                                    r#type,
                                    total_balance,)| TraderMessage {
                                    
                                        id,
                                        tra_id,
                                        name,
                                        equity,
                                        leverage,
                                        position,
                                        open_order_amt,
                                        avaliable_balance,
                                        tra_venue,
                                        r#type,
                                        total_balance,
                                   
                                })
                            },

                        );
                        match account_data {
                            Ok(tra_data) => match tra_data {
                                Some(trader_message) => {
                                    let new_name = names.clone();
                                    re.push(GroupAccountProRes {
                                        name: new_name,
                                        group_id: item.group_id,
                                        tra_id: trader_message.tra_id,
                                        tra_name: trader_message.name,
                                        equity: trader_message.equity,
                                        leverage: trader_message.leverage,
                                        position: trader_message.position,
                                        open_order_amt: trader_message.open_order_amt,
                                        avaliable_balance: trader_message.avaliable_balance,
                                        tra_venue: trader_message.tra_venue,
                                        r#type: trader_message.r#type,
                                        total_balance: trader_message.total_balance
                                    })

                                }
                                None => {
                                    continue;
                                }

                            },
                            Err(e) => {
                                return Err(e);
                            }
                            
                        }
                    }
                }
                return Ok(re);
            }
            None => {
                return Ok(re);
            }
        },
        Err(e) => {
            return Err(e);
        }
}

}




pub fn get_detail_account_group_tra(
    pool: web::Data<Pool>,
    group_id: u64
) -> Result<Option<Vec<TraderMessage>>> {
    let mut conn = pool.get_conn().unwrap();
    let mut re: Vec<TraderMessage> = Vec::new();
    let value = &format!("select * from group_tra where group_id = {}", group_id);
    let tra_data = conn.query_map(
        value, 
        |(id, group_id, tra_id)| { GroupTra{id, group_id, tra_id}} 
    ).unwrap();

    for tra_id in tra_data{
        let account_data = conn.exec_first(
        r"select * from trader_message where tra_id = :tra_id order by id desc limit 1", 
            params! {
                "tra_id" => tra_id.tra_id
            }
        )
        .map(
            |row| {
                row.map(|(id,
                          tra_id,
                          name,
                          equity,
                          leverage,
                          position,
                          open_order_amt,
                          avaliable_balance,
                          tra_venue,
                          r#type,
                          total_balance,)| TraderMessage {
                                    
                          id,
                          tra_id,
                          name,
                          equity,
                          leverage,
                          position,
                          open_order_amt,
                          avaliable_balance,
                          tra_venue,
                          r#type,
                          total_balance,
                                   
                        })
                    },

                );
                        match account_data {
                            Ok(tra_data) => match tra_data {
                                Some(trader_message) => {
                                    re.push(trader_message)

                                }
                                None => {
                                    continue;
                                }

                            },
                            Err(e) => {
                                return Err(e);
                            }
                            
                }
            }

   return Ok(Some(re));
}


pub fn get_detail_account_group_equity(
    pool: web::Data<Pool>,
    group_id: u64
) -> Result<Option<Vec<GroupEquitysProRes>>> {
    let mut conn = pool.get_conn().unwrap();
    let mut re: Vec<GroupEquitysProRes> = Vec::new();
    let value = &format!("select * from group_tra where group_id = {}", group_id);
    let tra_data = conn.query_map(
        value, 
        |(id, group_id, tra_id)| { GroupTra{id, group_id, tra_id}} 
    ).unwrap();

    for tra_id in tra_data{
        let values = &format!("select * from bian_15m_equity where name={} order by time", tra_id.tra_id);
        let account_data = conn.query_map(
        values, 
            |(id,
                name,
                equity,
                time,
                r#type)| { BybitEquity {
                    id,
                          name,
                          
                          time,
                          equity,
                          r#type
                }}
        );

        
        match account_data {
            Ok(equitys) => {
                println!("获取到的权益数据{}", equitys.len() / 4);
                let len = (equitys.len() + 5) / 4;
                for i in 0..len{
                    if i * 4 < equitys.len() {
                        let times = &equitys[i * 4].time;
                    let new_time = times.clone();
                    println!("数据{}", new_time);
                    let equitya = &equitys[i * 4].equity;
                    let new_equity = equitya.clone();
                    let status = &equitys[i * 4].r#type;
                    let new_status = status.clone();


                    re.push(GroupEquitysProRes {
                        name: equitys[i * 4].name,
                        time: new_time,
                        equity: new_equity,
                        r#type: new_status,
                    })

                    }
                    
                }

            }
            Err(_) => {
                
            }
            
        }
                        
    }

   return Ok(Some(re));
}
                








// 添加账号组名称
pub fn add_account_group(pool: web::Data<Pool>, name: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"INSERT INTO account_group (name)
        VALUES (:name)",
        params! {
            "name" => name,
        },
    );
    match res {
        Ok(()) => {
            return true;
        }
        Err(e) => {
            return false;
        }
    }
}





// 添加账户组
pub fn insert_group_tra(pool: web::Data<Pool>, name: &str, tra: Vec<u64>) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select group_id from account_group where name = :name",
        params! {
            "name" => name
        },
    );
    match res {
        Ok(group_ids) => {
            for group_id in group_ids {
                let mut conn = pool.get_conn().unwrap();
                for tra_id  in &tra {
                    let tra = conn.exec_drop(
                        r"insert into group_tra (group_id, tra_id) values (:group_id, :tra_id)",
                        params! {
                            "group_id" => group_id,
                            "tra_id" => tra_id
                        },
                    );

                    match tra {
                        Ok(()) => {
                            continue;
                        }
                        Err(e) => {
                            return false;
                        }
                    }
                }
            }
            return true;
        }
        Err(_e) => return false,
    }
}



// 删除此账号权限
pub fn delete_account_tra(pool: web::Data<Pool>, account_id: &u64, tra_id: Vec<u64> ) -> bool {
    let mut conn = pool.get_conn().unwrap();
    for tra in &tra_id {
        let res = conn.exec_drop(
            r"delete from test_acc_tra where acc_id=:acc_id and tra_id=:tra_id",
            params! {
                "tra_id" => tra,
                "acc_id" => account_id
            },
        );

        match res {
            Ok(()) => {
                continue;
            }
            Err(e) => {
                return false;
            }
        }
    }
    return true;
}


// 添加账号组权限
pub fn insert_acc_group(pool: web::Data<Pool>, name: &str, account_id: &u64) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select group_id from account_group where name = :name",
        params! {
            "name" => name
        },
    );
    match res {
        Ok(group_ids) => {
            for group_id in group_ids {
                let mut conn = pool.get_conn().unwrap();
                    let tra = conn.exec_drop(
                        r"insert into acc_group (acc_id, group_id) values (:acc_id, :group_id)",
                        params! {
                            "acc_id" => account_id,
                            "group_id" => group_id
                        },
                    );

                    match tra {
                        Ok(()) => {
                            continue;
                        }
                        Err(_e) => {
                            return false;
                        }
                    }
            }
            return true;
        }
        Err(_e) => return false,
    }
}



pub fn insert_traders_wx_notices(pool: web::Data<Pool>, tra_id: &str, wx_hook: &str, wx_name: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select id from notices where tra_id = :tra_id", 
        params! {
            "tra_id" => tra_id
        },
    );
    match res {
        Ok(c) => {
            println!("找到了{:?}", c);
            for n in c{
                let result: Result<Vec<String>> = conn.exec(
                    r"select wx_hook from notices where tra_id = :tra_id and id = :id", 
        params! {
            "tra_id" => tra_id,
            "id" => n
        },
                );

                match result {
                    Ok(r) => {

                        for s in r{
                            if s.len() == 0{
                                let notice = conn.exec_drop(
                                    r"update notices set wx_hook = :wx_hook, wx_name = :wx_name where tra_id = :tra_id and id = :id", 
                                    params! {
                                        "wx_hook" => wx_hook,
                                        "wx_name" => wx_name,
                                        "tra_id" => tra_id,
                                        "id" => n
                                    }
                                );
                                match notice {
                                    Ok(c) => {
                                        continue;
                                    },
                                    Err(e) => {
                                        return false;
                                    }  
                                }; 
                            } else {
                                let notice = conn.exec_drop(
                                    r"insert into notices (tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name) values (:tra_id, :wx_hook, :wx_name, :slack_hook, :slack_name, :mess_hook, :mess_name)", 
                                    params! {
                                        "wx_hook" => wx_hook,
                                        "wx_name" => wx_name,
                                        "tra_id" => tra_id,
                                        "slack_hook" => "",
                                        "slack_name" => "",
                                        "mess_hook" => "",
                                        "mess_name" => ""
                                    }
                                );
                                match notice {
                                    Ok(c) => {
                                        continue;
                                    },
                                    Err(e) => {
                                        return false;
                                    }  
                                }; 
                            }

                        }

                    },
                    Err(e) => {
                        return false;
                        
                    }
                    
                }
                
            }
            return true;
        },
        Err(e) => {
            println!("没有找到{}", e);
            let notice = conn.exec_drop(
                r"insert into notices (tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name) values (:tra_id, :wx_hook, :wx_name, :slack_hook, :slack_name, :mess_hook, :mess_name)", 
                params! {
                    "wx_hook" => wx_hook,
                    "wx_name" => wx_name,
                    "tra_id" => tra_id,
                    "slack_hook" => "",
                    "slack_name" => "",
                    "mess_hook" => "",
                    "mess_name" => ""
                }
            );
            match notice {
                Ok(c) => {
                    return true;
                },
                Err(e) => {
                    return false;
                }  
            };
        }
    };

    // return true;
}



pub fn insert_traders_slack_notices(pool: web::Data<Pool>, tra_id: &str, slack_hook: &str, slack_name: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res: Result<Vec<u64>> = conn.exec(
        r"select id from notices where tra_id = :tra_id", 
        params! {
            "tra_id" => tra_id
        },
    );
    match res {
        Ok(c) => {
            println!("找到了{:?}", c);
            for n in c{
                let result: Result<Vec<String>> = conn.exec(
                    r"select slack_hook from notices where tra_id = :tra_id and id = :id", 
        params! {
            "tra_id" => tra_id,
            "id" => n
        },
                );

                match result {
                    Ok(r) => {

                        for s in r{
                            if s.len() == 0{
                                let notice = conn.exec_drop(
                                    r"update notices set slack_hook = :slack_hook, slack_name = :slack_name where tra_id = :tra_id and id = :id", 
                                    params! {
                                        "slack_hook" => slack_hook,
                                        "slack_name" => slack_name,
                                        "tra_id" => tra_id,
                                        "id" => n
                                    }
                                );
                                match notice {
                                    Ok(c) => {
                                        return true;
                                    },
                                    Err(e) => {
                                        return false;
                                    }  
                                }; 
                            } else {
                                let notice = conn.exec_drop(
                                    r"insert into notices (tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name) values (:tra_id, :wx_hook, :wx_name, :slack_hook, :slack_name, :mess_hook, :mess_name)", 
                                    params! {
                                        "wx_hook" => "",
                                        "wx_name" => "",
                                        "tra_id" => tra_id,
                                        "slack_hook" => slack_hook,
                                        "slack_name" => slack_name,
                                        "mess_hook" => "",
                                        "mess_name" => ""
                                    }
                                );
                                match notice {
                                    Ok(c) => {
                                        return true;;
                                    },
                                    Err(e) => {
                                        return false;
                                    }  
                                }; 
                            }

                        }

                    },
                    Err(e) => {
                        return false;
                        
                    }
                    
                }
                
            }
            return true;
        },
        Err(e) => {
            println!("没有找到{}", e);
            let notice = conn.exec_drop(
                r"insert into notices (tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name) values (:tra_id, :wx_hook, :wx_name, :slack_hook, :slack_name, :mess_hook, :mess_name)", 
                params! {
                    "wx_hook" => "",
                    "wx_name" => "",
                    "tra_id" => tra_id,
                    "slack_hook" => slack_hook,
                    "slack_name" => slack_name,
                    "mess_hook" => "",
                    "mess_name" => ""
                }
            );
            match notice {
                Ok(c) => {
                    return true;
                },
                Err(e) => {
                    return false;
                }  
            };
        }
    };

    // return true;
}


pub fn check_traders_slack_notices(pool: web::Data<Pool>, tra_id: &str, slack_hook: &str, slack_name: &str) -> Result<Vec<NoticesData>> {
    let mut notices: Vec<NoticesData> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from notices where tra_id = :tra_id and slack_hook = :slack_hook",
            params! {
                "tra_id" => tra_id,
                "slack_hook" => slack_hook,
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(id, tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name)| NoticesData {
                    id,
                    tra_id,
                    wx_hook,
                    wx_name,
                    slack_hook,
                    slack_name,
                    mess_hook,
                    mess_name,
                })
            },
        );


        match res {
            Ok(trader) => match trader {
                Some(tra) => {
                    notices.push(tra);
                }
                None => {
                    return Ok(notices);
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
        

    return Ok(notices);
}


pub fn check_traders_wx_notices(pool: web::Data<Pool>, tra_id: &str, wx_hook: &str, wx_name: &str) -> Result<Vec<NoticesData>> {
    let mut notices: Vec<NoticesData> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
        .exec_first(
            r"select * from notices where tra_id = :tra_id and wx_hook = :wx_hook",
            params! {
                "tra_id" => tra_id,
                "wx_hook" => wx_hook,
            },
        )
        .map(
            // Unpack Result
            |row| {
                row.map(|(id, tra_id, wx_hook, wx_name, slack_hook, slack_name, mess_hook, mess_name)| NoticesData {
                    id,
                    tra_id,
                    wx_hook,
                    wx_name,
                    slack_hook,
                    slack_name,
                    mess_hook,
                    mess_name,
                })
            },
        );


        match res {
            Ok(trader) => match trader {
                Some(tra) => {
                    notices.push(tra);
                }
                None => {
                    return Ok(notices);
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
        

    return Ok(notices);
}

pub fn get_one_traders(pool: web::Data<Pool>, tra_id: &str) -> Result<HashMap<String, Trader>> {
    let mut traders: HashMap<String, Trader> = HashMap::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
    .exec_first(
                r"select * from trader where tra_id = :tra_id",
                params! {
                        "tra_id" => tra_id
                        },
                )
                .map(
                        // Unpack Result
                        |row| {
                            row.map(
                                |(
                                    tra_id,
                                    tra_venue,
                                    tra_currency,
                                    api_key,
                                    secret_key,
                                    r#type,
                                    name,
                                    alarm,
                                    threshold,
                                    borrow,
                                    amount,
                                    wx_hook
                                )| Trader {
                                    tra_id,
                                    tra_venue,
                                    tra_currency,
                                    api_key,
                                    secret_key,
                                    r#type,
                                    name,
                                    alarm,
                                    threshold,
                                    borrow,
                                    amount,
                                    wx_hook
                                },
                            )
                        },
                    );
                    match res {
                        Ok(trader) => match trader {
                            Some(tra) => {
                                traders.insert(format!("{}", &tra.name), tra);
                            }
                            None => {
                                return Ok(traders);
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }
                    }
    return Ok(traders);
}


pub fn get_one_traders_message(pool: web::Data<Pool>, tra_id: &str) -> Result<Vec<TraderMessage>> {
    let mut traders: Vec<TraderMessage> = Vec::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
    .exec_first(
        r"select * from trader_message where tra_id = :tra_id order by id desc limit 1",
        params! {
            "tra_id" => tra_id
        },
    )
    .map(
        // Unpack Result
        |row| {
            row.map(|(id,
                tra_id,
                name,
                equity,
                leverage,
                position,
                open_order_amt,
                avaliable_balance,
                tra_venue,
                r#type, total_balance,)| TraderMessage {
                
                    id,
                    tra_id,
                    name,
                    equity,
                    leverage,
                    position,
                    open_order_amt,
                    avaliable_balance,
                    tra_venue,
                    r#type,
                    total_balance,
               
            })
        },
    );
                    match res {
                        Ok(trader) => match trader {
                            Some(tra) => {
                                traders.push(tra);
                            }
                            None => {
                                return Ok(traders);
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }
                    }
    return Ok(traders);
}


// 删除此通知方式
pub fn delete_wx_trader_notices(pool: web::Data<Pool>, tra_id:&str, hook: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update notices set wx_hook = :hook, wx_name = :wx_name where tra_id = :tra_id and wx_hook = :wx_hook",
        params! {
            "tra_id" => tra_id,
            "wx_hook" => hook,
            "hook" => "",
            "wx_name" => ""
        },
    );
    match res {
        Ok(()) => {
            return true;

        }
        Err(e) => {
            return false;
        }
    }
}



// 删除此通知方式
pub fn delete_slack_trader_notices(pool: web::Data<Pool>, tra_id:&str, hook: &str) -> bool {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update notices set slack_hook = :hook, slack_name = :slack_name where tra_id = :tra_id and slack_hook = :slack_hook",
        params! {
            "tra_id" => tra_id,
            "slack_hook" => hook,
            "hook" => "",
            "slack_name" => ""
        },
    );
    match res {
        Ok(()) => {
            return true;

        }
        Err(e) => {
            return false;
        }
    }
}




// 获取所有的api Key 数据（账户历史划转记录）
pub fn get_trader_incomes(pool: web::Data<Pool>) -> Result<HashMap<String, Trader>> {
    let mut incomes: HashMap<String, Trader> = HashMap::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        "select * from trader",
        |(tra_id, tra_venue, tra_currency, api_key, secret_key, r#type, name, alarm, threshold, borrow, amount, wx_hook)| {
            Trader{ tra_id, tra_venue,  tra_currency, api_key, secret_key,  r#type, name, alarm, threshold, borrow, amount, wx_hook }
        }
        ).unwrap(); 

        for i in res {
            let name = i.name.as_str();
            incomes.insert(String::from(name), i);
        }
                
        // match res {
        //     Ok(trader) => match trader {
        //         Some(tra) => {
        //             incomes.insert(format!("{}_{}", &tra.name, &tra.tra_id), tra);
        //         }
        //         None => {
        //             return Ok(incomes);
        //         }
        //     },
        //     Err(e) => {
        //         return Err(e);
        //     }
        // }
    // println!("incomes账户划转{:?}", incomes);
    return Ok(incomes);
}


// 获取账户划转记录
pub fn get_history_incomes(
    pool: web::Data<Pool>,
    tra_id: &str
) -> Result<Vec<HistoryIncomes>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    if tra_id == "account1" {
        let incomes = conn.query_map(
            "select * from incomes order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
        // println!("获取划转记录account1{:?}", incomes);
        return Ok(incomes);
    } else if tra_id == "account2" {
        let incomes = conn.query_map(
            "select * from incomes_2 order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
            // println!("获取划转记录account2{:?}", incomes);
        return Ok(incomes);

        
    } else if tra_id == "account3" {
        let incomes = conn.query_map(
            "select * from incomes_3 order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
            // println!("获取划转记录account3{:?}", incomes);
        return Ok(incomes);

    } else if tra_id == "account5" {
        let incomes = conn.query_map(
            "select * from incomes_4 order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
            // println!("获取划转记录account4{:?}", incomes);
        return Ok(incomes);

    } else if tra_id == "account6" {
        let incomes = conn.query_map(
            "select * from incomes_5 order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
            // println!("获取划转记录account5{:?}", incomes);
        return Ok(incomes);

    } else if tra_id == "account7" {
        let incomes = conn.query_map(
            "select * from incomes_6 order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
            // println!("获取划转记录account5{:?}", incomes);
        return Ok(incomes);

    } else{
        let incomes = conn.query_map(
            "select * from incomes_7 order by time desc",
            |(time, r#type, asset, amount, tran_id, status)| {
                HistoryIncomes{ time, r#type, asset, amount, tran_id, status }
            }
            ).unwrap();
            // println!("获取划转记录account6{:?}", incomes);
        return Ok(incomes);

    }
}

    
// 获取持仓数据和挂单数据和账户资产明细

pub fn get_trader_positions(pool: web::Data<Pool>, tra_id: &str) -> Result<HashMap<String, Trader>> {
    let mut traders: HashMap<String, Trader> = HashMap::new();
    let mut conn = pool.get_conn().unwrap();
    let res = conn
    .exec_first(
                r"select * from trader where tra_id = :tra_id",
                params! {
                        "tra_id" => tra_id
                        },
                )
                .map(
                        // Unpack Result
                        |row| {
                            row.map(
                                |(
                                    tra_id,
                                    tra_venue,
                                    tra_currency,
                                    api_key,
                                    secret_key,
                                    r#type,
                                    name,
                                    alarm,
                                    threshold,
                                    borrow,
                                    amount,
                                    wx_hook
                                )| Trader {
                                    tra_id,
                                    tra_venue,
                                    tra_currency,
                                    api_key,
                                    secret_key,
                                    r#type,
                                    name,
                                    alarm,
                                    threshold,
                                    borrow,
                                    amount,
                                    wx_hook

                                },
                            )
                        },
                    );
                    match res {
                        Ok(trader) => match trader {
                            Some(tra) => {
                                traders.insert(format!("{}_{}", &tra.name, &tra.tra_id), tra);
                            }
                            None => {
                                return Ok(traders);
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }
                    }
    // println!("history_trader{:?}", traders);
    return Ok(traders);
}

// 获取历史交易数据

pub fn get_history_trades(
    pool: web::Data<Pool>,
    tra_id: &str
) -> Result<Vec<Trade>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    if tra_id == "account1" {
        let trades = conn.query_map(
            "select * from trade_histories order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, is_maker, qty, quote_qty, tra_time, side, price, position_side, tra_commision, realized_pnl)| {
                Trade{th_id, tra_symbol, tra_order_id, is_maker, qty, quote_qty, tra_time, side, price, position_side, tra_commision, realized_pnl}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        return Ok(trades);
    } else if tra_id == "account3" {
        let trades = conn.query_map(
            "select * from trade_histories_3 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account3{:?}", trades);
        return Ok(trades);

        
    } else if tra_id == "account4" {
        let trades = conn.query_map(
            "select * from trade_histories_4 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);

    } else if tra_id == "account5" {
        let trades = conn.query_map(
            "select * from trade_histories_5 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);

    } else if tra_id == "account7" {
        let trades = conn.query_map(
            "select * from trade_histories_7 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);

    } else if tra_id == "account8" {
        let trades = conn.query_map(
            "select * from trade_histories_8 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);

    } else if tra_id == "account9" {
        let trades = conn.query_map(
            "select * from trade_histories_9 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);

    } else if tra_id == "account10" {
        let trades = conn.query_map(
            "select * from trade_histories_10 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);

    } else {
        let trades = conn.query_map(
            "select * from trate_histories_2 order by tra_time desc limit 1000",
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
                Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据account2{:?}", trades);
        return Ok(trades);
    }
}

// 获取前1000条订单成交数据bybit
pub fn get_history_bybit_trades(
    pool: web::Data<Pool>,
    tra_id: &str
) -> Result<Vec<BybitTrade>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    if tra_id == "account11" {
        let trades = conn.query_map(
            "select * from bybit_trader_histories order by time desc limit 1000",
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
                BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        return Ok(trades);
    } else {
        let trades = conn.query_map(
            "select * from bybit_trader_histories order by time desc limit 1000",
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
                BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        return Ok(trades);
    }
}

// 清除数据
pub fn clear_data(
    pool: web::Data<Pool>,
) -> Result<Vec<ClearData>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
        let equitys = conn.query_map(
            "select * from test_clear",
            |(id, name)| {
                ClearData{id, name}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        // println!("bian权益数据{:?}", equitys);
        return Ok(equitys);
}

// 获取权益数据
pub fn get_bybit_equity(
    pool: web::Data<Pool>,
    name: &str
) -> Result<Vec<BybitEquity>> {
    let mut conn = pool.get_conn().unwrap();
    let value = &format!("select * from bian_15m_equity where name = {}", name);
    // let mut re: Vec<Trade> = Vec::new();
        let equitys = conn.query_map(
            value,
            |(id, name, equity, time, r#type)| {
                BybitEquity{id, name, equity, time, r#type}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        // println!("equity权益数据{:?}", equitys);
        return Ok(equitys);
}

// 获取bian权益数据
pub fn get_bian_equity(
    pool: web::Data<Pool>,
    name: &str
) -> Result<Vec<BianEquity>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    let value = &format!("select * from bian_equity where name = {}", name);
        let equitys = conn.query_map(
            value,
            |(id, name, time, equity, r#type)| {
                BianEquity{id, name, time, equity, r#type}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        // println!("bian权益数据{:?}", equitys);
        return Ok(equitys);
}


// 获取后续的权益数据
pub fn get_total_bybit_equity(
    pool: web::Data<Pool>,
    name: &str
) -> Result<Vec<BybitEquity>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    let value = &format!("select * from bian_15m_equity where name = {}", name);
        let equitys = conn.query_map(
            value,
            |(id, name, time, equity, r#type)| {
                BybitEquity{id, name, time, equity, r#type}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        // println!("equity权益数据{:?}", equitys);
        return Ok(equitys);
}

// 获取bian权益数据
pub fn get_total_bian_equity(
    pool: web::Data<Pool>,
    name: &str
) -> Result<Vec<BianEquity>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    let value = &format!("select * from total_bian_equity where name = {}", name);
        let equitys = conn.query_map(
            value,
            |(id, name, time, equity, r#type)| {
                BianEquity{id, name, time, equity, r#type}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        // println!("bian权益数据{:?}", equitys);
        return Ok(equitys);
}


// 获取papi_bian权益数据
pub fn get_total_papi_bian_equity(
    pool: web::Data<Pool>,
    name: &str
) -> Result<Vec<BianEquity>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    let value = &format!("select * from total_papi_equity where name = {}", name);
        let equitys = conn.query_map(
            value,
            |(id, name, time, equity, r#type)| {
                BianEquity{id, name, time, equity, r#type}
            }
            ).unwrap();
        // println!("获取历史交易数据account1{:?}", trades);
        // println!("bian权益数据{:?}", equitys);
        return Ok(equitys);
}



// 获取持仓数据
pub fn get_history_positions(
    pool: web::Data<Pool>,
    tra_id: &str
) -> Result<Vec<Position>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    if tra_id == "account1" {
        let positions = conn.query_map(
            "select * from position_histories order by time desc",
            |(symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price)| {
                Position{symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price}
            }
            ).unwrap();
        // println!("获取历史仓位数据account1{:?}", positions);
        return Ok(positions);
    } else if tra_id == "account3" {
        let positions = conn.query_map(
            "select * from position_histories_3 order by time desc",
            |(symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price)| {
                Position{symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price}
            }
            ).unwrap();
        // println!("获取历史仓位数据account3{:?}", positions);
        return Ok(positions);
        
    } else if tra_id == "account4" {
        let positions = conn.query_map(
            "select * from position_histories_4 order by time desc",
            |(symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price)| {
                Position{symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price}
            }
            ).unwrap();
        // println!("获取历史仓位数据account2{:?}", positions);
        return Ok(positions);
        
    } else if tra_id == "account5" {
        let positions = conn.query_map(
            "select * from position_histories_5 order by time desc",
            |(symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price)| {
                Position{symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price}
            }
            ).unwrap();
        // println!("获取历史仓位数据account2{:?}", positions);
        return Ok(positions);
        
    } else if tra_id == "account6" {
        let positions = conn.query_map(
            "select * from position_histories_6 order by time desc",
            |(symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price)| {
                Position{symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price}
            }
            ).unwrap();
        // println!("获取历史仓位数据account2{:?}", positions);
        return Ok(positions);
        
    } else {
        let positions = conn.query_map(
            "select * from position_histories_2 order by time desc",
            |(symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price)| {
                Position{symbol, position_amt, position_side, time, entry_price, un_realized_profit, tra_id, leverage, mark_price}
            }
            ).unwrap();
        // println!("获取历史仓位数据account2{:?}", positions);
        return Ok(positions);
    }
    
}

// 获取净值数据
pub fn get_history_networth(
    pool: web::Data<Pool>
) -> Result<Vec<NetWorth>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
        let net_worths = conn.query_map(
            "select * from net_worth",
            |(time, net_worth)| {
                NetWorth{ time, net_worth }
            }
            ).unwrap();
        // println!("获取历史净值数据{:?}", net_worths);
        return Ok(net_worths);
}

// 获取权益数据（计算盈亏）
// 获取净值数据
pub fn get_equity(
    pool: web::Data<Pool>
) -> Result<Vec<Equity>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
        let equitys = conn.query_map(
            "select * from (select * from equity order by id desc limit 12) tbl1 order by id limit 7;",
            |(id, name, time, equity_eth, equity, prod_id)| {
                Equity{id, name, time, equity_eth, equity, prod_id }
            }
            ).unwrap();
        // println!("获取历史净值数据{:?}", equitys);
        return Ok(equitys);
}

// 获取账户交易额
pub fn get_trade_price(
    pool: web::Data<Pool>
) -> Result<Vec<NewPrice>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
        let net_worths = conn.query_map(
            "select * from trade_price",
            |(id, name, week_price, day_price)| {
                NewPrice{id, name, week_price, day_price }
            }
            ).unwrap();
        // println!("获取历史净值数据{:?}", net_worths);
        return Ok(net_worths);
}

// 根据trad_id 获取第一条数据
// pub fn get_one_history_trades(
//     pool: web::Data<Pool>,
//     end: &i64,
//     tra_id: &str
// ) -> Result<Option<u64>> {
//     let mut conn = pool.get_conn().unwrap();
//     // let mut re: Vec<Trade> = Vec::new();
//     if tra_id == "Angus" {
//         let trades: Result<u64> = conn.exec(
//             "select tra_time from trade_histories_3 order by tra_time limit 1",
//             params! {
//                 "id" => tra_id
//             },
//         );
//         // println!("获取历史交易数据angus{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "trader02" {
//         let trades: Result<u64> = conn.exec(
//             "select tra_time from trade_histories_4 order by tra_time limit 1",
//             params! {
//                 "id" => tra_id
//             },
//         );
//         // println!("获取历史交易数据angus{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "trader04" {
//         let value = &format!("select * from trade_histories_5 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "xh01_feng4_virtual" {
//         let value = &format!("select * from trade_histories_7 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "xh02_b20230524_virtual" {
//         let value = &format!("select * from trade_histories_8 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "xh03_feng3_virtual" {
//         let value = &format!("select * from trade_histories_9 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "xh04_20230524_virtual" {
//         let value = &format!("select * from trade_histories_10 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     } else if tra_id == "pca01" {
//         let value = &format!("select * from trade_pca01 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_Romysqlqty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     } else {
//         let value = &format!("select * from trade_histories_2 where tra_time >= {} and tra_time <= {}", start_time, end_time);
//         let trades = conn.query_map(
//             value,
//             |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
//                 Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
//             }
//             ).unwrap();
//         // println!("获取历史交易数据account3{:?}", trades);
//         return Ok(trades);
//     }
    
// }

// 根据日期获取今天的交易数据
pub fn get_date_new_trades(
    pool: web::Data<Pool>,
    start_time: &str,
    tra_id: &str
) -> Result<Vec<NewTrade>> {
    let mut conn = pool.get_conn().unwrap();
    let value = &format!("select th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side from bian_traders where tra_time >= {} and name = {} order by tra_time", start_time, tra_id);

        

        // let value = &format!("select * from bian_traders where tra_time >= {} and name = {}", start_time, tra_id);
        let trades = conn.query_map(
            value,
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, 
                position_side, price, qty, quote_qty, realized_pnl, side)| {
                NewTrade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据angus{:?}", trades);
        return Ok(trades);
    
}


// 根据日期获取今天的交易数据
pub fn get_date_new_bybit_trades(
    pool: web::Data<Pool>,
    start_time: &str,
    tra_id: &str
) -> Result<Vec<BybitNewTrade>> {
    let mut conn = pool.get_conn().unwrap();
    let value = &format!("select tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, name, is_maker, exec_id from new_bybit_traders where time >= {} and name = {} order by time", start_time, tra_id);

        

        // let value = &format!("select * from bian_traders where tra_time >= {} and name = {}", start_time, tra_id);
        let trades = conn.query_map(
            value,
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, name, is_maker, exec_id)| {
                    BybitNewTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, name, is_maker, exec_id}
            }
            ).unwrap();
        // println!("获取历史交易数据angus{:?}", trades);
        return Ok(trades);
    
}



// 根据日期获取账户交易历史的数据
pub fn get_date_history_trades(
    pool: web::Data<Pool>,
    start_time: &str,
    end_time: &str,
    tra_id: &str
) -> Result<Vec<NewTrade>> {
    let mut conn = pool.get_conn().unwrap();
    let value = &format!("select th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side from bian_traders where tra_time >= {} and tra_time <= {} and name = {}", start_time, end_time, tra_id);

        

        // let value = &format!("select * from bian_traders where tra_time >= {} and name = {}", start_time, tra_id);
        let trades = conn.query_map(
            value,
            |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, 
                position_side, price, qty, quote_qty, realized_pnl, side)| {
                NewTrade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
            }
            ).unwrap();
        // println!("获取历史交易数据angus{:?}", trades);
        return Ok(trades);
    // let mut re: Vec<Trade> = Vec::new();
    // if tra_id == "account1" {
    //    let value = &format!("select * from trade_histories where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //    let trades = conn.query_map(
    //     value,
    //     |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //         Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //     }
    //     ).unwrap();
    // // println!("获取历史交易数据account3{:?}", trades);
    // return Ok(trades);
    // } else if tra_id == "Angus" {
    //     let value = &format!("select * from trade_histories_3 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据angus{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "trader02" {
    //     let value = &format!("select * from trade_histories_4 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "trader04" {
    //     let value = &format!("select * from trade_histories_5 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "xh01_feng4_virtual" {
    //     let value = &format!("select * from trade_histories_7 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "xh02_b20230524_virtual" {
    //     let value = &format!("select * from trade_histories_8 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "xh03_feng3_virtual" {
    //     let value = &format!("select * from trade_histories_9 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value, 
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "xh04_20230524_virtual" {
    //     let value = &format!("select * from trade_histories_10 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "pca01" {
    //     let value = &format!("select * from trade_pca01 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else if tra_id == "zd01" {
    //     let value = &format!("select * from trader_zd01 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else {
    //     let value = &format!("select * from trade_histories_2 where tra_time >= {} and tra_time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side)| {
    //             Trade{th_id, tra_symbol, tra_order_id, tra_commision, tra_time, is_maker, position_side, price, qty, quote_qty, realized_pnl, side}
    //         }
    //         ).unwrap();
        // println!("获取历史交易数据account3{:?}", trades);
        // return Ok(trades);
    }
    




// 根据日期获取bybit账户交易历史的数据
pub fn get_date_bybit_history_trades(
    pool: web::Data<Pool>,
    start_time: &str,
    end_time: &str,
    tra_id: &str
) -> Result<Vec<BybitNewTrade>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    let value = &format!("select tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, name, is_maker, exec_id from new_bybit_traders where time >= {} and time <= {} and name = {}", start_time, end_time, tra_id);

        

        // let value = &format!("select * from bian_traders where tra_time >= {} and name = {}", start_time, tra_id);
        let trades = conn.query_map(
            value,
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, name, is_maker, exec_id)| {
                    BybitNewTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, name, is_maker, exec_id}
            }
            ).unwrap();
        // println!("获取历史交易数据angus{:?}", trades);
        return Ok(trades);
    // if tra_id == "mmteam1" {
    //     let value = &format!("select * from bybit_trader_histories where time >= {} and time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
    //             BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // } else {
    //     let value = &format!("select * from bybit_trader_histories where time >= {} and time <= {}", start_time, end_time);
    //     let trades = conn.query_map(
    //         value,
    //         |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
    //             BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
    //         }
    //         ).unwrap();
    //     // println!("获取历史交易数据account3{:?}", trades);
    //     return Ok(trades);
    // }
    
}



// 获取今天bybit交易数据
pub fn get_date_bybit_new_trades(
    pool: web::Data<Pool>,
    start_time: &str,
    tra_id: &str
) -> Result<Vec<BybitTrade>> {
    let mut conn = pool.get_conn().unwrap();
    // let mut re: Vec<Trade> = Vec::new();
    if tra_id == "mmteam1" {
        let value = &format!("select * from mmteam1_traders where time >= {}", start_time);
        let trades = conn.query_map(
            value,
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
                BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
            }
            ).unwrap();
        // println!("获取历史交易数据account3{:?}", trades);
        return Ok(trades);
    } else if tra_id == "hmaker05" {
        let value = &format!("select * from hmaker05_traders where time >= {}", start_time);
        let trades = conn.query_map(
            value,
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
                BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
            }
            ).unwrap();
        // println!("获取历史交易数据account3{:?}", trades);
        return Ok(trades);
    } else {
        let value = &format!("select * from hmaker05_traders where time >= {}", start_time);
        let trades = conn.query_map(
            value,
            |(tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type)| {
                BybitTrade{ tra_order_id, th_id, time, symbol, side, price, qty, quote_qty, commission, r#type }
            }
            ).unwrap();
        // println!("获取历史交易数据account3{:?}", trades);
        return Ok(trades);
    }
    
}

// 获取所有的产品列表
pub fn get_all_products(pool: web::Data<Pool>) -> Result<Vec<Product>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from test_products",
        |(prod_id, prod_name, weixin_id, prog_id)| {
            Product{ prod_id, prod_name, weixin_id, prog_id }
        }
    ).unwrap();
    return Ok(res);
}

// 获取挂单监控列表
pub fn get_alarm_open_orders(pool: web::Data<Pool>) -> Result<Vec<OpenOrders>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from open_orders",
        |(id, api_key, secret_key, name)| {
            OpenOrders{ id, api_key, secret_key, name }
        }
    ).unwrap();
    return Ok(res);
}

// 删除挂单监控
pub fn delect_orders(pool: web::Data<Pool>, name:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"delete from open_orders where name = :name",
        params! {
            "name" => name
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 添加挂单
pub fn add_orders(pool: web::Data<Pool>, name:&str, api_key: &str, secret_key:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"INSERT INTO open_orders (api_key, secret_key, name)
        VALUES (:api_key, :secret_key, :name)",
        params! {
            "api_key" => api_key,
            "secret_key" => secret_key,
            "name" => name
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 获取净头寸监控列表
pub fn get_alarm_positions(pool: web::Data<Pool>) -> Result<Vec<PositionsAlarm>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from position_alarm",
        |(id, api_key, secret_key, name, threshold)| {
            PositionsAlarm{ id, api_key, secret_key, name, threshold }
        }
    ).unwrap();
    return Ok(res);
}

// 删除净头寸监控
pub fn delect_positions(pool: web::Data<Pool>, name:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"delete from position_alarm where name = :name",
        params! {
            "name" => name
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 添加监控账号
pub fn add_positions(pool: web::Data<Pool>, name:&str, api_key: &str, secret_key:&str, threshold:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"INSERT INTO test (api_key, secret_key, name, threshold)
        VALUES (:api_key, :secret_key, :name, :threshold)",
        params! {
            "api_key" => api_key,
            "secret_key" => secret_key,
            "name" => name,
            "threshold" => threshold
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 更新净头寸监控中的阈值
pub fn update_positions(pool: web::Data<Pool>, name:&str, threshold:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update trader set amount = :amount where tra_id = :tra_id",
        params! {
            "tra_id" => name,
            "amount" => threshold
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 设置账户的份额
pub fn update_ori_balance(pool: web::Data<Pool>, tra_id:&str, ori_balance:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update  set ori_balance = :ori_balance where tra_id = :tra_id",
        params! {
            "tra_id" => tra_id,
            "ori_balance" => ori_balance
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 更新是否打开监控开关
pub fn update_alarms(pool: web::Data<Pool>, name:&str, alarm:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update trader set alarm = :alarm where tra_id = :tra_id",
        params! {
            "tra_id" => name,
            "alarm" => alarm
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 更新是否打开监控开关
pub fn update_threshold(pool: web::Data<Pool>, name:&str, threshold:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update trader set threshold = :threshold where tra_id = :tra_id",
        params! {
            "tra_id" => name,
            "threshold" => threshold
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 更新是否打开监控开关
pub fn update_currency(pool: web::Data<Pool>, name:&str, currency:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update trader set tra_currency = :tra_currency where tra_id = :tra_id",
        params! {
            "tra_id" => name,
            "tra_currency" => currency
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}


pub fn update_borrow(pool: web::Data<Pool>, name:&str, borrow:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"update trader set borrow = :borrow where tra_id = :tra_id",
        params! {
            "tra_id" => name,
            "borrow" => borrow
        },
    );
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// 删除账户
pub fn delect_accounts(pool: web::Data<Pool>, tra_id:&str, account_id: &str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"delete from trader where tra_id = :tra_id",
        params! {
            "tra_id" => tra_id
        },
    );
    match res {
        Ok(()) => {
            let account = conn.exec_drop(
                r"delete from test_acc_tra where tra_id = :tra_id and acc_id = :acc_id",
                params! {
                    "tra_id" => tra_id,
                    "acc_id" => account_id
                },
            );
            match account {
                Ok(()) => {
                    return Ok(());

                }
                Err(e) => {
                    return Err(e);
                }
            }
            
        }
        Err(e) => {
            return Err(e);
        }
    }
}


// 移除账户
pub fn remove_accounts(pool: web::Data<Pool>, tra_id:&str, account_id: &str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"delete from test_acc_tra where tra_id = :tra_id and acc_id = :acc_id",
        params! {
            "tra_id" => tra_id,
            "acc_id" => account_id
        },
    );
    match res {
        Ok(()) => {
            return Ok(());

        }
        Err(e) => {
            return Err(e);
        }
    }
}


// 添加账户
pub fn add_accounts(pool: web::Data<Pool>, name:&str, api_key: &str, secret_key:&str, alarm:&str, threshold:&str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.exec_drop(
        r"INSERT INTO trader (tra_venue, ori_balance, tra_currency, api_key, secret_key, other_keys, type, name, alarm, threshold)
        VALUES (:tra_venue, :ori_balance, :tra_currency, :api_key, :secret_key, :other_keys, :type, :name, :alarm, :threshold)",
        params! {
            "tra_venue" => "Binance",
            "ori_balance" => "500",
            "tra_currency" => "USDT", 
            "api_key" => api_key,
            "secret_key" => secret_key,
            "other_keys" => "",
            "type" => "Futures",
            "name" => name,
            "alarm" => alarm,
            "threshold" => threshold
        },
    );


    
    match res {
        Ok(()) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}


// 查找tra_id并添加到test_prod_tra表中

pub fn select_id(pool: web::Data<Pool>, name: &str, prod_id: &str) -> Result<()> {
    let mut conn = pool.get_conn().unwrap();

    // println!("传过来的参数{}", prod_id);

    let res:Result<Vec<u64>> = conn.exec(
        "select tra_id from trader where name = :name", 
        params! {
            "name" => name
        },
    );

    // println!("data数据数据数据数据{:?}", res);
    // match data {
    //     Ok(tra_id) => {
    //         println!("查询到的tra_id", tra_id);
    //         conn.exec(
    //             r"INSERT INTO tset_prod_tra (pt_id, prod_id, tra_id) VALUES (:pt_id, :prod_id, :tra_id)", 
    //             params! {
    //                 "prod_id" => prod_id,
    //                 "tra_id" => tra_id,
    //             },
    //         );
    //     }
    //     Err(_) => todo!(),
        
    // }

    
    match res {
        Ok(tra_id) => {
            // println!("tra_id{:?}", tra_id[0]);
            let _data = conn.exec_drop(
                r"INSERT INTO test_prod_tra (prod_id, tra_id) VALUES (:prod_id, :tra_id)", 
                params! {
                    "prod_id" => prod_id,
                    "tra_id" => tra_id[0]
                },
            );
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}


// 获取净值快照
pub fn get_net_worths(pool: web::Data<Pool>) -> Result<Vec<NetWorths>> {  
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from net_worth order by time desc",
        |(name, time, net_worth, prod_id)| {
            NetWorths{ name, time, net_worth, prod_id}
        }
    ).unwrap();
    return Ok(res);
}

// 获取权益快照
pub fn get_equitys(pool: web::Data<Pool>) -> Result<Vec<Equitys>> {
    let mut conn = pool.get_conn().unwrap();
    let res = conn.query_map(
        r"select * from equity order by time desc",
        |(id, name, time, equity_eth, equity, prod_id)| {
            Equitys{ name, time, equity_eth, equity, prod_id, id }
        }
    ).unwrap();
    return Ok(res);
}