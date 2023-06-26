
use chrono::Local;
use log::error;
use serde_json::{Map,Value};
use std::collections::VecDeque;
use std::fs;
use chrono::{DateTime, NaiveDateTime, Utc};
use super::http_data::Sub;
use super::base::venue_api::HttpVenueApi;

pub async fn get_account_bybit(
  http_api: &Box<dyn HttpVenueApi>,
  name: &str,
  id: &u64,
  origin_balance: f64,
  alarm: &str,
) -> Option<Sub> {
  if let Some(data) = http_api.account().await {
      let value: Value = serde_json::from_str(&data).unwrap();
      println!("账户信息{}", value);
      let assets = value.as_object().unwrap().get("assets")
      .unwrap().as_array().unwrap();
      let mut new_total_balance = 0.00;
      let mut new_total_equity = 0.00;
      let mut best_price = 0.00;
      for a in assets {
          let obj = a.as_object().unwrap();
          let wallet_balance: f64 = obj.get("walletBalance").unwrap().as_str().unwrap().parse().unwrap();
          let symbol = obj.get("asset").unwrap().as_str().unwrap();

          if wallet_balance != 0.00 {
              if symbol == "ETH" {
                  let asset = format!("{}USDT", symbol);
                  if let Some(data) = http_api.get_klines(&asset).await {
                      let v: Value = serde_json::from_str(&data).unwrap();
                      let price_obj = v.as_object().unwrap();
                      let price:f64 = price_obj.get("price").unwrap().as_str().unwrap().parse().unwrap();
                      best_price = price;
                      let new_price = wallet_balance * price;
                      new_total_balance += new_price;
                      new_total_equity += new_price;
                  }
              }

              let cross_un_pnl: f64 = obj.get("crossUnPnl").unwrap().as_str().unwrap().parse().unwrap();
              let pnl = cross_un_pnl + wallet_balance;
              new_total_balance += wallet_balance;
              new_total_equity += pnl;
          }
      }
      // 余额
      let total_wallet_balance: f64 = ((new_total_balance / best_price) - 40.00) * best_price;
      // 权益
      let new_total_equity_eth: f64 = ((new_total_equity / best_price) - 40.00) * best_price;
      let net_worth = new_total_equity / origin_balance;
      
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
      // let total_equity = total_balance + total_pnl; // 权益 = 余额 + 未实现盈亏
      
      // let total_margin: f64 = value
      //     .as_object()
      //     .unwrap()
      //     .get("totalMarginBalance")
      //     .unwrap()
      //     .as_str()
      //     .unwrap()
      //     .parse()
      //     .unwrap();
      // let total_marign_eth: f64 = total_margin / best_price;
      // let available_margin: f64 = value
      //     .as_object()
      //     .unwrap()
      //     .get("totalMaintMargin")
      //     .unwrap()
      //     .as_str()
      //     .unwrap()
      //     .parse()
      //     .unwrap();
      // let available_margin_eth: f64 = available_margin / best_price;
      // let locked_margin = total_margin - available_margin;
      // let locked_margin_eth: f64 = locked_margin / best_price;
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
          }

      }
      // let position = amts * prices;


      let leverage = amts.abs() / new_total_equity; // 杠杆率 = 仓位价值 / 本金（账户总金额 + 未实现盈亏）
      // println!("当前杠杆率{}", leverage);
      let leverage_eth = amts.abs()/ total_wallet_balance;

      if let Some(data) = http_api.get_open_orders().await {
          let value: Value = serde_json::from_str(&data).unwrap();
          let open_orders = value.as_array().unwrap();
          let open_order = open_orders.len();

          println!("当前挂单数量:{}, name:{}", open_order, name);

          return Some(Sub {
              id: String::from(id.to_string()),
              name: String::from(name),
              total_balance_u:format!("{}", new_total_balance),
              total_balance: format!("{}", total_wallet_balance),
              total_equity: format!("{}", new_total_equity),
              total_equity_eth: format!("{}", new_total_equity_eth),
              leverage: format!("{}", leverage),
              leverage_eth: format!("{}", leverage_eth),
              position: format!("{}", amts),
              open_order_amt: format!("{}", open_order),
              net_worth: format!("{}", net_worth),
              // day_transaction_price: format!("{}", day_transaction_price),
              // week_transaction_price: format!("{}", week_transaction_price),
              // day_pnl: format!("{}", day_pnl ),
              // week_pnl: format!("{}", week_pnl ),
              available_balance: format!("{}", available_balance),
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