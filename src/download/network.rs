extern crate reqwest;

use self::reqwest::header;
use self::reqwest::Client;
use self::reqwest::Response;

pub struct Network {
    pub client: Client,
}

impl Default for Network {
    fn default() -> Network {
        Network {
            client: Client::new(),
        }
    }
}

impl Network {
    pub fn make_request(&self, url: &String, range: String) -> Response {
        let request = self.client
            .get(url)
            .header(header::RANGE, range);

        return request.send().expect("Could not send request.");
    }

    pub fn get_content_length(&self, url: &String) -> Option<u64> {
        return self.make_request(url, "".to_string()).content_length();
    }
}


/*trait Request {
    fn make_request<T>(&self, String, String) -> T;
}

impl Request for Network {
    fn make_request<Response>(&self, url: String, range: String) -> Response {
        let mut request = self.client.get(&url);
        if range.len() > 0 {
            request.header(header::RANGE, range);
        }
        return request.send().expect("Could not send request.");
    }
}*/