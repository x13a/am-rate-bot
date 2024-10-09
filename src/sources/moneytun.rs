use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://www.moneytun.ru/api/api/calculatetax";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub calculation_result: CalculationResult,
    pub error: bool,
    pub error_message: String,
    pub exchange_rate: String,
    pub fee_combination: Vec<FeeCombination>,
    pub payment_method_list: Vec<PaymentMethod>,
    #[serde(rename = "RecCurrID")]
    pub rec_curr_id: u64,
    pub rec_currency_code: String,
    pub send_currency_code: String,
    pub status_code: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CalculationResult {
    pub amount: f64,
    pub amount_to_display: f64,
    pub exchange_rate: String,
    pub exchange_rate2: Decimal,
    pub fee_customer: f64,
    pub per_customer: f64,
    pub per_customer_value: f64,
    pub purchase_rate: f64,
    pub rec_currency_code: String,
    pub rec_currency_symbol: Option<String>,
    pub recepient_receive_amount: f64,
    pub send_currency_code: String,
    pub send_currency_symbol: Option<String>,
    pub tax1_amount: f64,
    pub tax1_is_visible: bool,
    pub tax1_label: String,
    pub tax1_mode: String,
    pub tax1_paid_by: Option<String>,
    pub tax1_ratio: f64,
    pub tax2_amount: f64,
    pub tax2_is_visible: bool,
    pub tax2_label: String,
    pub tax2_paid_by: Option<String>,
    pub tax2_ratio: f64,
    pub tax_currency1: u64,
    pub tax_currency2: u64,
    pub total_charges: f64,
    pub total_sending_amount: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeeCombination {
    pub combination_img_url: String,
    pub delivery_method_id: u64,
    pub delivery_method_name: String,
    pub estimated_time: String,
    pub estimated_time_description: String,
    pub fee_customer: f64,
    pub payee_id: u64,
    pub payment_method_id: u64,
    pub payment_method_name: String,
    pub per_customer: f64,
    pub rec_city_id: u64,
    pub rec_country_id: u64,
    pub rec_currency_id: u64,
    pub rec_state_id: u64,
    pub selected: bool,
    pub send_country_id: u64,
    pub send_currency_id: u64,
    pub send_state_id: u64,
    pub total_fee: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentMethod {
    pub id: u64,
    pub method: String,
    pub method_spanish: Option<String>,
    pub payment_method_img_url: String,
    pub selected: bool,
}

pub mod request {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct Json {
        pub sender_id: u64,
        pub rec_country_id: u64,
        pub rec_currency_id: u64,
        pub send_country_id: u64,
        pub send_currency_id: u64,
        pub amount: f64,
        pub payee_id: u64,
        pub delivery_method_id: u64,
        pub payment_method_id: u64,
        pub min_fee: i32,
        pub option: i32,
        pub sub_payer_name: String,
        pub is_sending_amount: bool,
        pub send_state_id: u64,
        pub rec_state_id: u64,
        pub rec_city_id: u64,
    }
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> anyhow::Result<Self> {
        let json = request::Json {
            sender_id: 0,
            rec_country_id: 10,
            rec_currency_id: 113,
            send_country_id: 193,
            send_currency_id: 112,
            amount: 10000.0,
            payee_id: 0,
            delivery_method_id: 2,
            payment_method_id: 6,
            min_fee: 1,
            option: 1,
            sub_payer_name: "".into(),
            is_sending_amount: true,
            send_state_id: 10,
            rec_state_id: 0,
            rec_city_id: 0,
        };
        let resp = c.post(Self::url()).json(&json).send().await?.json().await?;
        Ok(resp)
    }
}
