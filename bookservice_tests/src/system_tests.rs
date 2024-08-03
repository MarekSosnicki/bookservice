use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde_json::json;

#[test]
/// Simple test for bookservice repository
/// Creates two books
/// Gets them
/// Patches one of them
fn bookservice_repository_e2e_test() {
    let bookservice_repository_url = "http://127.0.0.1:1234";

    let client = reqwest::blocking::Client::new();

    let book_details = json!(
        {
          "title": "title1",
          "authors": [
            "Author1"
          ],
          "publisher": "Publisher1",
          "description": "Description1",
          "tags": [
            "TAG1", "TAG2"
          ]
        }
    );

    let add_response = client
        .post(format!("{}/api/book", bookservice_repository_url))
        .json(&book_details)
        .send()
        .expect("Failed to post book");

    assert!(add_response.status().is_success());

    let location = add_response
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("Missing location header")
        .to_str()
        .expect("Failed to transform header value to string");

    assert!(location.starts_with("/api/book/"));

    let get_response = client
        .get(format!("{}{}", bookservice_repository_url, location))
        .send()
        .expect("Failed to get book");
    assert!(get_response.status().is_success());

    let returned_book_details: serde_json::Value =
        get_response.json().expect("Failed to parse response json");

    assert_eq!(returned_book_details, book_details);

    let updated_title = format!(
        "updated title {}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let book_patch = json!(
        {
          "title": updated_title,
        }
    );

    let patch_response = client
        .patch(format!("{}{}", bookservice_repository_url, location))
        .json(&book_patch)
        .send()
        .expect("Failed to patch book");
    assert!(patch_response.status().is_success());

    let get_response = client
        .get(format!("{}{}", bookservice_repository_url, location))
        .send()
        .expect("Failed to get book");
    assert!(get_response.status().is_success());

    let returned_book_details: serde_json::Value =
        get_response.json().expect("Failed to parse response json");

    let mut patched_book_details = book_details;

    patched_book_details
        .as_object_mut()
        .unwrap()
        .insert("title".to_string(), json!(updated_title));

    assert_eq!(returned_book_details, patched_book_details);

    let get_all_response = client
        .get(format!("{}/api/books", bookservice_repository_url))
        .send()
        .expect("Failed to get list of book");
    assert!(get_all_response.status().is_success());

    #[derive(Deserialize)]
    struct BookIdAndTitle {
        book_id: i32,
        title: String,
    }

    #[derive(Deserialize)]
    struct GetAllResponse {
        books: Vec<BookIdAndTitle>,
    }

    let get_all_response_body: GetAllResponse =
        get_all_response.json().expect("Failed to parse response");

    let book_id: i32 = location
        .split("/")
        .last()
        .expect("Failed to get id")
        .parse()
        .expect("failed to parse book id");

    assert!(get_all_response_body
        .books
        .iter()
        .any(|id_and_title| id_and_title.book_id == book_id && id_and_title.title == updated_title))
}
