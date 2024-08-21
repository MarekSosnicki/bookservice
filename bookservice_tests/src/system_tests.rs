use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde_json::json;

use bookservice_repository::api::{BookDetails, BookDetailsPatch};
use bookservice_repository::client::BookServiceRepositoryClient;

#[tokio::test]
/// Simple test for bookservice repository
/// Creates a book
/// Get the book
/// Patches the book
/// Gets list of books and checks if the book is there
async fn bookservice_repository_e2e_test() {
    let bookservice_repository_url = "http://127.0.0.1:8001";
    let bookservice_repository_client =
        BookServiceRepositoryClient::new(bookservice_repository_url)
            .expect("Failed to create client");

    let book_details = BookDetails {
        title: "title1".to_string(),
        authors: vec!["Author1".to_string()],
        publisher: "Publisher1".to_string(),
        description: "Description1".to_string(),
        tags: vec!["TAG1".to_string(), "TAG2".to_string()],
    };

    let book_id = bookservice_repository_client
        .add_book(book_details.clone())
        .await
        .expect("Failed to add book");

    let returned_book_details = bookservice_repository_client
        .get_book(book_id)
        .await
        .expect("Failed to get book")
        .expect("Book not found");

    assert_eq!(returned_book_details, book_details);

    let updated_title = format!(
        "updated title {}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let book_patch = BookDetailsPatch {
        title: Some(updated_title.clone()),
        ..BookDetailsPatch::default()
    };

    bookservice_repository_client
        .update_book(book_id, book_patch)
        .await
        .expect("Failed to patch book");

    let returned_book_details = bookservice_repository_client
        .get_book(book_id)
        .await
        .expect("Failed to get book")
        .expect("Book not found");

    let patched_book_details = BookDetails {
        title: updated_title.clone(),
        ..book_details
    };
    assert_eq!(returned_book_details, patched_book_details);

    let books_and_titles = bookservice_repository_client
        .list_books()
        .await
        .expect("Failed to list books");

    assert!(books_and_titles
        .iter()
        .any(|id_and_title| id_and_title.book_id == book_id && id_and_title.title == updated_title))
}

#[test]
/// Simple test for bookservice reservations
/// Creates a user
/// Gets the user
/// Gets all users to see if user is there
/// Creates a book (in booservice repository)
/// Reserves a book for user
/// Checks current reservations
/// Unreserves a book
/// Gets history of reservations
fn bookservice_reservations_e2e_test() {
    let bookservice_repository_url = "http://127.0.0.1:8001";
    let bookservice_reservations_url = "http://127.0.0.1:8002";
    let client = reqwest::blocking::Client::new();

    let username = format!(
        "User{}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let user_details = json!(
        {
          "username": username,
          "favourite_tags": [
            "TAG1", "TAG2"
          ]
        }
    );

    // ADD USER
    let add_user_response = client
        .post(format!("{}/api/user", bookservice_reservations_url))
        .json(&user_details)
        .send()
        .expect("Failed to add user");

    assert!(add_user_response.status().is_success());

    let user_location = add_user_response
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("Missing location header")
        .to_str()
        .expect("Failed to transform header value to string");

    assert!(user_location.starts_with("/api/user/"));

    // GET USER

    let get_user_response = client
        .get(format!("{}{}", bookservice_reservations_url, user_location))
        .send()
        .expect("Failed to get user");
    assert!(get_user_response.status().is_success());
    let returned_user_details: serde_json::Value = get_user_response
        .json()
        .expect("Failed to parse response json");
    assert_eq!(returned_user_details, user_details);

    // GET ALL USERS

    let get_all_users_response = client
        .get(format!("{}/api/users", bookservice_reservations_url))
        .send()
        .expect("Failed to get list of users");
    assert!(get_all_users_response.status().is_success());

    let get_all_response_body: Vec<i32> = get_all_users_response
        .json()
        .expect("Failed to parse response");

    let user_id: i32 = user_location
        .split("/")
        .last()
        .expect("Failed to get id")
        .parse()
        .expect("failed to parse book id");

    assert!(get_all_response_body.iter().any(|id| *id == user_id));

    // ADD BOOK

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

    let book_location = add_response
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("Missing location header")
        .to_str()
        .expect("Failed to transform header value to string");

    let book_id: i32 = book_location
        .split("/")
        .last()
        .expect("Failed to get id")
        .parse()
        .expect("failed to parse book id");

    // RESERVE Book
    let reserve_response = client
        .post(format!(
            "{}{}/reservation/{}",
            bookservice_reservations_url, user_location, book_id
        ))
        .send()
        .expect("Failed to reserve book");
    assert!(reserve_response.status().is_success());

    // RESERVE AGAIN - this time should fail as already reserved
    // TODO: Add second user for this?
    let reserve_response = client
        .post(format!(
            "{}{}/reservation/{}",
            bookservice_reservations_url, user_location, book_id
        ))
        .send()
        .expect("Failed to reserve book");
    assert!(reserve_response.status().is_client_error());

    // GET ALL RESERVATIONS
    let get_all_reservations_response = client
        .get(format!(
            "{}{}/reservations",
            bookservice_reservations_url, user_location
        ))
        .send()
        .expect("Failed to get all reservations");
    assert!(get_all_reservations_response.status().is_success());

    let reservation_ids: Vec<i32> = get_all_reservations_response
        .json()
        .expect("Failed to parse reservation ids");
    assert_eq!(reservation_ids, vec![book_id]);

    // UNRESERVE
    let unreserve_response = client
        .delete(format!(
            "{}{}/reservation/{}",
            bookservice_reservations_url, user_location, book_id
        ))
        .send()
        .expect("Failed to unreserve book");
    assert!(unreserve_response.status().is_success());

    // GET ALL RESERVATIONS to see if it is removed
    let get_all_reservations_response = client
        .get(format!(
            "{}{}/reservations",
            bookservice_reservations_url, user_location
        ))
        .send()
        .expect("Failed to get all reservations");
    assert!(get_all_reservations_response.status().is_success());

    let reservation_ids: Vec<i32> = get_all_reservations_response
        .json()
        .expect("Failed to parse reservation ids");
    assert_eq!(reservation_ids, Vec::<i32>::default());

    // GET History response
    let get_reservation_history = client
        .get(format!(
            "{}{}/history",
            bookservice_reservations_url, user_location
        ))
        .send()
        .expect("Failed to get all reservations");
    assert!(get_reservation_history.status().is_success());

    #[derive(Deserialize)]
    struct HistoryRecord {
        book_id: i32,
        unreserved_at: u64,
    }

    let history_records: Vec<HistoryRecord> = get_reservation_history
        .json()
        .expect("Failed to parse reservation history");
    assert_eq!(history_records.len(), 1);
    assert_eq!(history_records[0].book_id, book_id);
    assert!(history_records[0].unreserved_at > 0);
}
