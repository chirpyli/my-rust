use std::cmp::{PartialEq, Eq};

#[derive(Debug, PartialEq)]
enum BookFormat {
    Paperback,
    Hardback,
    Ebook,
}

#[derive(Debug)]
struct Book {
    isbn: i32,
    format: BookFormat,
}

impl Book {
    pub fn new(isbn: i32, format: BookFormat) -> Self {
        Book {
            isbn,
            format,
        }
    }
}

// Implement <Book> == <BookFormat> comparisons
impl PartialEq<BookFormat> for Book {
    fn eq(&self, other: &BookFormat) -> bool {
        self.format == *other
    }
}

// Implement <BookFormat> == <Book> comparisons
impl PartialEq<Book> for BookFormat {
    fn eq(&self, other: &Book) -> bool {
        *self == other.format
    }
}


// impl Eq for Book {}


fn main() {
    println!("ParticalEq or Eq:");
    let bf1 = BookFormat::Paperback;
    let bf2 = BookFormat::Hardback;
    let book1 = Book::new(1, BookFormat::Paperback);
    let book2 = Book::new(2, BookFormat::Paperback);
    let book3 = Book::new(1, BookFormat::Paperback);
    assert!(book1 != bf2);
    assert!(bf1 == book1);
    println!("book1:{:?}, book2:{:?}, book3:{:?}", book1, book2, book3);

    println!("NaN {}", f64::NAN == f64::NAN);
    assert!(0.30000000000000000000000000000000000001 == 0.3);
}

