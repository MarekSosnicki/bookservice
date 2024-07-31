# `bookservice_repository`

A crate containing a repository types for books.
It contains one public trait `BookRepository` and two implementations of it:
- `InMemoryBookRepository` - containing simple in memory thread safe implementation (allowing single operation at once).
- `DatabaseBookRepository`.- containing database implementation using postgres
