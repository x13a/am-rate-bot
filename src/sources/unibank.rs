use crate::sources::lsoft::AphenaResponse;

pub const API_URL: &str = "https://force.unibank.am:9443/xmlParser_mob.php";

pub struct Response;

impl AphenaResponse for Response {
    fn url() -> String {
        API_URL.into()
    }
}
