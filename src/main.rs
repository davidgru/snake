
fn print_usage() -> ! {
    println!("Usage");
    std::process::exit(1);
}

fn main() {

    if std::env::args().count() != 3 {
        print_usage();
    }

    let width: usize = match std::env::args().nth(1) {
        Some(arg) => arg.parse().unwrap(),
        None => print_usage()
    };

    let height: usize = match std::env::args().nth(2) {
        Some(arg) => arg.parse().unwrap(),
        None => print_usage()
    };

    println!("width is {}", &width);
    println!("height is {}", &height);
}
