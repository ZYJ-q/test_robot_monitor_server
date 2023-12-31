use chrono::Local;
use itertools::Itertools;
use log::error;
use serde_json::{Map,Value};
use std::collections::VecDeque;
use std::fs;
use chrono::{DateTime, NaiveDateTime, Utc};
use crate::models::db_data::BybitEquity;

use super::http_data::{Sub, PapiSub };
use super::base::venue_api::HttpVenueApi;

pub async fn get_account_sub(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
    borrow_currency: &str,
) -> Option<Sub> {
    if let Some(data) = http_api.account().await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // println!("账户信息binance{}", value);
        let assets = value.as_object().unwrap().get("assets").unwrap().as_array().unwrap();
        // let total_maint_margin: f64 = value.as_object().unwrap().get("totalMarginBalance").unwrap().as_str().unwrap().parse().unwrap();
        // if name == "trader02" {
        //     println!(" 账户数据{:?}", assets);
        // }
        let mut new_total_balance = 0.00;
        let mut total_margin_balance = 0.0;
        let mut new_total_equity = 0.00;
        let mut best_price = 0.00;
        let mut  total_maint_margin = 0.00;
        for a in assets {
            let obj = a.as_object().unwrap();
            let wallet_balance: f64 = obj.get("walletBalance").unwrap().as_str().unwrap().parse().unwrap();
            let symbol = obj.get("asset").unwrap().as_str().unwrap();
            let margin_balance:f64 = obj.get("marginBalance").unwrap().as_str().unwrap().parse().unwrap();



            if wallet_balance != 0.00 {
                if symbol == "USDT" || symbol == "USDC" || symbol == "BUSD" || symbol == "USDP"  {
                    println!("u本位的金额{}", margin_balance);
                    total_margin_balance += margin_balance;
                    println!("加完之后的金额{}", total_margin_balance);
                } else {
                    let asset = format!("{}USDT", symbol);
                    if let Some(data) = http_api.get_klines(&asset).await {
                        let v: Value = serde_json::from_str(&data).unwrap();
                        let price_obj = v.as_object().unwrap();
                        let price:f64 = price_obj.get("price").unwrap().as_str().unwrap().parse().unwrap();
                        let new_margin_balance = margin_balance * price;
                        println!("不是u本位的金额{}", new_margin_balance);
                        total_margin_balance += new_margin_balance;
                    }
                    
                }
            }
            


            

            if wallet_balance != 0.00 {
                let borrows: Vec<&str> = borrow_currency.split('-').collect();

                println!("处理之后的借贷币种{}", borrows[0]);
                
                if symbol == borrows[0]{
                    continue;
                }

                let cross_un_pnl: f64 = obj.get("crossUnPnl").unwrap().as_str().unwrap().parse().unwrap();
                let pnl = cross_un_pnl + wallet_balance;
                new_total_balance += wallet_balance;
                new_total_equity += pnl;
            }
        }
        
        // let net_worth = new_total_equity / origin_balance;
        
        // let total_balance: f64 = value
        //     .as_object()
        //     .unwrap()
        //     .get("totalWalletBalance")
        //     .unwrap()
        //     .as_str()
        //     .unwrap()
        //     .parse()
        //     .unwrap();
        // 可用余额
        let available_balance: f64 = value
            .as_object()
            .unwrap()
            .get("availableBalance")
            .unwrap()
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let positions = value.as_object().unwrap().get("positions").unwrap().as_array().unwrap();
        // let mut position: f64 = 0.0;
        let mut amts: f64 = 0.0;
        let mut prices: f64 = 0.0;

        // let mut short_position: f64 = 0.0;
        for p in positions {
            let obj = p.as_object().unwrap();
            let position_amt: f64 = obj.get("positionAmt").unwrap().as_str().unwrap().parse().unwrap();
            
            if position_amt == 0.0 {
                continue;
            } else {
                
            let symbol = obj.get("symbol").unwrap().as_str().unwrap();
            let symbols= &symbol[0..symbol.len()-4];
            // println!("symbols: {},symbol: {}", symbols, symbol);
            let sbol = format!("{}USDT", symbols);
            // println!("传过去的参数{}", sbol);
                if let Some(data) = http_api.get_klines(&sbol).await {
                    let v: Value = serde_json::from_str(&data).unwrap();
                    let price_obj = v.as_object().unwrap();

                    let price:f64 = price_obj.get("price").unwrap().as_str().unwrap().parse().unwrap();
                    let new_amt = position_amt * price;
                    amts += new_amt;
                    // prices = price;
                }
                total_maint_margin = amts / 2.00;
            }

        }


        let leverage = amts.abs() / total_margin_balance; // 杠杆率 = 仓位价值 / 本金（账户总金额 + 未实现盈亏）
       

        if let Some(data) = http_api.get_open_orders("none").await {
            let value: Value = serde_json::from_str(&data).unwrap();
            let open_orders = value.as_array().unwrap();
            let open_order = open_orders.len();

            // println!("当前挂单数量:{}, name:{}", open_order, name);

            return Some(Sub {
                id: String::from(id.to_string()),
                name: String::from(name),
                total_balance: format!("{}", total_margin_balance),
                total_equity: format!("{}", new_total_equity),
                leverage: format!("{}", leverage),
                position: format!("{}", amts),
                open_order_amt: format!("{}", open_order),
                available_balance: format!("{}", new_total_balance),
                tra_venue: format!("Binance"),
                r#type: format!("Futures"),
                total_maint_margin: format!("{}", total_maint_margin),
            });
        } else {
            error!("Can't get {} openOrders.", name);
        return None;
            
        }
    } else {
        error!("Can't get {} account.", name);
        return None;
    }
}



// 获取仓位明细
pub async fn get_account_positions(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value> {
    let mut history_positions: VecDeque<Value> = VecDeque::new();
    if let Some(data) = http_api.account().await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // let mut history_positions: Vec<http_data::Position> = Vec::new();
        
        let positions = value.as_object().unwrap().get("positions").unwrap().as_array().unwrap();
        for p in positions {
            let mut pos_obj: Map<String, Value> = Map::new();
            let obj = p.as_object().unwrap();
            let amt:f64= obj.get("positionAmt").unwrap().as_str().unwrap().parse().unwrap();
            if amt == 0.0 {
                continue;
            } else {
                let symbol = obj.get("symbol").unwrap().as_str().unwrap();
            let millis = obj.get("updateTime").unwrap().as_i64().unwrap();
            let datetime: DateTime<Utc> = DateTime::from_utc(
                NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                Utc,
            );
            let position_amt= obj.get("positionAmt").unwrap().as_str().unwrap();
            // info!("datetime: {}", datetime);
            let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
            let position_side = obj.get("positionSide").unwrap().as_str().unwrap();
            let entry_price = obj.get("entryPrice").unwrap().as_str().unwrap();
            let leverage = obj.get("leverage").unwrap().as_str().unwrap();
            let mark_price = obj.get("initialMargin").unwrap().as_str().unwrap();
            let unrealized_profit = obj.get("unrealizedProfit").unwrap().as_str().unwrap();

            pos_obj.insert(String::from("symbol"), Value::from(symbol));
            pos_obj.insert(String::from("position_amt"), Value::from(position_amt));
            pos_obj.insert(String::from("time"), Value::from(time));
            pos_obj.insert(String::from("position_side"), Value::from(position_side));
            pos_obj.insert(String::from("entry_price"), Value::from(entry_price));
            pos_obj.insert(String::from("leverage"), Value::from(leverage));
            pos_obj.insert(String::from("mark_price"), Value::from(mark_price));
            pos_obj.insert(String::from("unrealized_profit"), Value::from(unrealized_profit));
            // 新加的
            pos_obj.insert(String::from("id"), Value::from(id.to_string()));

            history_positions.push_back(Value::from(pos_obj));
            }
        }

        println!("仓位数据{:?}", history_positions);
            return history_positions.into();
    } else {
        error!("Can't get {} account.", name);
        return history_positions.into();
    }
}

// 获取挂单明细
pub async fn get_open_orders(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value> {
    let mut history_open_orders: VecDeque<Value> = VecDeque::new();
    if let Some(data) = http_api.get_open_orders("none").await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // let mut history_positions: Vec<http_data::Position> = Vec::new();
        
        let open_orders = value.as_array().unwrap();
        if open_orders.len() == 0 {
            println!("当前没有挂单")
        } else {
            for a in open_orders {
                let obj = a.as_object().unwrap();
                let mut open_order_object: Map<String, Value> = Map::new();
                let millis = obj.get("time").unwrap().as_i64().unwrap();
                let datetime: DateTime<Utc> = DateTime::from_utc(
                    NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                    Utc,
                );
                // info!("datetime: {}", datetime);
                let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
                
                let symbol = obj.get("symbol").unwrap().as_str().unwrap();
                let r#type = obj.get("type").unwrap().as_str().unwrap();
                let mut type_value = "";
                if r#type == "LIMIT" {
                    type_value = "限价单"
                } else if r#type == "MARKET" {
                    type_value = "市价单"
                } else if r#type == "STOP" {
                    type_value = "止损限价单"
                } else if r#type == "STOP_MARKET" {
                    type_value = "止盈市价单"
                } else if r#type == "TAKE_PROFIT" {
                    type_value = "止盈限价单"
                } else if r#type == "TAKE_PROFIT_MARKET" {
                    type_value = "止盈市价单"
                } else if r#type == "TRAILING_STOP_MARKET" {
                    type_value = "跟踪止损单" 
                }
                let side = obj.get("side").unwrap().as_str().unwrap();
                let price = obj.get("price").unwrap().as_str().unwrap();
                let orig_qty = obj.get("origQty").unwrap().as_str().unwrap();
                let executed_qty = obj.get("executedQty").unwrap().as_str().unwrap();
                let reduce_only = obj.get("reduceOnly").unwrap().as_bool().unwrap();
                open_order_object.insert(String::from("time"), Value::from(time.clone()));
                open_order_object.insert(String::from("name"), Value::from(name));
                open_order_object.insert(String::from("symbol"), Value::from(symbol));
                open_order_object.insert(String::from("type"), Value::from(type_value));
                open_order_object.insert(String::from("side"), Value::from(side));
                open_order_object.insert(String::from("price"), Value::from(price));
                open_order_object.insert(String::from("orig_qty"), Value::from(orig_qty));
                open_order_object.insert(String::from("executed_qty"), Value::from(executed_qty));
                open_order_object.insert(String::from("reduce_only"), Value::from(reduce_only));
                history_open_orders.push_back(Value::from(open_order_object));
                // println!("11111{}", vec[a]);
            }
        }
            return history_open_orders.into();
    } else {
        error!("Can't get {} account.", name);
        return history_open_orders.into();
    }
}

// 获取资产明细
pub async fn get_history_accounts(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value> {
    let mut history_assets: VecDeque<Value> = VecDeque::new();
    if let Some(data) = http_api.account().await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // let mut history_positions: Vec<http_data::Position> = Vec::new();
        
        let assets = value.as_object().unwrap().get("assets").unwrap().as_array().unwrap();
        for p in assets {
            let mut asset_obj: Map<String, Value> = Map::new();
            let obj = p.as_object().unwrap();
            let amt:f64= obj.get("walletBalance").unwrap().as_str().unwrap().parse().unwrap();
            if amt == 0.0 {
                continue;
            } else {
                let symbol = obj.get("asset").unwrap().as_str().unwrap();
            let wallet_balance= obj.get("walletBalance").unwrap().as_str().unwrap();
            let unrealized_profit = obj.get("unrealizedProfit").unwrap().as_str().unwrap();
            let margin_balance = obj.get("marginBalance").unwrap().as_str().unwrap();
            let available_balance = obj.get("availableBalance").unwrap().as_str().unwrap();

            asset_obj.insert(String::from("symbol"), Value::from(symbol));
            asset_obj.insert(String::from("wallet_balance"), Value::from(wallet_balance));
            asset_obj.insert(String::from("unrealized_profit"), Value::from(unrealized_profit));
            asset_obj.insert(String::from("margin_balance"), Value::from(margin_balance));
            asset_obj.insert(String::from("availableBalance"), Value::from(available_balance));
            // 新加的
            asset_obj.insert(String::from("id"), Value::from(id.to_string()));

            history_assets.push_back(Value::from(asset_obj));
            }
        }
            return history_assets.into();
    } else {
        error!("Can't get {} account.", name);
        return history_assets.into();
    }
}


// 获取划转明细
pub async fn get_income_data(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value>{
    
    let mut trade_incomes: VecDeque<Value> = VecDeque::new();

    // println!("传过来的数据,  name:{:?}, id:{:?}", name, id);
    // println!("当前时间戳{}", dt);

        if let Some(data) = http_api.get_income("").await {
            let value: Value = serde_json::from_str(&data).unwrap();
            let vec = value.as_object().unwrap();
                println!("划转记录数据{:?},账户名字{}", vec,name);
                let total = vec.get("total").unwrap().as_i64().unwrap();
            // println!("获取基金流水{:?}", value);
            if total == 0 {
                println!("没有划转记录")
            } else {
                let rows = vec.get("rows").unwrap().as_array().unwrap();
                for r in rows {
                    let obj = r.as_object().unwrap();
                    let mut income_object: Map<String, Value> = Map::new();
                    let asset = obj.get("asset").unwrap().as_str().unwrap();
                    let amount = obj.get("amount").unwrap().as_str().unwrap();
                    let tran_id = obj.get("tranId").unwrap().as_i64().unwrap();
                    let r#type = obj.get("type").unwrap().as_i64().unwrap();
                    let mut type_value = "";
                    if r#type == 1 {
                        type_value = "现货==>>USDT本位合约"
                    } else if r#type == 2 {
                        type_value = "USDT本位合约==>>现货"
                    } else if r#type == 3 {
                        type_value = "现货==>>币本位合约"
                    } else if r#type == 4 {
                        type_value = "币本位合约==>>现货"
                    }
                    let millis = obj.get("timestamp").unwrap().as_i64().unwrap();
                    let datetime: DateTime<Utc> = DateTime::from_utc(
                        NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                        Utc,
                    );
                    // info!("datetime: {}", datetime);
                    let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
                    let status = obj.get("status").unwrap().as_str().unwrap();
                    let mut status_value = "";
                    if status == "PENDING" {
                        status_value = "等待执行"
                    } else if status == "CONFIRMED" {
                        status_value = "成功划转"
                    } else if status == "FAILED" {
                        status_value = "执行失败"   
                    }

                    income_object.insert(String::from("time"), Value::from(time));
                    income_object.insert(String::from("type"), Value::from(type_value));
                    income_object.insert(String::from("asset"), Value::from(asset));
                    income_object.insert(String::from("amount"), Value::from(amount));
                    income_object.insert(String::from("tran_id"), Value::from(tran_id));
                    income_object.insert(String::from("status"), Value::from(status_value));
                    trade_incomes.push_back(Value::from(income_object));
                }
                
            }
            // println!("处理之后的账户资金账户数据{:?}", trade_incomes);
            return Vec::from(trade_incomes.clone());
        } else {
            error!("Can't get {} income.", name);
            return Vec::from(trade_incomes.clone());
        }
}


pub async fn get_klines_price(
    http_api: &Box<dyn HttpVenueApi>,
    symbol: &str,
) -> Option<Value> {
    if let Some(data) = http_api.get_klines(symbol).await {
        let v: Value = serde_json::from_str(&data).unwrap();
        return Some(v);
    } else {
        return None;
    }
}


// papi
// 账户信息
pub async fn get_papi_account_sub(
    http_api: &Box<dyn HttpVenueApi>,
    http_future_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
    borrow_currency: &str,
) -> Option<Sub> {

    if let Some (data) = http_api.account().await{
        let value: Value = serde_json::from_str(&data).unwrap();
        println!("账户信息{}", value);
        let total_available_balance = value.as_object().unwrap().get("totalAvailableBalance").unwrap().as_str().unwrap();
        let total_maint_margin: f64 = value.as_object().unwrap().get("accountMaintMargin").unwrap().as_str().unwrap().parse().unwrap();

        if let Some(data) = http_api.position_risk().await {
            let value: Value = serde_json::from_str(&data).unwrap();
            let assets = value.as_array().unwrap();
            let mut equity = 0.0;
            let mut total_margin_balance= 0.0;
            let mut borrow_balance = 0.0;


        for p in assets {
            let obj = p.as_object().unwrap();
            let amt:f64 = obj.get("totalWalletBalance").unwrap().as_str().unwrap().parse().unwrap();
            if amt == 0.0 {
                continue;
            } else {
                let symbol = obj.get("asset").unwrap().as_str().unwrap();
                let borrows: Vec<&str> = borrow_currency.split('-').collect();
                println!("处理之后的借贷币种{}", borrows[0]);
                if symbol == borrows[0] {
                    borrow_balance = amt;
                    continue;
                } else {
                    let unrealied_um:f64 = obj.get("umUnrealizedPNL").unwrap().as_str().unwrap().parse().unwrap();
                    let unrealied_cm:f64 = obj.get("cmUnrealizedPNL").unwrap().as_str().unwrap().parse().unwrap();
                    let unrealied = unrealied_cm + unrealied_um;
                    let total_equity = unrealied + amt;
                    equity += total_equity;
                    let symbols = format!("{}USDT", borrows[0]);

                    if let Some(data) = http_future_api.get_klines(&symbols).await {
                        let v: Value = serde_json::from_str(&data).unwrap();
                        let price_obj = v.as_object().unwrap();
                        let price:f64 = price_obj.get("price").unwrap().as_str().unwrap().parse().unwrap();
                        let new_balance = borrow_balance * price;
                        total_margin_balance += new_balance;
                    }
                }
            } 
        }





            if let Some(data) = http_api.position_um().await {
                let value: Value = serde_json::from_str(&data).unwrap();

            let positions = value.as_object().unwrap().get("positions").unwrap().as_array().unwrap();
        // let mut position: f64 = 0.0;
        let mut amts: f64 = 0.0;
        let mut prices: f64 = 0.0;
        let mut new_symbol = "";

        // let mut short_position: f64 = 0.0;
        for p in positions {
            let obj = p.as_object().unwrap();
            let position_amt: f64 = obj.get("positionAmt").unwrap().as_str().unwrap().parse().unwrap();
            
            if position_amt == 0.0 {
                continue;
            } else {
                println!("positions{:?}", obj);
                
            let symbol = obj.get("symbol").unwrap().as_str().unwrap();
            new_symbol= &symbol[0..symbol.len()-4];
            let sbol = format!("{}USDT", new_symbol);
            if let Some(data) = http_future_api.get_klines(&sbol).await {
                let v: Value = serde_json::from_str(&data).unwrap();
                let price_obj = v.as_object().unwrap();

                let price:f64 = price_obj.get("price").unwrap().as_str().unwrap().parse().unwrap();
                let new_amt = position_amt * price;
                amts += new_amt;
                // prices = price;
            }
            }

        }
        // let position = amts * prices;

        // println!("账户本金{}, 名字{}", equity, name);


        let leverage = amts.abs() / (total_margin_balance + equity); // 杠杆率 = 仓位价值 / 本金

            

            

        

        

        if let Some(data) = http_api.get_open_orders("none").await {
            let value: Value = serde_json::from_str(&data).unwrap();
            let open_orders = value.as_array().unwrap();
            let open_order = open_orders.len();

            // println!("当前挂单数量:{}, name:{}", open_order, name);

            return Some(Sub {
                id: String::from(id.to_string()),
                name: String::from(name),
                total_balance: format!("{}", total_margin_balance + equity),
                total_equity: format!("{}", equity),
                leverage: format!("{}", leverage),
                position: format!("{}", amts),
                open_order_amt: format!("{}", open_order),
                available_balance: format!("{}", total_available_balance),
                tra_venue: format!("Binance"),
                r#type: format!("Papi"),
                total_maint_margin: format!("{}", total_maint_margin),
                
            });
        } else {
            error!("Can't get {} openOrders.", name);
        return None;
            
        }
    }else {

        error!("Can't get {} positions_um.", name);
        return None;
        
    }
    } else {
        error!("Can't get {} positions.", name);
        return None;
        
    }
} else {
    error!("Can't get {} positions.", name);
        return None;
    
}
}


pub async fn get_papi_account_positions(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value> {
    let mut history_positions: VecDeque<Value> = VecDeque::new();
    if let Some(data) = http_api.position_um().await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // let mut history_positions: Vec<http_data::Position> = Vec::new();
        
        let positions = value.as_object().unwrap().get("positions").unwrap().as_array().unwrap();
        for p in positions {
            let mut pos_obj: Map<String, Value> = Map::new();
            let obj = p.as_object().unwrap();
            let amt:f64= obj.get("positionAmt").unwrap().as_str().unwrap().parse().unwrap();
            if amt == 0.0 {
                continue;
            } else {
                let symbol = obj.get("symbol").unwrap().as_str().unwrap();
            let millis = obj.get("updateTime").unwrap().as_i64().unwrap();
            let datetime: DateTime<Utc> = DateTime::from_utc(
                NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                Utc,
            );
            let position_amt= obj.get("positionAmt").unwrap().as_str().unwrap();
            // info!("datetime: {}", datetime);
            let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
            let position_side = obj.get("positionSide").unwrap().as_str().unwrap();
            let entry_price = obj.get("entryPrice").unwrap().as_str().unwrap();
            let leverage = obj.get("leverage").unwrap().as_str().unwrap();
            let mark_price = obj.get("initialMargin").unwrap().as_str().unwrap();
            let unrealized_profit = obj.get("unrealizedProfit").unwrap().as_str().unwrap();

            pos_obj.insert(String::from("symbol"), Value::from(symbol));
            pos_obj.insert(String::from("position_amt"), Value::from(position_amt));
            pos_obj.insert(String::from("time"), Value::from(time));
            pos_obj.insert(String::from("position_side"), Value::from(position_side));
            pos_obj.insert(String::from("entry_price"), Value::from(entry_price));
            pos_obj.insert(String::from("leverage"), Value::from(leverage));
            pos_obj.insert(String::from("mark_price"), Value::from(mark_price));
            pos_obj.insert(String::from("unrealized_profit"), Value::from(unrealized_profit));
            // 新加的
            pos_obj.insert(String::from("id"), Value::from(id.to_string()));

            history_positions.push_back(Value::from(pos_obj));
            }
        }
            return history_positions.into();
    } else {
        error!("Can't get {} account.", name);
        return history_positions.into();
    }
}

// 获取挂单明细
pub async fn get_papi_open_orders(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value> {
    let mut history_open_orders: VecDeque<Value> = VecDeque::new();
    if let Some(data) = http_api.get_open_orders("none").await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // let mut history_positions: Vec<http_data::Position> = Vec::new();
        println!("挂单数据{}", value);
        
        let open_orders = value.as_array().unwrap();
        if open_orders.len() == 0 {
            println!("当前没有挂单")
        } else {
            for a in open_orders {
                let obj = a.as_object().unwrap();
                let mut open_order_object: Map<String, Value> = Map::new();
                let millis = obj.get("time").unwrap().as_i64().unwrap();
                let datetime: DateTime<Utc> = DateTime::from_utc(
                    NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                    Utc,
                );
                // info!("datetime: {}", datetime);
                let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
                
                let symbol = obj.get("symbol").unwrap().as_str().unwrap();
                let r#type = obj.get("type").unwrap().as_str().unwrap();
                let mut type_value = "";
                if r#type == "LIMIT" {
                    type_value = "限价单"
                } else if r#type == "MARKET" {
                    type_value = "市价单"
                } else if r#type == "STOP" {
                    type_value = "止损限价单"
                } else if r#type == "STOP_MARKET" {
                    type_value = "止盈市价单"
                } else if r#type == "TAKE_PROFIT" {
                    type_value = "止盈限价单"
                } else if r#type == "TAKE_PROFIT_MARKET" {
                    type_value = "止盈市价单"
                } else if r#type == "TRAILING_STOP_MARKET" {
                    type_value = "跟踪止损单" 
                }
                let side = obj.get("side").unwrap().as_str().unwrap();
                let price = obj.get("price").unwrap().as_str().unwrap();
                let orig_qty = obj.get("origQty").unwrap().as_str().unwrap();
                let executed_qty = obj.get("executedQty").unwrap().as_str().unwrap();
                let reduce_only = obj.get("reduceOnly").unwrap().as_bool().unwrap();
                open_order_object.insert(String::from("time"), Value::from(time.clone()));
                open_order_object.insert(String::from("name"), Value::from(name));
                open_order_object.insert(String::from("symbol"), Value::from(symbol));
                open_order_object.insert(String::from("type"), Value::from(type_value));
                open_order_object.insert(String::from("side"), Value::from(side));
                open_order_object.insert(String::from("price"), Value::from(price));
                open_order_object.insert(String::from("orig_qty"), Value::from(orig_qty));
                open_order_object.insert(String::from("executed_qty"), Value::from(executed_qty));
                open_order_object.insert(String::from("reduce_only"), Value::from(reduce_only));
                history_open_orders.push_back(Value::from(open_order_object));
                // println!("11111{}", vec[a]);
            }
        }
            return history_open_orders.into();
    } else {
        error!("Can't get {} account.", name);
        return history_open_orders.into();
    }
}



// 获取papi资产明细
pub async fn get_papi_history_accounts(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value> {
    let mut history_assets: VecDeque<Value> = VecDeque::new();
    if let Some(data) = http_api.position_risk().await {
        let value: Value = serde_json::from_str(&data).unwrap();
        // let mut history_positions: Vec<http_data::Position> = Vec::new();
        
        let assets = value.as_array().unwrap();
        for p in assets {
            let mut asset_obj: Map<String, Value> = Map::new();
            let obj = p.as_object().unwrap();
            let amt:f64= obj.get("totalWalletBalance").unwrap().as_str().unwrap().parse().unwrap();
            if amt == 0.0 {
                continue;
            } else {
                let symbol = obj.get("asset").unwrap().as_str().unwrap();
                    let wallet_balance= obj.get("totalWalletBalance").unwrap().as_str().unwrap();
            let unrealized_profit_um:f64 = obj.get("umUnrealizedPNL").unwrap().as_str().unwrap().parse().unwrap();
            let unrealized_profit_cm: f64 = obj.get("cmUnrealizedPNL").unwrap().as_str().unwrap().parse().unwrap(); 
            let unrealized_profit = unrealized_profit_cm + unrealized_profit_um;
            let margin_balance = amt + unrealized_profit;
            let available_balance = obj.get("crossMarginFree").unwrap().as_str().unwrap();

            asset_obj.insert(String::from("symbol"), Value::from(symbol));
            asset_obj.insert(String::from("wallet_balance"), Value::from(wallet_balance));
            asset_obj.insert(String::from("unrealized_profit"), Value::from(unrealized_profit));
            asset_obj.insert(String::from("margin_balance"), Value::from(margin_balance));
            asset_obj.insert(String::from("availableBalance"), Value::from(available_balance));
            // 新加的
            asset_obj.insert(String::from("id"), Value::from(id.to_string()));

            history_assets.push_back(Value::from(asset_obj));
            }
        }
            return history_assets.into();
    } else {
        error!("Can't get {} account.", name);
        return history_assets.into();
    }
}

// 获取papi划转明细
pub async fn get_papi_income_data(
    http_api: &Box<dyn HttpVenueApi>,
    name: &str,
    id: &u64,
) -> Vec<Value>{
    
    let mut trade_incomes: VecDeque<Value> = VecDeque::new();

    // println!("传过来的数据,  name:{:?}, id:{:?}", name, id);
    // println!("当前时间戳{}", dt);

        if let Some(data) = http_api.get_income("TRANSFER").await {
            let value: Value = serde_json::from_str(&data).unwrap();
            println!("获取基金流水{:?}", value);
            let incomes = value.as_array().unwrap();
            if incomes.len() == 0 {

            }
            return Vec::from(trade_incomes.clone());
        } else {
            error!("Can't get {} income.", name);
            return Vec::from(trade_incomes.clone());
        }
}



// // 处理equity数据
// pub fn get_group_equitys(
//     equity: Option<Vec<BybitEquity>>
// ) -> Vec<Value>{

//     let mut trade_incomes: VecDeque<Value> = VecDeque::new();
    
//     println!("{:?}", equity);

//     match equity {
//         Some(equitys) => {

//             for i in equitys{
//                 let time = i.time;
//                 let new_time = &time[1..time.len()-1];

//                 if 
//             }


//             return Vec::from(trade_incomes.clone());

//         },
//         None => {
//             return Vec::from(trade_incomes.clone());

//         },
        
//     }

    
// }
