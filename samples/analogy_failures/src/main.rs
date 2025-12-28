// 所有権の類推による誤解を示すサンプルコード

fn main() {
    println!("=== 1. 類推が正しく機能する例 ===\n");
    basic_ownership();

    println!("\n=== 2. 類推が誤解を生む例 ===\n");
    analogy_problem();

    println!("\n=== 3. 正しいアプローチ ===\n");
    correct_approach();
}

fn basic_ownership() {
    let book = String::from("Rust Book");
    println!("本を持っている: {}", book);

    let borrowed_book = book;
    println!("本を渡した後: {}", borrowed_book);
    // println!("{}", book); // コンパイルエラー: value borrowed here after move
    println!("（元の変数 `book` はもう使えない）");
}

fn analogy_problem() {
    struct BookShelf<'a> {
        books: Vec<String>,
        lent_to: Option<&'a String>,
    }

    let mut shelf = BookShelf {
        books: vec![
            String::from("Rust Book"),
            String::from("Programming Rust"),
        ],
        lent_to: None,
    };

    shelf.lent_to = Some(&shelf.books[0]);
    println!("貸し出し中: {:?}", shelf.lent_to);

    // ここで新しい本を追加しようとするとエラー
    // shelf.books.push(String::from("New Book"));
    // error[E0502]: cannot borrow `shelf.books` as mutable
    //               because it is also borrowed as immutably
    
    println!("\nshelf.books.push(...) を呼ぼうとすると:");
    println!("error[E0502]: cannot borrow `shelf.books` as mutable");
    println!("              because it is also borrowed as immutably");
}

fn correct_approach() {
    // アプローチ1: インデックスで管理
    println!("--- アプローチ1: インデックスで管理 ---\n");
    
    struct BookShelfV1 {
        books: Vec<String>,
        lent_index: Option<usize>,
    }

    let mut shelf = BookShelfV1 {
        books: vec![String::from("Rust Book")],
        lent_index: None,
    };

    shelf.lent_index = Some(0);
    println!("貸し出しインデックス: {:?}", shelf.lent_index);
    
    shelf.books.push(String::from("New Book"));
    println!("新しい本を追加: {:?}", shelf.books);
    
    if let Some(idx) = shelf.lent_index {
        println!("貸し出し中の本: {}", shelf.books[idx]);
    }

    // アプローチ2: 所有権を完全に移動
    println!("\n--- アプローチ2: 所有権を完全に移動 ---\n");

    struct BookShelfV2 {
        books: Vec<String>,
    }

    struct LentBook {
        book: String,
        borrower: String,
    }

    let mut shelf = BookShelfV2 {
        books: vec![
            String::from("Rust Book"),
            String::from("Programming Rust"),
        ],
    };
    println!("本棚: {:?}", shelf.books);

    let lent = LentBook {
        book: shelf.books.remove(0),
        borrower: String::from("Alice"),
    };
    println!("貸し出し: {} -> {}", lent.book, lent.borrower);
    println!("本棚: {:?}", shelf.books);

    shelf.books.push(String::from("New Book"));
    println!("新しい本を追加: {:?}", shelf.books);

    // 返却
    shelf.books.push(lent.book);
    println!("返却後: {:?}", shelf.books);
}
