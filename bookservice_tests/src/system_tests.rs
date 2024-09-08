use std::time::UNIX_EPOCH;

use bookservice_repository::api::{BookDetails, BookDetailsPatch};
use bookservice_repository::client::BookServiceRepositoryClient;
use bookservice_reservations::api::{BookId, UserDetails};
use bookservice_reservations::client::BookServiceReservationsClient;

#[tokio::test]
/// Simple test for bookservice repository
/// Creates a book
/// Get the book
/// Patches the book
/// Gets list of books and checks if the book is there
async fn bookservice_repository_e2e_test() {
    let bookservice_repository_url = "http://127.0.0.1:80";
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

#[tokio::test]
/// Simple test for bookservice reservations
/// Creates a user
/// Gets the user
/// Gets all users to see if user is there
/// Creates a book (in booservice repository)
/// Reserves a book for user
/// Checks current reservations
/// Unreserves a book
/// Gets history of reservations
async fn bookservice_reservations_e2e_test() {
    let bookservice_repository_url = "http://127.0.0.1:80";
    let bookservice_reservations_url = "http://127.0.0.1:80";
    let bookservice_repository_client =
        BookServiceRepositoryClient::new(bookservice_repository_url)
            .expect("Failed to create bookservice_repository_client");
    let bookservice_reservations_client =
        BookServiceReservationsClient::new(bookservice_reservations_url)
            .expect("Failed to create bookservice_reservations_client");

    let username = format!(
        "User{}",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let user_details = UserDetails {
        username: username.clone(),
        favourite_tags: vec!["tag1".to_string()],
    };

    // ADD USER
    let user_id = bookservice_reservations_client
        .add_user(user_details.clone())
        .await
        .expect("Failed to add user");

    // GET USER
    let returned_user_details = bookservice_reservations_client
        .get_user(user_id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(returned_user_details, user_details);

    // GET ALL USERS

    let users_list = bookservice_reservations_client
        .list_users()
        .await
        .expect("Failed to get list of users");

    assert!(users_list.iter().any(|id| *id == user_id));

    // ADD BOOK

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

    // RESERVE Book
    let reserve_response = bookservice_reservations_client
        .reserve_book(book_id, user_id)
        .await
        .expect("Failed to reserve book");

    assert!(reserve_response);

    // RESERVE AGAIN - this time should fail as already reserved
    // TODO: Add second user for this?
    let reserve_response = bookservice_reservations_client
        .reserve_book(book_id, user_id)
        .await
        .expect("Failed to reserve book");
    assert!(!reserve_response);

    // GET ALL RESERVATIONS
    let reservation_ids = bookservice_reservations_client
        .list_reservations(user_id)
        .await
        .expect("Failed to get all reservations");

    assert_eq!(reservation_ids, vec![book_id]);

    // UNRESERVE
    let unreserve_response = bookservice_reservations_client
        .unreserve_book(book_id, user_id)
        .await
        .expect("Failed to unreserve book");

    assert!(unreserve_response);

    // GET ALL RESERVATIONS to see if it is removed
    let reservation_ids = bookservice_reservations_client
        .list_reservations(user_id)
        .await
        .expect("Failed to get all reservations");
    assert_eq!(reservation_ids, Vec::<BookId>::default());

    // GET History response
    let history_records = bookservice_reservations_client
        .history(user_id)
        .await
        .expect("Failed to get all reservations");

    assert_eq!(history_records.len(), 1);
    assert_eq!(history_records[0].book_id, book_id);
    assert!(history_records[0].unreserved_at > 0);
}
