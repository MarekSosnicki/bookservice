use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use itertools::Itertools;

use bookservice_repository::api::BookDetails;
use bookservice_reservations::api::{BookId, ReservationHistoryRecord, UserId};

use crate::api::Recommendations;

const NO_OF_RECOMMENDATIONS: usize = 4;
#[derive(Default)]
pub struct RecommendationsEngine {
    user_to_recommendations: HashMap<UserId, Recommendations>,
    default_recommendations: Recommendations,
}

#[derive(Default)]
pub struct CoefficientsStorage {
    books_sorted_by_popularity: Vec<BookId>,
    author_to_books_sorted_by_popularity: HashMap<String, Vec<BookId>>,
    author_match_score: HashMap<(String, String), i64>,
    popularity_score: HashMap<BookId, i64>,
    author_to_books: BTreeMap<String, Vec<BookId>>,
    book_id_to_authors: HashMap<BookId, Vec<String>>,
    last_processed_timestamp_per_user: HashMap<UserId, i64>,
}

impl CoefficientsStorage {
    pub fn update_storage(
        &mut self,
        user_to_history: &HashMap<UserId, Vec<ReservationHistoryRecord>>,
        book_details: &HashMap<BookId, BookDetails>,
    ) -> anyhow::Result<()> {
        for (book_id, details) in book_details.iter() {
            self.book_id_to_authors
                .insert(*book_id, details.authors.clone());
        }

        for (user_id, history_records) in user_to_history.iter() {
            // BTreeSet ensures that authors are always in the same order for author_match
            let mut user_history_authors: BTreeSet<String> = Default::default();

            let last_processed_timestamp_for_user = self
                .last_processed_timestamp_per_user
                .get(user_id)
                .cloned()
                .unwrap_or(-1);

            for book_id in history_records
                .iter()
                .filter(|r| r.unreserved_at > last_processed_timestamp_for_user)
                .map(|r| r.book_id)
                .unique()
            {
                *self.popularity_score.entry(book_id).or_default() += 1;
                if let Some(details) = book_details.get(&book_id) {
                    for author in details.authors.iter() {
                        user_history_authors.insert(author.clone());
                        self.author_to_books
                            .entry(author.clone())
                            .or_default()
                            .push(book_id);
                    }
                } else {
                    tracing::warn!("Could not find details for {book_id}")
                }
            }
            for (author1, author2) in user_history_authors.iter().tuple_combinations() {
                *self
                    .author_match_score
                    .entry((author1.clone(), author2.clone()))
                    .or_default() += 1;
            }

            self.last_processed_timestamp_per_user.insert(
                *user_id,
                history_records
                    .iter()
                    .map(|r| r.unreserved_at)
                    .max()
                    .unwrap_or_default(),
            );
        }

        // Sort books per author by popularity
        self.author_to_books_sorted_by_popularity = self
            .author_to_books
            .iter()
            .map(|(author, books)| {
                (
                    author.clone(),
                    books
                        .iter()
                        .sorted_by_key(|book_id| {
                            self.popularity_score
                                .get(book_id)
                                .cloned()
                                .unwrap_or_default()
                        })
                        .cloned()
                        .collect(),
                )
            })
            .collect();

        // Sort books by popularity
        self.books_sorted_by_popularity = self
            .popularity_score
            .iter()
            .sorted_by_key(|(_, score)| -**score)
            .map(|(book_id, _)| *book_id)
            .collect_vec();

        Ok(())
    }
}

impl RecommendationsEngine {
    pub fn update_recommendations_for_users(
        &mut self,
        coefficients_storage: &CoefficientsStorage,
        user_to_reservations: &HashMap<UserId, Vec<BookId>>,
        user_to_history: &HashMap<UserId, Vec<ReservationHistoryRecord>>,
    ) -> anyhow::Result<()> {
        println!(
            "Updating recommendations for {} users",
            user_to_reservations.len()
        );

        // Generate recommendations for each user
        user_to_reservations
            .iter()
            .for_each(|(user_id, reservations)| {
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
                        .filter_map(|book_id| coefficients_storage.book_id_to_authors.get(book_id))
                        .fold(HashMap::default(), |mut map, authors| {
                            for author in authors.iter() {
                                *map.entry(author).or_default() += 1;
                            }
                            map
                        });

                let author_match: Vec<BookId> = all_user_authors_with_number_of_books_reserved
                    .iter()
                    .sorted_by_key(|(_, score)| -**score)
                    .filter_map(|(author, _)| {
                        coefficients_storage
                            .author_to_books_sorted_by_popularity
                            .get(*author)
                            .and_then(|author_books| {
                                author_books
                                    .iter()
                                    .filter(|book_id| !all_books_reserved_by_user.contains(book_id))
                                    .next()
                            })
                    })
                    .take(NO_OF_RECOMMENDATIONS)
                    .cloned()
                    .collect();

                // Tak books of 4 authors with best score
                let new_author_match: Vec<BookId> = coefficients_storage
                    .author_to_books_sorted_by_popularity
                    .keys()
                    .filter(|a| !all_user_authors_with_number_of_books_reserved.contains_key(a))
                    .map(|new_author| {
                        (
                            new_author,
                            all_user_authors_with_number_of_books_reserved
                                .iter()
                                .map(|(user_author, _)| {
                                    coefficients_storage
                                        .author_match_score
                                        .get(&(new_author.clone(), (*user_author).clone()))
                                        .cloned()
                                        .unwrap_or_default()
                                })
                                .sum::<i64>(),
                        )
                    })
                    .sorted_by_key(|(_, score)| -*score)
                    .filter_map(|(author, _)| {
                        coefficients_storage
                            .author_to_books_sorted_by_popularity
                            .get(author)
                            .and_then(|author_books| author_books.first().cloned())
                    })
                    .take(NO_OF_RECOMMENDATIONS)
                    .collect();

                let recommendations = Recommendations {
                    most_popular: coefficients_storage
                        .books_sorted_by_popularity
                        .iter()
                        .filter(|book_id| !all_books_reserved_by_user.contains(book_id))
                        .take(NO_OF_RECOMMENDATIONS)
                        .cloned()
                        .collect(),
                    author_match,
                    new_author_match,
                };

                tracing::info!(
                    "Adding recommendations for user {} : {:?}",
                    user_id,
                    recommendations
                );

                self.user_to_recommendations
                    .insert(*user_id, recommendations);
            });

        self.default_recommendations = Recommendations {
            most_popular: coefficients_storage
                .books_sorted_by_popularity
                .iter()
                .take(NO_OF_RECOMMENDATIONS)
                .cloned()
                .collect(),
            author_match: vec![],
            new_author_match: vec![],
        };

        Ok(())
    }

    pub fn get_recommendations_for_user(&self, user_id: UserId) -> Recommendations {
        self.user_to_recommendations
            .get(&user_id)
            .cloned()
            .unwrap_or_else(|| self.default_recommendations.clone())
    }
}
