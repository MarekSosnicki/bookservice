use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use itertools::Itertools;

use bookservice_repository::api::BookDetails;
use bookservice_reservations::api::{BookId, ReservationHistoryRecord, UserId};

use crate::api::Recommendations;

pub struct RecommendationsEngine {
    user_to_recommendations: HashMap<UserId, Recommendations>,
    no_of_recommendations: usize,
}

pub struct CoefficientsStorage {
    books_sorted_by_popularity: Vec<BookId>,
    author_to_books_sorted_by_popularity: HashMap<String, Vec<BookId>>,
}

impl RecommendationsEngine {
    pub fn new(
        user_to_reservations: HashMap<UserId, Vec<BookId>>,
        user_to_history: HashMap<UserId, Vec<ReservationHistoryRecord>>,
        book_details: HashMap<BookId, BookDetails>,
        no_of_recommendations: usize,
    ) -> Self {
        let mut popularity_score: HashMap<BookId, i64> = Default::default();
        let mut author_match_score: HashMap<(String, String), i64> = Default::default();
        let mut author_to_books: BTreeMap<String, Vec<BookId>> = Default::default();

        for history_records in user_to_history.values() {
            // BTreeSet ensures that authors are always in the same order for author_match
            let mut user_history_authors: BTreeSet<String> = Default::default();
            for book_id in history_records.iter().map(|r| r.book_id).unique() {
                *popularity_score.entry(book_id).or_default() += 1;
                if let Some(details) = book_details.get(&book_id) {
                    for author in details.authors.iter() {
                        user_history_authors.insert(author.clone());
                        author_to_books
                            .entry(author.clone())
                            .or_default()
                            .push(book_id);
                    }
                } else {
                    tracing::warn!("Could not find details for {book_id}")
                }
            }
            for (author1, author2) in user_history_authors.iter().tuple_combinations() {
                *author_match_score
                    .entry((author1.clone(), author2.clone()))
                    .or_default() += 1;
            }
        }

        // Sort books per author by popularity
        let author_to_books_sorted_by_popularity: HashMap<String, Vec<BookId>> = author_to_books
            .into_iter()
            .map(|(author, books)| {
                (
                    author,
                    books
                        .into_iter()
                        .sorted_by_key(|book_id| {
                            popularity_score.get(book_id).cloned().unwrap_or_default()
                        })
                        .collect(),
                )
            })
            .collect();

        // Sort books by popularity
        let books_sorted_by_popularity = popularity_score
            .into_iter()
            .sorted_by_key(|(_, score)| -score)
            .map(|(book_id, _)| book_id)
            .collect_vec();

        // Generate recommendations for each user
        let user_to_recommendations = user_to_reservations
            .iter()
            .map(|(user_id, reservations)| {
                let all_books_reserved_by_user: HashSet<BookId> = reservations
                    .iter()
                    .cloned()
                    .chain(
                        user_to_history
                            .get(user_id)
                            .iter()
                            .flat_map(|history_records| history_records.iter().map(|r| r.book_id)),
                    )
                    .collect();

                let all_user_authors_with_number_of_books_reserved: HashMap<&String, i64> =
                    all_books_reserved_by_user
                        .iter()
                        .filter_map(|book_id| book_details.get(book_id))
                        .fold(HashMap::default(), |mut map, details| {
                            for author in details.authors.iter() {
                                *map.entry(author).or_default() += 1;
                            }
                            map
                        });

                let author_match: Vec<BookId> = all_user_authors_with_number_of_books_reserved
                    .iter()
                    .sorted_by_key(|(_, score)| -**score)
                    .filter_map(|(author, _)| {
                        author_to_books_sorted_by_popularity
                            .get(*author)
                            .and_then(|author_books| {
                                author_books
                                    .iter()
                                    .filter(|book_id| !all_books_reserved_by_user.contains(book_id))
                                    .next()
                            })
                    })
                    .take(no_of_recommendations)
                    .cloned()
                    .collect();

                // Tak books of 4 authors with best score
                let new_author_match: Vec<BookId> = author_to_books_sorted_by_popularity
                    .keys()
                    .filter(|a| !all_user_authors_with_number_of_books_reserved.contains_key(a))
                    .map(|new_author| {
                        (
                            new_author,
                            all_user_authors_with_number_of_books_reserved
                                .iter()
                                .map(|(user_author, _)| {
                                    author_match_score
                                        .get(&(new_author.clone(), (*user_author).clone()))
                                        .cloned()
                                        .unwrap_or_default()
                                })
                                .sum::<i64>(),
                        )
                    })
                    .sorted_by_key(|(_, score)| -*score)
                    .filter_map(|(author, _)| {
                        author_to_books_sorted_by_popularity
                            .get(author)
                            .and_then(|author_books| author_books.first().cloned())
                    })
                    .take(no_of_recommendations)
                    .collect();

                (
                    *user_id,
                    Recommendations {
                        most_popular: books_sorted_by_popularity
                            .iter()
                            .filter(|book_id| !all_books_reserved_by_user.contains(book_id))
                            .take(no_of_recommendations)
                            .cloned()
                            .collect(),
                        author_match,
                        new_author_match,
                        wild_tags_matches: vec![],
                    },
                )
            })
            .collect();

        Self {
            user_to_recommendations,
            no_of_recommendations,
        }
    }

    pub fn generate_recommendations_for_user(&self, user_id: UserId) -> Recommendations {
        self.user_to_recommendations
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }
}
