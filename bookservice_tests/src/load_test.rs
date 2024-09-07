use std::collections::HashMap;

use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;

use bookservice_repository::api::BookDetails;
use bookservice_repository::client::BookServiceRepositoryClient;
use bookservice_reservations::api::UserDetails;
use bookservice_reservations::client::BookServiceReservationsClient;

#[tokio::test]
async fn generate_lots_of_books_and_user_reservations() {
    const NO_OF_BOOKS_TO_GENERATE: usize = 10;
    const NO_OF_AUTHORS_TO_GENERATE: usize = 100;
    const NO_OF_USERS_TO_GENERATE: usize = 10;
    const NO_OF_RESERVATIONS: usize = 100;

    let mut rng = thread_rng();
    let bookservice_repository_url = "http://127.0.0.1:8001";
    let bookservice_reservations_url = "http://127.0.0.1:8002";

    let bookservice_repository_client =
        BookServiceRepositoryClient::new(bookservice_repository_url)
            .expect("Failed to create client");

    let bookservice_reservations_client =
        BookServiceReservationsClient::new(bookservice_reservations_url)
            .expect("Failed to create bookservice_reservations_client");

    let authors = generate_authors(&mut rng, NO_OF_AUTHORS_TO_GENERATE);
    let books = generate_books(&mut rng, NO_OF_BOOKS_TO_GENERATE, &authors);
    let users = generate_users(&mut rng, NO_OF_USERS_TO_GENERATE);

    let mut book_ids = vec![];

    for book in books {
        let book_id = bookservice_repository_client
            .add_book(book)
            .await
            .expect("Failed to add book");
        book_ids.push(book_id);

        println!("Added book {}", book_id);
    }

    let mut user_ids = vec![];
    for user in users {
        let user_id = bookservice_reservations_client
            .add_user(user)
            .await
            .expect("Failed to add user");
        user_ids.push(user_id);
        println!("Added user {}", user_id);
    }

    let mut reserved_books = HashMap::default();

    for _ in 0..NO_OF_RESERVATIONS {
        let book_id = book_ids.choose(&mut rng).unwrap();
        let user_id = user_ids.choose(&mut rng).unwrap();
        if let Some(currently_reserving_user) = reserved_books.remove(book_id) {
            let result = bookservice_reservations_client
                .unreserve_book(*book_id, *currently_reserving_user)
                .await
                .expect("Failed to unreserve book");
            assert!(result, "Failed to unreserve book - result false");
            println!(
                "Unreserved book {} from user {}",
                book_id, currently_reserving_user
            );
        }

        let result = bookservice_reservations_client
            .reserve_book(*book_id, *user_id)
            .await
            .expect("Failed to reserve book");
        assert!(result, "Failed to reserve book  - result false");

        reserved_books.insert(*book_id, user_id);
        println!("Reserved book {} for user {}", book_id, user_id);
    }
}

fn generate_authors(rng: &mut impl Rng, no_of_authors: usize) -> Vec<String> {
    (0..no_of_authors)
        .map(|_| {
            format!(
                "{} {}",
                FIRST_NAMES.choose(rng).unwrap(),
                LAST_NAMES.choose(rng).unwrap()
            )
        })
        .collect()
}

fn generate_books(
    rng: &mut impl Rng,
    no_of_books_to_generate: usize,
    authors: &[String],
) -> Vec<BookDetails> {
    (0..no_of_books_to_generate)
        .map(|no| BookDetails {
            title: format!("A tale of number {} and {}", no, rng.gen_range(0..1000)),
            authors: (0..rng.gen_range(1..3))
                .map(|_| authors.choose(rng).unwrap())
                .cloned()
                .collect(),
            publisher: format!("Publisher {}", no % 20),
            description: "Some long description that is long".to_string(),
            tags: vec![],
        })
        .collect()
}

fn generate_users(rng: &mut impl Rng, no_of_users_to_generate: usize) -> Vec<UserDetails> {
    (0..no_of_users_to_generate)
        .map(|no| UserDetails {
            username: format!(
                "{}_{}_{}",
                FIRST_NAMES.choose(rng).unwrap(),
                LAST_NAMES.choose(rng).unwrap(),
                no
            ),
            favourite_tags: vec![],
        })
        .collect()
}

/// List of first names, based on most popular names list
const FIRST_NAMES: [&str; 142] = [
    "Ryan",
    "Dorothy",
    "Jacob",
    "Amy",
    "Nicholas",
    "Kathleen",
    "Gary",
    "Angela",
    "Eric",
    "Shirley",
    "Jonathan",
    "Emma",
    "Stephen",
    "Brenda",
    "Larry",
    "Pamela",
    "Justin",
    "Nicole",
    "Scott",
    "Anna",
    "Brandon",
    "Samantha",
    "Benjamin",
    "Katherine",
    "Samuel",
    "Christine",
    "Gregory",
    "Debra",
    "Alexander",
    "Rachel",
    "Patrick",
    "Carolyn",
    "Frank",
    "Janet",
    "Raymond",
    "Maria",
    "Jack",
    "Olivia",
    "Dennis",
    "Heather",
    "Jerry",
    "Helen",
    "Tyler",
    "Catherine",
    "Aaron",
    "Diane",
    "Jose",
    "Julie",
    "Adam",
    "Victoria",
    "Nathan",
    "Joyce",
    "Henry",
    "Lauren",
    "Zachary",
    "Kelly",
    "Douglas",
    "Christina",
    "Peter",
    "Ruth",
    "Kyle",
    "Joan",
    "Noah",
    "Virginia",
    "Ethan",
    "Judith",
    "Jeremy",
    "Evelyn",
    "Christian",
    "Hannah",
    "Walter",
    "Andrea",
    "Keith",
    "Megan",
    "Austin",
    "Cheryl",
    "Roger",
    "Jacqueline",
    "Terry",
    "Madison",
    "Sean",
    "Teresa",
    "Gerald",
    "Abigail",
    "Carl",
    "Sophia",
    "Dylan",
    "Martha",
    "Harold",
    "Sara",
    "Jordan",
    "Gloria",
    "Jesse",
    "Janice",
    "Bryan",
    "Kathryn",
    "Lawrence",
    "Ann",
    "Arthur",
    "Isabella",
    "Gabriel",
    "Judy",
    "Bruce",
    "Charlotte",
    "Logan",
    "Julia",
    "Billy",
    "Grace",
    "Joe",
    "Amber",
    "Alan",
    "Alice",
    "Juan",
    "Jean",
    "Elijah",
    "Denise",
    "Willie",
    "Frances",
    "Albert",
    "Danielle",
    "Wayne",
    "Marilyn",
    "Randy",
    "Natalie",
    "Mason",
    "Beverly",
    "Vincent",
    "Diana",
    "Liam",
    "Brittany",
    "Roy",
    "Theresa",
    "Bobby",
    "Kayla",
    "Caleb",
    "Alexis",
    "Bradley",
    "Doris",
    "Russell",
    "Lori",
    "Lucas",
    "Tiffany",
];

/// List of last names based on most popular last names
const LAST_NAMES: [&str; 127] = [
    "Wilson",
    "Moore",
    "Taylor",
    "Anderson",
    "Thomas",
    "Jackson",
    "White",
    "Harris",
    "Martin",
    "Thompson",
    "Garcia",
    "Martinez",
    "Robinson",
    "Clark",
    "Rodriguez",
    "Lewis",
    "Lee",
    "Walker",
    "Hall",
    "Allen",
    "Young",
    "Hernandez",
    "King",
    "Wright",
    "Lopez",
    "Hill",
    "Scott",
    "Green",
    "Adams",
    "Baker",
    "Gonzalez",
    "Nelson",
    "Carter",
    "Mitchell",
    "Perez",
    "Roberts",
    "Turner",
    "Phillips",
    "Campbell",
    "Parker",
    "Evans",
    "Edwards",
    "Collins",
    "Stewart",
    "Sanchez",
    "Morris",
    "Rogers",
    "Reed",
    "Cook",
    "Morgan",
    "Bell",
    "Murphy",
    "Bailey",
    "Rivera",
    "Cooper",
    "Richardson",
    "Cox",
    "Howard",
    "Ward",
    "Torres",
    "Peterson",
    "Gray",
    "Ramirez",
    "James",
    "Watson",
    "Brooks",
    "Kelly",
    "Sanders",
    "Price",
    "Bennett",
    "Wood",
    "Barnes",
    "Ross",
    "Henderson",
    "Coleman",
    "Jenkins",
    "Perry",
    "Powell",
    "Long",
    "Patterson",
    "Hughes",
    "Flores",
    "Washington",
    "Butler",
    "Simmons",
    "Foster",
    "Gonzales",
    "Bryant",
    "Alexander",
    "Russell",
    "Griffin",
    "Diaz",
    "Hayes",
    "Myers",
    "Ford",
    "Hamilton",
    "Graham",
    "Sullivan",
    "Wallace",
    "Woods",
    "Cole",
    "West",
    "Jordan",
    "Owens",
    "Reynolds",
    "Fisher",
    "Ellis",
    "Harrison",
    "Gibson",
    "Mcdonald",
    "Cruz",
    "Marshall",
    "Ortiz",
    "Gomez",
    "Murray",
    "Freeman",
    "Wells",
    "Webb",
    "Simpson",
    "Stevens",
    "Tucker",
    "Porter",
    "Hunter",
    "Hicks",
    "Crawford",
    "Henry",
    "Boyd",
];
