use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub acc_id: u64,
    pub acc_name: String,
    pub acc_password: String,
    pub admin: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccProd {
    pub ap_id: u64,
    pub acc_id: u64,
    pub prod_id: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub prod_id: u64,
    pub prod_name: String,
    pub weixin_id: u64,
    pub prog_id: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AccountGroup {
    pub group_id: u64,
    pub name: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct GroupTra {
    pub id: u64,
    pub group_id: u64,
    pub tra_id: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenOrders {
    pub id: u64,
    pub api_key: String,
    pub secret_key: String,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetWorths {
    pub name: String,
    pub time: String,
    pub net_worth: String,
    pub prod_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Equitys {
    pub id: u64,
    pub name: String,
    pub time: String,
    pub equity_eth: String,
    pub equity: String,
    pub prod_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PositionsAlarm {
    pub id: u64,
    pub api_key: String,
    pub secret_key: String,
    pub name: String,
    pub threshold: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Active {
    pub acc_id: u64,
    pub token: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trader {
    pub tra_id: u64,
    pub tra_venue: String,
    pub tra_currency: String,
    pub api_key: String,
    pub secret_key: String,
    pub r#type: String,
    pub name: String,
    pub borrow: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraderAlarm {
    pub id: u64,
    pub acc_id: u64,
    pub tra_id: u64,
    pub open_alarm: String,
    pub position_alarm: String,
    pub position_amount: String,
    pub equity_alarm: String,
    pub equity_amount: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraderMessage {
    pub id: u64,
    pub tra_id: u64,
    pub name: String,
    pub equity: String,
    pub leverage: String,
    pub position: String,
    pub open_order_amt: String,
    pub avaliable_balance: String,
    pub tra_venue: String,
    pub r#type: String,
    pub total_balance: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountData {
    pub ap_id: u64,
    pub acc_id: u64,
    pub tra_id: u64,
    pub is_show: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShareList {
    pub sh_id: u64,
    pub from_id: u64,
    pub to_id: String,
    pub tra_id: u64,
    pub tra_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvitationData {
    pub code: String,
    pub user: String,
    pub max: u64,
    pub status: String,
    pub id: u64
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlackNotices {
    pub id: u64,
    pub acc_id: u64,
    pub slack_hook: String,
    pub slack_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WxNotices {
    pub id: u64,
    pub acc_id: u64,
    pub wx_hook: String,
    pub wx_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    pub th_id: u64,
    pub tra_symbol: String,
    pub tra_order_id: u64,
    // pub tra_id: u64,
    pub tra_commision: String,
    pub tra_time: u64,
    pub is_maker: String,
    pub position_side: String,
    pub price: String,
    pub qty: String,
    pub quote_qty: String,
    pub realized_pnl: String,
    pub side: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct NewTrade {
    pub th_id: u64,
    pub tra_symbol: String,
    pub tra_order_id: u64,
    pub tra_commision: String,
    pub tra_time: u64,
    pub is_maker: String,
    pub position_side: String,
    pub price: String,
    pub qty: String,
    pub quote_qty: String,
    pub realized_pnl: String,
    pub side: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BybitNewTrade {
    pub tra_order_id: String,
    pub th_id: String,
    pub time: u64,
    pub symbol: String,
    pub side: String,
    pub price: String,
    pub qty: String,
    pub quote_qty: String,
    pub commission: String,
    pub name: u64,
    pub is_maker: String,
    pub exec_id: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct BybitTrade {
    pub tra_order_id: String,
    pub th_id: String,
    pub time: u64,
    pub symbol: String,
    pub side: String,
    pub price: String,
    pub qty: String,
    pub quote_qty: String,
    pub commission: String,
    pub r#type: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BybitEquity {
    pub id: u64,
    pub name: u64,
    pub time: String,
    pub equity: String,
    pub r#type: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct GroupEquity {
    pub id: u64,
    pub name: u64,
    pub equity: u64,
    pub time: String,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BianEquity {
    pub id: u64,
    pub name: u64,
    pub time: String,
    pub equity: String,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClearData {
    pub id: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Incomes {
    pub tra_id: u64,
    pub tra_venue: String,
    pub ori_balance: String,
    pub tra_currency: String,
    pub api_key: String,
    pub secret_key: String,
    pub other_keys: String,
    pub r#type: String,
    pub name: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryIncomes {
    pub time: String,
    pub r#type: String,
    pub asset: String,
    // pub tra_id: u64,
    pub amount: String,
    pub tran_id: u64,
    pub status: String,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub symbol: String, 
    pub position_amt: String, 
    pub position_side: String, 
    pub time: String, 
    pub entry_price: String, 
    pub un_realized_profit: String, 
    pub tra_id: u64, 
    pub leverage: String, 
    pub mark_price: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetWorth {
    pub time: String,
    pub net_worth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Equity {
    pub id: u64,
    pub name: String,
    pub time: String,
    pub equity_eth: String,
    pub equity: String,
    pub prod_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewPrice {
    pub id: u64,
    pub name: String,
    pub week_price: String,
    pub day_price: String,
}