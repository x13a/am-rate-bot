use crate::sources::lsoft::AphenaResponse;

pub const API_URL: &str = "https://m.artsakhbank.am:9443/get_ART.php";

pub struct Response;

impl AphenaResponse for Response {
    fn url() -> String {
        API_URL.into()
    }
}
