use std::string;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignIn {
    pub name: String,
    pub password: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitation {
    pub token: String,
    pub r#type: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateInvitationProRes {
    pub(crate) code: String,
    pub(crate) status: String,
    pub(crate) max: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignInProRes {
    pub(crate) name: String,
    pub(crate) id: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignInRes {
    pub(crate) name: String,
    pub(crate) account: u64,
    pub(crate) admin: String,
    pub(crate) products: Vec<SignInProRes>,
    pub(crate) token: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvitationRes {
    pub(crate) name: String,
    pub(crate) invitation: Vec<CreateInvitationProRes>
}



#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupAccountProRes {
    pub(crate) name: String,
    pub(crate) group_id: u64,
    pub(crate) tra_id: u64,
    pub(crate) tra_name: String,
    pub(crate) equity: String,
    pub(crate) leverage: String,
    pub(crate) position: String,
    pub(crate) open_order_amt: String,
    pub(crate) avaliable_balance: String,
    pub(crate) tra_venue: String,
    pub(crate) r#type: String,
    pub(crate) total_balance: String,
}



#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupEquitysProRes {
    pub(crate) name: u64,
    pub(crate) time: String,
    pub(crate) equity: String,
    pub(crate) r#type: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradeRe {
    pub th_id: u64,
    pub tra_symbol: String,
    pub tra_order_id: u64,
    // pub tra_id: u64,
    pub tra_commision: String,
    pub tra_time: String,
    pub is_maker: String,
    pub position_side: String,
    pub price: String,
    pub qty: String,
    pub quote_qty: String,
    pub realized_pnl: String,
    pub side: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradeRes {
    pub trades_history: Vec<TradeRe>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SignOut {
    pub name: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    // pub prod_id: String,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectTraders {
    pub account_id: u64,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SelectAccounts {
    pub name: String,
    pub account_id: u64,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SelectWeixin {
    pub wx_name: String,
    pub wx_hook: String,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct InsertAccounts {
    pub user_name: String,
    pub password: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct IsAccGroup {
    pub r#type: String,
    pub token: String,
    pub account_id: u64,
    pub group_id: u64
}


#[derive(Debug, Serialize, Deserialize)]
pub struct IsAccTra {
    pub r#type: String,
    pub token: String,
    pub account_id: u64,
    pub tra_id: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddShareList {
    pub r#type: String,
    pub token: String,
    pub tra_id: u64,
    pub from_id: String,
    pub to_id: String,
    pub tra_name: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteShareAcc {
    pub r#type: String,
    pub token: String,
    pub tra_id: u64,
    pub to_id: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteShareAccGroup {
    pub r#type: String,
    pub token: String,
    pub group_id: u64,
    pub to_id: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct InsertAccount {
    pub token: String,
    pub tra_venue: String, 
    pub tra_currency: String, 
    pub ori_balance: String, 
    pub api_key: String, 
    pub secret_key: String, 
    pub r#type: String, 
    pub name: String, 
    pub alarm: String, 
    pub threshold: String, 
    pub thres_amount: String, 
    pub borrow_currency: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AccountEquity {
    // pub prod_id: String,
    pub r#type: String,
    pub token: String,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccShareList {
    // pub prod_id: String,
    pub r#type: String,
    pub token: String,
    pub from_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectAccount {
    pub tra_id: String,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub tra_id: String,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AddTradeNotice {
    pub tra_id: String,
    pub wx_hook: String,
    pub wx_name: String,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTradeSlackNotice {
    pub tra_id: String,
    pub slack_hook: String,
    pub slack_name: String,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTradeSlackNotice {
    pub tra_id: String,
    pub hook: String,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AddAccountGroup {
    pub name: String,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAccGroup {
    pub account_id: u64,
    pub name: String,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AddGroupTra {
    pub tra_id: Vec<u64>,
    pub name: String,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAccountTra {
    pub tra_id: Vec<u64>,
    pub account_id: u64,
    pub r#type: String,
    pub token: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PapiSub {
    pub id: String,
    pub name: String,
    pub total_equity: String,
    pub leverage: String,
    pub open_order_amt: String,
    pub position: String,
    pub available_balance: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateTrade {
    pub tra_id: String,
    pub r#type: String,
    pub token: String,
    pub start_time: String,
    pub end_time: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Equity {
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelectOrders{
    pub r#type: String,
    pub token: String,
    pub tra_id: String,
    pub account_id: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SelectNewOrders{
    pub r#type: String,
    pub token: String,
    pub tra_id: String,
    pub start_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAccounts{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub api_key: String,
    pub secret_key: String,
    pub alarm: String,
    pub threshold: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectId{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub prod_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddOrders{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub api_key: String,
    pub secret_key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPositions{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub api_key: String,
    pub secret_key: String,
    pub threshold: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePositions{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub threshold: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAlarms{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub alarm: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCurreny{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub currency: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBorrow{
    pub r#type: String,
    pub token: String,
    pub name: String,
    pub borrow: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOriBalance{
    pub r#type: String,
    pub token: String,
    pub tra_id: String,
    pub ori_balance: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Posr {
    pub tra_id: String,
    pub r#type: String,
    pub token: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub account_id: u64,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailGroup {
    pub group_id: u64,
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectInvitation {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectAllInvitation {
    pub token: String,
    pub r#type: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomesRe {
    pub r#type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetWorthRe {
    pub r#type: String,
    pub token: String
}







#[derive(Serialize, Deserialize, Clone)]
pub struct Position {
    pub time: String,
    pub entry_price: String,
    pub leverage: String,
    pub mark_price: String,
    pub position_amt: String,
    pub position_side: String,
    pub symbol: String, 
    pub un_realized_profit: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PositionsRe {
    pub positions: Vec<Position>,
}



#[derive(Serialize, Deserialize, Clone)]
pub struct Total {
    pub equity_eth: String,
    pub net_worth: String,
    pub net_worth_eth: String,
    pub equity: String,
    // pub day_pnl: String,
    // pub week_pnl: String,
    pub time: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sub {
    pub id: String,
    pub name: String,
    pub net_worth: String,
    // pub origin_balance: String,
    pub total_balance_u: String,
    pub total_balance: String,
    // pub total_margin: String,
    // pub total_margin_eth: String, 
    pub total_equity: String,
    pub total_equity_eth: String,
    pub leverage: String,
    pub leverage_eth: String,
    pub open_order_amt: String,
    // pub available_margin: String,
    // pub available_margin_eth: String,
    // pub locked_margin: String,
    // pub locked_margin_eth: String,
    pub position: String,
    // pub short_position: String,
    // pub day_transaction_qty: String,
    // pub week_transaction_qty: String,
    // pub day_transaction_price: String,
    // pub week_transaction_price: String,
    // pub day_pnl: String,
    // pub week_pnl: String,
    pub available_balance: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Klines {
    pub symbol: String,
    pub r#type: String,
    pub token: String
}


#[derive(Serialize, Deserialize, Clone)]
pub struct ByBitSub {
    pub id: String,
    pub name: String,
    pub net_worth: String,
    pub total_equity: String,
    pub leverage: String,
    pub open_order_amt: String,
    pub position: String,
    pub available_balance: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountRe {
    // pub total: Total,
    pub subs: Vec<Sub>,
}

impl AccountRe {
    pub fn new() -> Self {
        Self {
            subs: Vec::new(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct AccountByBitRe {
    // pub total: Total,
    pub bybit_subs: Vec<ByBitSub>,
}

impl AccountByBitRe {
    pub fn new() -> Self {
        Self {
            bybit_subs: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountPapiRe {
    // pub total: Total,
    pub papi_subs: Vec<PapiSub>,
}

impl AccountPapiRe {
    pub fn new() -> Self {
        Self {
            papi_subs: Vec::new(),
        }
    }
}