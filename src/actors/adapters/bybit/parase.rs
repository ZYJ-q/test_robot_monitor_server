
use chrono::Local;
use log::error;
use serde_json::{Map,Value};
use std::collections::VecDeque;
use std::fs;
use chrono::{DateTime, NaiveDateTime, Utc};
use super::http_data::ByBitSub;
use super::base::venue_api::HttpVenueApi;

pub async fn get_account_bybit(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
  alarm: &str,
) -> Option<ByBitSub> {
  if let Some(data) = http_api.account().await {
      let value: Value = serde_json::from_str(&data).unwrap();
      
      let assets = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
      let list = assets.get("list").unwrap().as_array().unwrap();
      let mut wallet_balance = "";
      let mut equity = 0.0;
      for a in list {
          let obj = a.as_object().unwrap();
          wallet_balance = obj.get("totalWalletBalance").unwrap().as_str().unwrap();
          equity = obj.get("totalEquity").unwrap().as_str().unwrap().parse().unwrap();

      }

      let net_worth = equity / origin_balance;
      let category_spot = "spot";
      let category_lear = "linear";


     
      if let Some(data) = http_api.position(category_lear).await {
        println!("持仓数据{}", data);
        let value: Value = serde_json::from_str(&data).unwrap();
        let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
        let positions = result.get("list").unwrap().as_array().unwrap();
      // let mut position: f64 = 0.0;
      
      let mut amts: f64 = 0.0;
      // let mut short_position: f64 = 0.0;
      for p in positions {
          let obj = p.as_object().unwrap();
          let po = obj.get("size").unwrap().as_str().unwrap();
          println!("持仓数量{}", po);
          let position_amt: f64 = obj.get("size").unwrap().as_str().unwrap().parse().unwrap();
          let price: f64 = obj.get("markPrice").unwrap().as_str().unwrap().parse().unwrap();
          let pos_price = position_amt * price;
          amts += pos_price;
      }
      // let position = amts * prices;


      let leverage = amts.abs() / equity; // 杠杆率 = 仓位价值 / 本金（账户总金额 + 未实现盈亏）

      if let Some(data) = http_api.get_open_orders(category_lear).await {
        println!("挂单数据{}", data);
          let value: Value = serde_json::from_str(&data).unwrap();
          let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
          let list = result.get("list").unwrap().as_array().unwrap();
          let open_order = list.len();

          println!("权益{}, 杠杆率{}, 净头寸{}, 挂单数量{}, 净值{}, 可用余额{}", equity, leverage, amts, open_order, net_worth, wallet_balance);

          return Some(ByBitSub {
              id: String::from(id.to_string()),
              name: String::from(name),
              total_equity: format!("{}", equity),
              leverage: format!("{}", leverage),
              position: format!("{}", amts),
              open_order_amt: format!("{}", open_order),
              net_worth: format!("{}", net_worth),
              available_balance: format!("{}", wallet_balance),
          });
      } else {
          error!("Can't get {} openOrders.", name);
          None   
      }
    } else {
        error!("Can't get {} position.", name);
        return Some(ByBitSub {
          id: String::from(id.to_string()),
          name: String::from(name),
          total_equity: format!("{}", equity),
          leverage: format!("{}", 0),
          position: format!("{}", 0),
          open_order_amt: format!("{}", 0),
          net_worth: format!("{}", net_worth),
          available_balance: format!("{}", wallet_balance),
      });
        
    }
  } else {
      error!("Can't get {} account.", name);
      return None;
  }
}




// 获取bybit期货仓位明细
pub async fn get_futures_bybit_positions(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
) -> Vec<Value> {
      let category_lear = "linear";
  let mut history_positions: VecDeque<Value> = VecDeque::new();
  if let Some(data) = http_api.position(category_lear).await {
      let value: Value = serde_json::from_str(&data).unwrap();
      // let mut history_positions: Vec<http_data::Position> = Vec::new();
      println!("bybit账户仓位数据{:?}", value);

      let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
      let list = result.get("list").unwrap().as_array().unwrap();
      for p in list {
          let mut pos_obj: Map<String, Value> = Map::new();
          let obj = p.as_object().unwrap();
          let amt:f64= obj.get("size").unwrap().as_str().unwrap().parse().unwrap();
          if amt == 0.0 {
              continue;
          } else {
              let symbol = obj.get("symbol").unwrap().as_str().unwrap();
          let millis: i64 = obj.get("updateTime").unwrap().as_str().unwrap().parse().unwrap();
          let datetime: DateTime<Utc> = DateTime::from_utc(
              NaiveDateTime::from_timestamp_millis(millis).unwrap(),
              Utc,
          );
          let position_amt= obj.get("size").unwrap().as_str().unwrap();
          // info!("datetime: {}", datetime);
          let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
          let position_side = obj.get("side").unwrap().as_str().unwrap();
          let entry_price = obj.get("avgPrice").unwrap().as_str().unwrap();
          let leverage = obj.get("leverage").unwrap().as_str().unwrap();
          let mark_price = obj.get("markPrice").unwrap().as_str().unwrap();
          let unrealized_profit = obj.get("unrealisedPnl").unwrap().as_str().unwrap();

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



// 获取bybit现货仓位明细
pub async fn get_spot_bybit_positions(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
) -> Vec<Value> {
      let category_lear = "spot";
  let mut history_positions: VecDeque<Value> = VecDeque::new();
  if let Some(data) = http_api.position(category_lear).await {
      let value: Value = serde_json::from_str(&data).unwrap();
      // let mut history_positions: Vec<http_data::Position> = Vec::new();
      let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
      let list = result.get("list").unwrap().as_array().unwrap();
      for p in list {
          let mut pos_obj: Map<String, Value> = Map::new();
          let obj = p.as_object().unwrap();
          let amt:f64= obj.get("size").unwrap().as_str().unwrap().parse().unwrap();
          if amt == 0.0 {
              continue;
          } else {
              let symbol = obj.get("symbol").unwrap().as_str().unwrap();
          let millis: i64 = obj.get("updateTime").unwrap().as_str().unwrap().parse().unwrap();
          let datetime: DateTime<Utc> = DateTime::from_utc(
              NaiveDateTime::from_timestamp_millis(millis).unwrap(),
              Utc,
          );
          let position_amt= obj.get("size").unwrap().as_str().unwrap();
          // info!("datetime: {}", datetime);
          let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
          let position_side = obj.get("side").unwrap().as_str().unwrap();
          let entry_price = obj.get("avgPrice").unwrap().as_str().unwrap();
          let leverage = obj.get("leverage").unwrap().as_str().unwrap();
          let mark_price = obj.get("markPrice").unwrap().as_str().unwrap();
          let unrealized_profit = obj.get("unrealisedPnl").unwrap().as_str().unwrap();

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



// 获取bybit期货挂单明细
pub async fn get_bybit_futures_open_orders(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
) -> Vec<Value> {
  let category_lear = "linear";
  let mut history_open_orders: VecDeque<Value> = VecDeque::new();
  if let Some(data) = http_api.get_open_orders(category_lear).await {
      let value: Value = serde_json::from_str(&data).unwrap();
      println!("bybit挂单数据{:?}", value);
      // let mut history_positions: Vec<http_data::Position> = Vec::new();
      let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
      let open_orders = result.get("list").unwrap().as_array().unwrap();

      if open_orders.len() == 0 {
          println!("当前没有挂单")
      } else {
          for a in open_orders {
              let obj = a.as_object().unwrap();
              let mut open_order_object: Map<String, Value> = Map::new();
              let millis:i64 = obj.get("createdTime").unwrap().as_str().unwrap().parse().unwrap();
              let datetime: DateTime<Utc> = DateTime::from_utc(
                  NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                  Utc,
              );
              // info!("datetime: {}", datetime);
              let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
              
              let symbol = obj.get("symbol").unwrap().as_str().unwrap();
              let r#type = obj.get("orderType").unwrap().as_str().unwrap();
              let mut type_value = "";
              if r#type == "Limit" {
                  type_value = "限价单"
              } else if r#type == "Market" {
                  type_value = "市价单"
              }
              let side = obj.get("side").unwrap().as_str().unwrap();
              let price = obj.get("price").unwrap().as_str().unwrap();
              let orig_qty = obj.get("qty").unwrap().as_str().unwrap();
              let executed_qty = obj.get("cumExecQty").unwrap().as_str().unwrap();
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


// 获取bybit现货挂单明细
pub async fn get_bybit_spot_open_orders(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
) -> Vec<Value> {
  let category_lear = "spot";
  let mut history_open_orders: VecDeque<Value> = VecDeque::new();
  if let Some(data) = http_api.get_open_orders(category_lear).await {
      let value: Value = serde_json::from_str(&data).unwrap();
      // let mut history_positions: Vec<http_data::Position> = Vec::new();
      let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
      let open_orders = result.get("list").unwrap().as_array().unwrap();

      if open_orders.len() == 0 {
          println!("当前没有挂单")
      } else {
          for a in open_orders {
              let obj = a.as_object().unwrap();
              let mut open_order_object: Map<String, Value> = Map::new();
              let millis:i64 = obj.get("createdTime").unwrap().as_str().unwrap().parse().unwrap();
              let datetime: DateTime<Utc> = DateTime::from_utc(
                  NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                  Utc,
              );
              // info!("datetime: {}", datetime);
              let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
              
              let symbol = obj.get("symbol").unwrap().as_str().unwrap();
              let r#type = obj.get("orderType").unwrap().as_str().unwrap();
              let mut type_value = "";
              if r#type == "Limit" {
                  type_value = "限价单"
              } else if r#type == "Market" {
                  type_value = "市价单"
              }
              let side = obj.get("side").unwrap().as_str().unwrap();
              let price = obj.get("price").unwrap().as_str().unwrap();
              let orig_qty = obj.get("qty").unwrap().as_str().unwrap();
              let executed_qty = obj.get("cumExecQty").unwrap().as_str().unwrap();
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

// 获取bybit资产明细
pub async fn get_bybit_history_accounts(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
) -> Vec<Value> {
  let mut history_assets: VecDeque<Value> = VecDeque::new();
  if let Some(data) = http_api.account().await {
      let value: Value = serde_json::from_str(&data).unwrap();
      // let mut history_positions: Vec<http_data::Position> = Vec::new();
      let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
      let list = result.get("list").unwrap().as_array().unwrap();
      
      
      for p in list {
          let obj = p.as_object().unwrap();
          let assets = obj.get("coin").unwrap().as_array().unwrap();
          for c in assets {
            let mut asset_obj: Map<String, Value> = Map::new();
            let objs = c.as_object().unwrap();
            let amt:f64= objs.get("walletBalance").unwrap().as_str().unwrap().parse().unwrap();
          if amt == 0.0 {
              continue;
          } else {
              let symbol = objs.get("coin").unwrap().as_str().unwrap();
          let wallet_balance= objs.get("walletBalance").unwrap().as_str().unwrap();
          let unrealized_profit = objs.get("unrealisedPnl").unwrap().as_str().unwrap(); 
          let margin_balance = objs.get("usdValue").unwrap().as_str().unwrap();
          let available_balance = objs.get("availableToWithdraw").unwrap().as_str().unwrap();

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
      }
          return history_assets.into();
  } else {
      error!("Can't get {} account.", name);
      return history_assets.into();
  }
}


// 获取bybit划转明细
pub async fn get_income_bybit_data(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
) -> Vec<Value>{
  
  let mut trade_incomes: VecDeque<Value> = VecDeque::new();
  let mut to = "";
  let mut from = "";
  let mut status = "";

      if let Some(data) = http_api.get_income().await {
          let value: Value = serde_json::from_str(&data).unwrap();
          println!("划转明细{:?}", value);
          let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
          let list = result.get("list").unwrap().as_array().unwrap();
          for i in list {
              let mut income_obj: Map<String, Value> = Map::new();
              let obj = i.as_object().unwrap(); // positionAmt positionSide
              
              let bybit_status = obj.get("status").unwrap().as_str().unwrap();
              let from_account_type = obj.get("fromAccountType").unwrap().as_str().unwrap();
              let to_account_type = obj.get("toAccountType").unwrap().as_str().unwrap();

              if bybit_status == "SUCCESS" {
                status = "划转成功"
              } else if bybit_status  == "PENDING" {
                status = "正在划转中"
              } else if bybit_status == "FAILED" {
                status = "划转失败"
              }

              if from_account_type == "FUND" {
                from = "资金账户"
              } else if from_account_type == "CONTRACT" {
                from = "合约账户"
              } else if from_account_type == "SPOT" {
                from = "现货账户"
              } else if from_account_type == "OPTION" {
                from = "USDC合约账户"
              } else if from_account_type == "UNIFIED" {
                from = "统一账户"  
              }


              if to_account_type == "FUND" {
                to = "资金账户"
              } else if to_account_type == "CONTRACT" {
                to = "合约账户"
              } else if to_account_type == "SPOT" {
                to = "现货账户"
              } else if to_account_type == "OPTION" {
                to = "USDC合约账户"
              } else if to_account_type == "UNIFIED" {
                to = "统一账户"  
              }



                  let millis:i64 = obj.get("timestamp").unwrap().as_str().unwrap().parse().unwrap();
                  let datetime: DateTime<Utc> = DateTime::from_utc(
                    NaiveDateTime::from_timestamp_millis(millis).unwrap(),
                    Utc,
                );
                // info!("datetime: {}", datetime);
                let time = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
                  let amount:f64 = obj.get("amount").unwrap().as_str().unwrap().parse().unwrap();
                  let asset = obj.get("coin").unwrap().as_str().unwrap();
                  income_obj.insert(String::from("from"), Value::from(from));
                  income_obj.insert(String::from("to"), Value::from(to));
                  income_obj.insert(String::from("name"), Value::from(name));
                  income_obj.insert(String::from("id"), Value::from(id.to_string()));
                  income_obj.insert(String::from("time"), Value::from(time));
                  income_obj.insert(String::from("amount"), Value::from(amount));
                  income_obj.insert(String::from("asset"), Value::from(asset));
                  income_obj.insert(String::from("status"), Value::from(status));
                  trade_incomes.push_back(Value::from(income_obj)); 
          }
          // println!("处理之后的账户资金账户数据{:?}", trade_incomes);
          return Vec::from(trade_incomes.clone());
      } else {
          error!("Can't get {} income.", name);
          return Vec::from(trade_incomes.clone());
      }
}