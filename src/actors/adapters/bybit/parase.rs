
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
      println!("bybit账户数据{}", value);
      
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
        let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
        let positions = result.get("list").unwrap().as_array().unwrap();
      // let mut position: f64 = 0.0;
      
      let mut amts: f64 = 0.0;
      // let mut short_position: f64 = 0.0;
      for p in positions {
          let obj = p.as_object().unwrap();
          let position_amt: f64 = obj.get("size").unwrap().as_str().unwrap().parse().unwrap();
          let price: f64 = obj.get("markPrice").unwrap().as_str().unwrap().parse().unwrap();
          let pos_price = position_amt * price;
          amts += pos_price;
      }
      // let position = amts * prices;


      let leverage = amts.abs() / equity; // 杠杆率 = 仓位价值 / 本金（账户总金额 + 未实现盈亏）

      if let Some(data) = http_api.get_open_orders(category_lear).await {
          let value: Value = serde_json::from_str(&data).unwrap();
          let result = value.as_object().unwrap().get("result").unwrap().as_object().unwrap();
          let list = result.get("list").unwrap().as_array().unwrap();
          let open_order = list.len();

          println!("当前挂单数量:{}, name:{}", open_order, name);

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
      return None;
          
      }
    } else {
        error!("Can't get {} openOrders.", name);
    return None;
        
    }
  } else {
      error!("Can't get {} account.", name);
      return None;
  }
}