use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://avosend.com/api/comission.php";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub from_scale: i32,
    pub to_scale: i32,
    pub from: f64,
    pub to: f64,
    pub fee: f64,
    pub convert_rate: Decimal,
    pub currency_rate_text: String,
    pub from_prv_id: u64,
    pub to_prv_id: u64,
    pub restriction_from: Restriction,
    pub restriction_to: Restriction,
    pub special_rate_offer: Option<String>,
    pub code: i32,
    pub error_message: Option<String>,
    pub tariff_info: Option<String>,
    pub tariffs: Vec<Tariff>,
}

#[derive(Debug, Deserialize)]
pub struct Tariff {
    pub from: Option<f64>,
    pub to: Option<f64>,
    pub percent: f64,
    pub fix: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Restriction {
    pub ccy: u64,
    pub ccy_iso: String,
    pub max_amount: f64,
    pub min_amount: f64,
    pub multiple: Option<f64>,
}

pub mod request {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Body {
        pub country_code_from: String,
        pub country_id_from: u64,
        pub country_code_to: String,
        pub country_id_to: u64,
        pub currency_id_from: u64,
        pub currency_id_to: u64,
        pub summ_send: u64,
        pub direction: String,
    }
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> anyhow::Result<Self> {
        const RU_ID: u64 = 643;
        const AM_ID: u64 = 51;
        let req_body = request::Body {
            country_code_from: "ru".into(),
            country_id_from: RU_ID,
            country_code_to: "am".into(),
            country_id_to: AM_ID,
            currency_id_from: RU_ID,
            currency_id_to: AM_ID,
            summ_send: 10000,
            direction: "from".into(),
        };
        let mut resp = c
            .post(Self::url())
            .form(&req_body)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .send()
            .await?
            .text()
            .await?;
        const CLOSE_SCRIPT_TAG: &str = "</script>";
        if let Some(idx) = resp.find(CLOSE_SCRIPT_TAG) {
            resp.drain(..idx + CLOSE_SCRIPT_TAG.len());
        }
        let resp = serde_json::from_str(&resp.trim())?;
        Ok(resp)
    }
}
